use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};
use std::collections::HashSet;

use crate::camera::Camera;
use crate::world::World;
use crate::vertex::Vertex;
use crate::uniforms::Uniforms;
use crate::world_update::update_world;
use crate::texture::Texture; // Correctly import the Texture struct

pub async fn run(event_loop: EventLoop<()>, window: winit::window::Window) {
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
        present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &config);

    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    // Use the Texture module to load the texture
    let texture = Texture::from_image(&device, &queue, "P:\\Code\\mutetra\\src\\images\\dirt\\dirt.png");

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
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
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
            _ => {}
        }
    });
}
