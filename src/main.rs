use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use cgmath::{prelude::*, Matrix4, Point3, Rad, Vector3};
use bytemuck::{Pod, Zeroable};
use image::GenericImageView;
use std::collections::{HashSet, HashMap};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

const VERTICES: &[Vertex] = &[
    // Front face
    Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 1.0] },

    // Back face
    Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-1.0,  1.0, -1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 0.0] },

    // Top face
    Vertex { position: [-1.0, 1.0, -1.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [-1.0, 1.0,  1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 1.0, 1.0,  1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 1.0, 1.0, -1.0], tex_coords: [1.0, 1.0] },

    // Bottom face
    Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [1.0, 0.0] },

    // Right face
    Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [0.0, 0.0] },

    // Left face
    Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-1.0,  1.0,  1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [-1.0,  1.0, -1.0], tex_coords: [0.0, 1.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // front
    4, 5, 6, 6, 7, 4, // back
    8, 9, 10, 10, 11, 8, // top
    12, 13, 14, 14, 15, 12, // bottom
    16, 17, 18, 18, 19, 16, // right
    20, 21, 22, 22, 23, 20, // left
];

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

    fn update_view_proj(&mut self, camera: &Camera) {
        let view = Matrix4::look_at_rh(camera.eye, camera.target, camera.up);
        let projection = cgmath::perspective(Rad(camera.fovy), camera.aspect, camera.znear, camera.zfar);
        self.view_proj = (projection * view).into();
    }
}

struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    fovy: f32,
    aspect: f32,
    znear: f32,
    zfar: f32,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    fn new() -> Self {
        Self {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            fovy: 45.0f32.to_radians(),
            aspect: 16.0 / 9.0,
            znear: 0.1,
            zfar: 100.0,
            yaw: -90.0f32.to_radians(),
            pitch: 0.0,
        }
    }

    fn update_camera_vectors(&mut self) {
        let front = Vector3 {
            x: self.yaw.cos() * self.pitch.cos(),
            y: self.pitch.sin(),
            z: self.yaw.sin() * self.pitch.cos(),
        };
        self.target = self.eye + front.normalize();
    }

    fn process_mouse_movement(&mut self, xoffset: f32, yoffset: f32, sensitivity: f32) {
        self.yaw += xoffset * sensitivity;
        self.pitch += yoffset * sensitivity;

        if self.pitch > 89.0f32.to_radians() {
            self.pitch = 89.0f32.to_radians();
        }
        if self.pitch < -89.0f32.to_radians() {
            self.pitch = -89.0f32.to_radians();
        }

        self.update_camera_vectors();
    }

    fn move_forward(&mut self, amount: f32) {
        let forward = (self.target - self.eye).normalize();
        self.eye += forward * amount;
        self.target += forward * amount;
    }

    fn strafe_right(&mut self, amount: f32) {
        let forward = (self.target - self.eye).normalize();
        let right = forward.cross(self.up).normalize();
        self.eye += right * amount;
        self.target += right * amount;
    }

    fn move_up(&mut self, amount: f32) {
        let up = Vector3::unit_y();
        self.eye += up * amount;
        self.target += up * amount;
    }
}

struct Chunk {
    position: (i32, i32),
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

struct World {
    chunks: HashMap<(i32, i32), Chunk>,
    chunk_size: usize,
}

impl World {
    fn new(chunk_size: usize) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_size,
        }
    }

    fn load_chunk(&mut self, chunk_pos: (i32, i32)) {
        if !self.chunks.contains_key(&chunk_pos) {
            let vertices = generate_chunk_vertices(chunk_pos, self.chunk_size);
            let indices = generate_chunk_indices(self.chunk_size);

            self.chunks.insert(chunk_pos, Chunk {
                position: chunk_pos,
                vertices,
                indices,
            });
        }
    }
}

fn generate_chunk_vertices(chunk_pos: (i32, i32), chunk_size: usize) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for x in 0..chunk_size {
        for z in 0..chunk_size {
            let base_position = [
                x as f32 + (chunk_pos.0 * chunk_size as i32) as f32,
                0.0,
                z as f32 + (chunk_pos.1 * chunk_size as i32) as f32,
            ];
            for vertex in VERTICES.iter() {
                let mut position = vertex.position;
                position[0] += base_position[0];
                position[2] += base_position[2];
                vertices.push(Vertex {
                    position,
                    tex_coords: vertex.tex_coords,
                });
            }
        }
    }
    vertices
}

fn generate_chunk_indices(chunk_size: usize) -> Vec<u16> {
    let mut indices = Vec::new();
    let vertex_count = INDICES.len() as u16;
    for x in 0..chunk_size {
        for z in 0..chunk_size {
            let offset = ((x * chunk_size + z) * vertex_count as usize) as u16;
            indices.extend(INDICES.iter().map(|i| i + offset));
        }
    }
    indices
}

fn update_world(camera: &Camera, world: &mut World) {
    let current_chunk_pos = (
        (camera.eye.x / (world.chunk_size as f32)).floor() as i32,
        (camera.eye.z / (world.chunk_size as f32)).floor() as i32,
    );

    for dx in -1..=1 {
        for dz in -1..=1 {
            world.load_chunk((current_chunk_pos.0 + dx, current_chunk_pos.1 + dz));
        }
    }
}

async fn run(event_loop: EventLoop<()>, window: winit::window::Window) {
    let size = window.inner_size();
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
        present_mode: wgpu::PresentMode::Mailbox, // Switch to Mailbox
    };

    surface.configure(&device, &config);

    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let img_path = "P:\\Code\\mutetra\\src\\images\\dirt\\dirt.png";  // Ensure path is correct

    let img = image::open(img_path).expect("Failed to open texture image: Check file path and existence");

    let rgba = img.to_rgba8();
    let dimensions = img.dimensions();

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
            rows_per_image: std::num::NonZeroU32::new(dimensions.1),
        },
        texture_size,
    );

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat, // This line is added
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        compare: None,
        anisotropy_clamp: std::num::NonZeroU8::new(16), // Consider enabling anisotropic filtering
        border_color: None,
        label: Some("Texture Sampler"),
    });

    let mut uniforms = Uniforms::new();
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[uniforms]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("uniform_texture_bind_group_layout"),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("uniform_texture_bind_group"),
    });

    // Create a depth texture
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    });

    let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
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
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut camera = Camera::new();
    camera.aspect = config.width as f32 / config.height as f32;

    let mut world = World::new(4); // Set the chunk size here
    update_world(&camera, &mut world);

    let mut dynamic_vertex_buffer_size = 1024 * 1024 * std::mem::size_of::<Vertex>() as u64;
    let mut dynamic_index_buffer_size = 1024 * 1024 * std::mem::size_of::<u16>() as u64;

    let mut dynamic_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Dynamic Vertex Buffer"),
        size: dynamic_vertex_buffer_size,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut dynamic_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Dynamic Index Buffer"),
        size: dynamic_index_buffer_size,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut last_frame_time = std::time::Instant::now();
    let mut pressed_keys = HashSet::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        config.width = new_size.width;
                        config.height = new_size.height;
                        camera.aspect = config.width as f32 / config.height as f32;
                        surface.configure(&device, &config);
                        uniforms.update_view_proj(&camera);
                        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
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
                    let xoffset = position.x as f32 - (size.width / 2) as f32;
                    let yoffset = (size.height / 2) as f32 - position.y as f32;

                    let sensitivity = 0.005;
                    camera.process_mouse_movement(xoffset, yoffset, sensitivity);

                    let _ = window.set_cursor_position(winit::dpi::PhysicalPosition::new(size.width / 2, size.height / 2));
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                // Update camera based on input
                let move_amount = 0.05;
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

                update_world(&camera, &mut world);
                
                let mut total_vertices: Vec<Vertex> = vec![];
                let mut total_indices = vec![];
                let mut index_offset = 0;

                for chunk in world.chunks.values() {
                    total_vertices.extend(&chunk.vertices);

                    let indices: Vec<u16> = chunk
                        .indices
                        .iter()
                        .map(|i| *i + index_offset as u16)
                        .collect();
                    index_offset += chunk.vertices.len() as u16;
                    total_indices.extend(indices);
                }

                let total_vertices_bytes = (total_vertices.len() * std::mem::size_of::<Vertex>()) as u64;
                let total_indices_bytes = (total_indices.len() * std::mem::size_of::<u16>()) as u64;

                if total_vertices_bytes > dynamic_vertex_buffer_size {
                    dynamic_vertex_buffer_size = total_vertices_bytes;
                    dynamic_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Expanded Vertex Buffer"),
                        contents: bytemuck::cast_slice(&total_vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
                } else {
                    queue.write_buffer(&dynamic_vertex_buffer, 0, bytemuck::cast_slice(&total_vertices));
                }

                if total_indices_bytes > dynamic_index_buffer_size {
                    dynamic_index_buffer_size = total_indices_bytes;
                    dynamic_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Expanded Index Buffer"),
                        contents: bytemuck::cast_slice(&total_indices),
                        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    });
                } else {
                    queue.write_buffer(&dynamic_index_buffer, 0, bytemuck::cast_slice(&total_indices));
                }

                let current_frame_time = std::time::Instant::now();
                let dt = current_frame_time.duration_since(last_frame_time).as_secs_f32();
                last_frame_time = current_frame_time;

                // Use dt for smooth rotation and movements
                uniforms.update_model(dt * 0.1); // Ensures model rotates at a steady speed
                uniforms.update_view_proj(&camera);
                queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

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
            _ => {},
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let monitor = window.primary_monitor();
    window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(monitor)));

    // Attempt to grab the cursor and make it invisible
    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);

    pollster::block_on(run(event_loop, window));
}