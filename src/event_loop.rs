use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    event::WindowEvent,
    event::ElementState,
    window::{WindowId, Window},
    event_loop::ActiveEventLoop,
    application::ApplicationHandler,
    keyboard::{PhysicalKey, KeyCode}
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use crate::camera::Camera;
use crate::world::World;
use crate::vertex::Vertex;
use crate::uniforms::Uniforms;
use crate::world_update::update_world;
use crate::texture::get_texture;
use wgpu::util::DeviceExt;

struct AppHandler<'a> {
    surface: Arc<wgpu::Surface<'a>>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: Arc<Mutex<wgpu::SurfaceConfiguration>>,
    bind_group: Arc<wgpu::BindGroup>,
    render_pipeline: Arc<wgpu::RenderPipeline>,
    dynamic_vertex_buffer: Arc<Mutex<wgpu::Buffer>>,
    dynamic_index_buffer: Arc<Mutex<wgpu::Buffer>>,
    uniform_buffer: Arc<wgpu::Buffer>,
    depth_texture_view: Arc<wgpu::TextureView>,
    camera: Arc<Mutex<Camera>>,
    world: Arc<Mutex<World>>,
    uniforms: Arc<Mutex<Uniforms>>,
    last_frame_time: Arc<Mutex<std::time::Instant>>,
    pressed_keys: Arc<Mutex<HashSet<KeyCode>>>,
    log_frame_count: usize,
    last_camera_position: [f32; 3],
    movement_threshold: f32,
    dynamic_vertex_buffer_size: Arc<Mutex<u64>>, // Added
    dynamic_index_buffer_size: Arc<Mutex<u64>>,  // Added
    window: Arc<winit::window::Window>,
}

impl AppHandler<'_> {
    fn handle_event(&mut self, event: &Event<()>, control_flow: &mut ControlFlow, window: &Window) {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        let mut config = self.config.lock().unwrap();
                        config.width = new_size.width;
                        config.height = new_size.height;
                        let mut camera = self.camera.lock().unwrap();
                        camera.aspect = config.width as f32 / config.height as f32;
                        self.surface.configure(&self.device, &config);
                        let mut uniforms = self.uniforms.lock().unwrap();
                        uniforms.update_view_proj(&camera);
                        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
                    }
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Wait,
                WindowEvent::KeyboardInput { event, .. } => {
                    let mut pressed_keys = self.pressed_keys.lock().unwrap();
                    match event.state {
                        ElementState::Pressed => {
                            match event.physical_key {
                                PhysicalKey::Code(key_code) => match key_code {
                                    KeyCode::KeyW | KeyCode::KeyS | KeyCode::KeyA | KeyCode::KeyD => {
                                        pressed_keys.insert(key_code);
                                    }
                                    // Include other keys as needed
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                        ElementState::Released => {
                            match event.physical_key {
                                PhysicalKey::Code(key_code) => {
                                    pressed_keys.remove(&key_code);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let mut camera = self.camera.lock().unwrap();
                    let config = self.config.lock().unwrap();
                    let xoffset = position.x as f32 - (config.width / 2) as f32;
                    let yoffset = (config.height / 2) as f32 - position.y as f32;

                    let sensitivity = 0.005;
                    camera.process_mouse_movement(xoffset, yoffset, sensitivity);

                    let _ = window.set_cursor_position(winit::dpi::PhysicalPosition::new(config.width / 2, config.height / 2));
                }
                WindowEvent::RedrawRequested => {
                    self.log_frame_count += 1;

                    let move_amount = 0.05;
                    let mut camera = self.camera.lock().unwrap();
                    let pressed_keys = self.pressed_keys.lock().unwrap();
                    if pressed_keys.contains(&KeyCode::KeyW) {
                        camera.move_forward(move_amount);
                    }
                    if pressed_keys.contains(&KeyCode::KeyS) {
                        camera.move_forward(-move_amount);
                    }
                    if pressed_keys.contains(&KeyCode::KeyA) {
                        camera.strafe_right(-move_amount);
                    }
                    if pressed_keys.contains(&KeyCode::KeyD) {
                        camera.strafe_right(move_amount);
                    }
                    if pressed_keys.contains(&KeyCode::Space) {
                        camera.move_up(move_amount);
                    }
                    if pressed_keys.contains(&KeyCode::ShiftLeft) {
                        camera.move_up(-move_amount);
                    }

                    let current_position = [camera.eye.x, camera.eye.y, camera.eye.z];
                    let distance_moved = ((current_position[0] - self.last_camera_position[0]).powi(2) +
                                          (current_position[1] - self.last_camera_position[1]).powi(2) +
                                          (current_position[2] - self.last_camera_position[2]).powi(2)).sqrt();

                    if distance_moved > self.movement_threshold {
                        let mut world = self.world.lock().unwrap();
                        update_world(&camera, &mut world);
                        self.last_camera_position = current_position;
                    }

                    let mut total_vertices: Vec<Vertex> = vec![];
                    let mut total_indices: Vec<u16> = vec![];
                    let mut index_offset: u16 = 0;

                    let world = self.world.lock().unwrap();
                    for chunk in world.chunks.values() {
                        total_vertices.extend(&chunk.vertices);

                        let indices: Vec<u16> = chunk
                            .indices
                            .iter()
                            .map(|i| *i + index_offset)
                            .collect();
                        index_offset += chunk.vertices.len() as u16;
                        total_indices.extend(indices);
                    }

                    let total_vertices_bytes = (total_vertices.len() * std::mem::size_of::<Vertex>()) as u64;
                    let total_indices_bytes = (total_indices.len() * std::mem::size_of::<u16>()) as u64;

                    if self.log_frame_count % 1000 == 0 {
                        println!("Rendering loop executed.");
                        println!("Dynamic Vertex Buffer Size: {}, Dynamic Index Buffer Size: {}", total_vertices_bytes, total_indices_bytes);
                    }

                    let mut dynamic_vertex_buffer_size = self.dynamic_vertex_buffer_size.lock().unwrap();
                    let mut dynamic_vertex_buffer = self.dynamic_vertex_buffer.lock().unwrap();
                    if total_vertices_bytes > *dynamic_vertex_buffer_size {
                        *dynamic_vertex_buffer_size = total_vertices_bytes;
                        *dynamic_vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Expanded Vertex Buffer"),
                            contents: bytemuck::cast_slice(&total_vertices),
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        });
                    } else {
                        self.queue.write_buffer(&*dynamic_vertex_buffer, 0, bytemuck::cast_slice(&total_vertices));
                    }

                    let mut dynamic_index_buffer_size = self.dynamic_index_buffer_size.lock().unwrap();
                    let mut dynamic_index_buffer = self.dynamic_index_buffer.lock().unwrap();
                    if total_indices_bytes > *dynamic_index_buffer_size {
                        *dynamic_index_buffer_size = total_indices_bytes;
                        let new_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Expanded Index Buffer"),
                            contents: bytemuck::cast_slice(&total_indices),
                            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                        });
                        *dynamic_index_buffer = new_buffer;
                    } else {
                        self.queue.write_buffer(&*dynamic_index_buffer, 0, bytemuck::cast_slice(&total_indices));
                    }

                    let current_frame_time = std::time::Instant::now();
                    let mut last_frame_time = self.last_frame_time.lock().unwrap();
                    let _dt = current_frame_time.duration_since(*last_frame_time).as_secs_f32();
                    *last_frame_time = current_frame_time;

                    let mut uniforms = self.uniforms.lock().unwrap();
                    uniforms.update_model();
                    uniforms.update_view_proj(&camera);
                    self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));

                    let output = self.surface.get_current_texture().expect("Failed to acquire next swap chain texture");
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &self.depth_texture_view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(1.0),
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });

                        render_pass.set_pipeline(&self.render_pipeline);
                        render_pass.set_bind_group(0, &*self.bind_group, &[]);

                        render_pass.set_vertex_buffer(0, dynamic_vertex_buffer.slice(..));
                        render_pass.set_index_buffer(dynamic_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(0..total_indices.len() as u32, 0, 0..1);
                    }

                    self.queue.submit(Some(encoder.finish()));
                    output.present();
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl ApplicationHandler<()> for AppHandler<'_> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // Placeholder for resumed event handling
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Extract the reference to the `Window` from `Arc<Window>`
        let window_ref = Arc::clone(&self.window);
        // Call the existing handle_event method to process window events
        self.handle_event(&Event::WindowEvent { window_id, event }, &mut ControlFlow::Poll, &window_ref);
    }
}

pub fn handle_event_loop(
    event_loop: EventLoop<()>, 
    window: Arc<winit::window::Window>, 
    device: Arc<wgpu::Device>, 
    queue: Arc<wgpu::Queue>, 
    surface: Arc<wgpu::Surface>, 
    config: Arc<Mutex<wgpu::SurfaceConfiguration>>, 
    bind_group: Arc<wgpu::BindGroup>, 
    render_pipeline: Arc<wgpu::RenderPipeline>, 
    dynamic_vertex_buffer: Arc<Mutex<wgpu::Buffer>>, 
    dynamic_index_buffer: Arc<Mutex<wgpu::Buffer>>, 
    uniform_buffer: Arc<wgpu::Buffer>, 
    depth_texture_view: Arc<wgpu::TextureView>, 
    camera: Arc<Mutex<Camera>>, 
    world: Arc<Mutex<World>>, 
    uniforms: Arc<Mutex<Uniforms>>,
) {
    let last_frame_time = Arc::new(Mutex::new(std::time::Instant::now()));
    let pressed_keys = Arc::new(Mutex::new(HashSet::new()));

    let dynamic_vertex_buffer_size = Arc::new(Mutex::new(1024 * 1024 * std::mem::size_of::<Vertex>() as u64));
    let dynamic_index_buffer_size = Arc::new(Mutex::new(1024 * 1024 * std::mem::size_of::<u16>() as u64));

    let _texture = get_texture(&device, &queue, [
        "src/images/pos_x.png",
        "src/images/neg_x.png",
        "src/images/pos_y.png",
        "src/images/neg_y.png",
        "src/images/pos_z.png",
        "src/images/neg_z.png",
    ]);

    let mut app_handler = AppHandler {
        surface,
        device,
        queue,
        config,
        bind_group,
        render_pipeline,
        dynamic_vertex_buffer,
        dynamic_index_buffer,
        uniform_buffer,
        depth_texture_view,
        camera,
        world,
        uniforms,
        last_frame_time,
        pressed_keys,
        log_frame_count: 0,
        last_camera_position: [0.0, 0.0, 0.0],
        movement_threshold: 10.0,
        dynamic_vertex_buffer_size, // Added
        dynamic_index_buffer_size,   // Added
        window,
    };

    let _ = event_loop.run_app(&mut app_handler);
}
