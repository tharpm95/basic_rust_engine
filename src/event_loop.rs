use winit::{
    event::*,
    event_loop::ControlFlow,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use crate::camera::Camera;
use crate::world::World;
use crate::vertex::Vertex;
use crate::uniforms::Uniforms;
use crate::world_update::update_world;
use crate::texture::get_texture; // Import the get_texture function
use wgpu::util::DeviceExt; // Import DeviceExt for create_buffer_init
use log::{info}; // Import logging macros

pub fn handle_event_loop(
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
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

    // Manually track buffer sizes
    let dynamic_vertex_buffer_size = Arc::new(Mutex::new(1024 * 1024 * std::mem::size_of::<Vertex>() as u64));
    let dynamic_index_buffer_size = Arc::new(Mutex::new(1024 * 1024 * std::mem::size_of::<u16>() as u64));

    // Load the texture once
    let _texture = get_texture(&device, &queue, "src/images/dirt/dirt.png"); // Adjust the path as necessary

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        let mut config = config.lock().unwrap();
                        config.width = new_size.width;
                        config.height = new_size.height;
                        let mut camera = camera.lock().unwrap();
                        camera.aspect = config.width as f32 / config.height as f32;
                        surface.configure(&device, &config);
                        let mut uniforms = uniforms.lock().unwrap();
                        uniforms.update_view_proj(&camera);
                        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
                    }
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                    .. 
                } => {
                    let mut pressed_keys = pressed_keys.lock().unwrap();
                    match state {
                        ElementState::Pressed => {
                            pressed_keys.insert(keycode);
                        }
                        ElementState::Released => {
                            pressed_keys.remove(&keycode);
                        }
                    }
                }
                WindowEvent::CursorMoved { device_id: _, position, .. } => {
                    let mut camera = camera.lock().unwrap();
                    let config = config.lock().unwrap();
                    let xoffset = position.x as f32 - (config.width / 2) as f32;
                    let yoffset = (config.height / 2) as f32 - position.y as f32;

                    let sensitivity = 0.005;
                    camera.process_mouse_movement(xoffset, yoffset, sensitivity);

                    let _ = window.set_cursor_position(winit::dpi::PhysicalPosition::new(config.width / 2, config.height / 2));
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                // Update camera based on input
                let move_amount = 0.05;
                let mut camera = camera.lock().unwrap();
                let pressed_keys = pressed_keys.lock().unwrap();
                if pressed_keys.contains(&VirtualKeyCode::W) {
                    camera.move_forward(move_amount);
                }
                if pressed_keys.contains(&VirtualKeyCode::S) {
                    camera.move_forward(-move_amount);
                }
                if pressed_keys.contains(&VirtualKeyCode::A) {
                    camera.strafe_right(-move_amount);
                }
                if pressed_keys.contains(&VirtualKeyCode::D) {
                    camera.strafe_right(move_amount);
                }
                if pressed_keys.contains(&VirtualKeyCode::Space) {
                    camera.move_up(move_amount);
                }
                if pressed_keys.contains(&VirtualKeyCode::LShift) {
                    camera.move_up(-move_amount);
                }

                let mut world = world.lock().unwrap();
                update_world(&camera, &mut world);
                
                let mut total_vertices: Vec<Vertex> = vec![];
                let mut total_indices = vec![];
                let mut index_offset: u16 = 0; // Changed back to u16

                for chunk in world.chunks.values() {
                    total_vertices.extend(&chunk.vertices);

                    let indices: Vec<u16> = chunk
                        .indices
                        .iter()
                        .map(|i| *i + index_offset) // Keep as u16
                        .collect();
                    index_offset += chunk.vertices.len() as u16; // Keep as u16
                    total_indices.extend(indices);
                }

                let total_vertices_bytes = (total_vertices.len() * std::mem::size_of::<Vertex>()) as u64;
                let total_indices_bytes = (total_indices.len() * std::mem::size_of::<u16>()) as u64;

                // Log the sizes of vertices and indices
                info!("Total Vertices: {}, Total Indices: {}", total_vertices.len(), total_indices.len());
                info!("Dynamic Vertex Buffer Size: {}, Dynamic Index Buffer Size: {}", total_vertices_bytes, total_indices_bytes);

                let mut dynamic_vertex_buffer_size = dynamic_vertex_buffer_size.lock().unwrap();
                let mut dynamic_vertex_buffer = dynamic_vertex_buffer.lock().unwrap();
                if total_vertices_bytes > *dynamic_vertex_buffer_size {
                    *dynamic_vertex_buffer_size = total_vertices_bytes;
                    *dynamic_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Expanded Vertex Buffer"),
                        contents: bytemuck::cast_slice(&total_vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
                } else {
                    queue.write_buffer(&dynamic_vertex_buffer, 0, bytemuck::cast_slice(&total_vertices));
                }

                let mut dynamic_index_buffer_size = dynamic_index_buffer_size.lock().unwrap();
                let mut dynamic_index_buffer = dynamic_index_buffer.lock().unwrap();
                if total_indices_bytes > *dynamic_index_buffer_size {
                    *dynamic_index_buffer_size = total_indices_bytes;
                    *dynamic_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Expanded Index Buffer"),
                        contents: bytemuck::cast_slice(&total_indices),
                        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    });
                } else {
                    queue.write_buffer(&dynamic_index_buffer, 0, bytemuck::cast_slice(&total_indices));
                }

                let current_frame_time = std::time::Instant::now();
                let mut last_frame_time = last_frame_time.lock().unwrap();
                let _dt = current_frame_time.duration_since(*last_frame_time).as_secs_f32();
                *last_frame_time = current_frame_time;

                // Use a fixed rotation speed to reduce tearing
                let mut uniforms = uniforms.lock().unwrap();
                uniforms.update_model(0.05); // Fixed rotation speed
                uniforms.update_view_proj(&camera);
                queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));

                let output = surface.get_current_texture().expect("Failed to acquire next swap chain texture");
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_texture_view,
                            depth_ops: Some(wgpu::Operations { 
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true
                            }),
                            stencil_ops: None,
                        }),
                    });

                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    
                    render_pass.set_vertex_buffer(0, dynamic_vertex_buffer.slice(..));
                    render_pass.set_index_buffer(dynamic_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..total_indices.len() as u32, 0, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                output.present();
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            _ => {}
        }
    });
}
