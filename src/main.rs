use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use cgmath::{Matrix4, Point3, Rad, Vector3, SquareMatrix}; 
use std::f32::consts::PI;
use bytemuck::{Pod, Zeroable};

// Struct to manage camera orientation through mouse input
struct CameraController {
    yaw: f32,
    pitch: f32,
    last_mouse_position: Option<(f64, f64)>,
}

impl CameraController {
    fn new() -> Self {
        Self {
            yaw: PI,  // Adjust to face the desired direction
            pitch: 0.0,  // Set pitch level
            last_mouse_position: None,
        }
    }

    fn process_mouse(&mut self, dx: f64, dy: f64) {
        let sensitivity = 0.005; // Decrease sensitivity for slower mouse control
        self.yaw += dx as f32 * sensitivity;  // Invert yaw control
        self.pitch -= dy as f32 * sensitivity; // Invert pitch control
        self.pitch = self.pitch.clamp(-PI / 2.0, PI / 2.0);
    }

    fn update_view_proj(&self, aspect_ratio: f32, uniforms: &mut Uniforms) {
        let direction = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );

        // Direct the camera to face the cube located at the origin
        let eye = Point3::new(0.0, 0.0, 5.0);
        let target = eye + direction; // Look towards the direction
        let up = Vector3::unit_y();

        let view = Matrix4::look_at_rh(eye, target, up);

        let proj = cgmath::perspective(Rad(PI / 4.0), aspect_ratio, 0.1, 100.0);
        uniforms.view_proj = (proj * view).into();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
            model: Matrix4::identity().into(),
        }
    }

    fn update_model(&mut self, rotation: f32) {
        let model = Matrix4::from_angle_y(Rad(rotation));
        self.model = model.into();
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, -1.0] },
    Vertex { position: [ 1.0, -1.0, -1.0] },
    Vertex { position: [ 1.0,  1.0, -1.0] },
    Vertex { position: [-1.0,  1.0, -1.0] },
    Vertex { position: [-1.0, -1.0,  1.0] },
    Vertex { position: [ 1.0, -1.0,  1.0] },
    Vertex { position: [ 1.0,  1.0,  1.0] },
    Vertex { position: [-1.0,  1.0,  1.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0,
    4, 5, 6, 6, 7, 4,
    0, 1, 5, 5, 4, 0,
    2, 3, 7, 7, 6, 2,
    0, 3, 7, 7, 4, 0,
    1, 2, 6, 6, 5, 1,
];

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
}

async fn run(event_loop: EventLoop<()>, window: winit::window::Window) {
    let mut size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }).await.unwrap();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .unwrap();

    let swapchain_format = surface.get_preferred_format(&adapter).unwrap();
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &config);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    let index_count = INDICES.len() as u32;

    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let mut uniforms = Uniforms::new();
    let mut camera_controller = CameraController::new(); 

    camera_controller.update_view_proj(size.width as f32 / size.height as f32, &mut uniforms);

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[uniforms]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("uniform_bind_group_layout"),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("uniform_bind_group"),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: None, 
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut last_update_inst = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => {
                    size = new_size;
                    if size.width > 0 && size.height > 0 {
                        config.width = size.width;
                        config.height = size.height;
                        surface.configure(&device, &config);

                        // Update the aspect ratio and view projection
                        camera_controller.update_view_proj(size.width as f32 / size.height as f32, &mut uniforms);
                        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if let Some((previous_x, previous_y)) = camera_controller.last_mouse_position {
                        let dx = position.x - previous_x;
                        let dy = position.y - previous_y;
                        camera_controller.process_mouse(dx, dy);
                        camera_controller.update_view_proj(size.width as f32 / size.height as f32, &mut uniforms);
                        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
                    }
                    camera_controller.last_mouse_position = Some((position.x, position.y));
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let duration = now - last_update_inst;
                last_update_inst = now;

                uniforms.update_model(duration.as_secs_f32());
                queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

                let output = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

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
                        depth_stencil_attachment: None,
                    });

                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..index_count, 0, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                output.present();
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            _ => {},
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    pollster::block_on(run(event_loop, window));
}