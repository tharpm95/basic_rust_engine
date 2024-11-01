use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;
use winit::{
    event_loop::EventLoop,
};

use crate::camera::Camera;
use crate::world::World;
use crate::vertex::Vertex;
use crate::uniforms::Uniforms;
use crate::world_update::update_world;
use crate::texture::Texture;
use crate::event_loop::handle_event_loop; // Correctly import the handle_event_loop function

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
    let config = wgpu::SurfaceConfiguration { // Removed unnecessary mut
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

    let uniforms = Uniforms::new(); // Removed unnecessary mut
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
            bias: wgpu::DepthBiasState {
                constant: 0,
                slope_scale: 0.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let camera = Camera::new();
    let mut world = World::new(8); // Increased chunk size to 8
    update_world(&camera, &mut world);

    let dynamic_vertex_buffer_size = 1024 * 1024 * std::mem::size_of::<Vertex>() as u64; // Removed unnecessary mut
    let dynamic_index_buffer_size = 1024 * 1024 * std::mem::size_of::<u16>() as u64; // Removed unnecessary mut

    let dynamic_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Dynamic Vertex Buffer"),
        size: dynamic_vertex_buffer_size,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let dynamic_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Dynamic Index Buffer"),
        size: dynamic_index_buffer_size,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Wrap necessary arguments in Arc and Mutex
    let device = Arc::new(device);
    let queue = Arc::new(queue);
    let surface = Arc::new(surface);
    let config = Arc::new(Mutex::new(config));
    let bind_group = Arc::new(bind_group);
    let render_pipeline = Arc::new(render_pipeline);
    let dynamic_vertex_buffer = Arc::new(Mutex::new(dynamic_vertex_buffer));
    let dynamic_index_buffer = Arc::new(Mutex::new(dynamic_index_buffer));
    let uniform_buffer = Arc::new(uniform_buffer);
    let depth_texture_view = Arc::new(depth_texture_view);
    let camera = Arc::new(Mutex::new(camera));
    let world = Arc::new(Mutex::new(world));
    let uniforms = Arc::new(Mutex::new(uniforms));

    // Call the handle_event_loop function from the event_loop module
    handle_event_loop(
        event_loop,
        window,
        device,
        queue,
        surface,
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
    );
}
