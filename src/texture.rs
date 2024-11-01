use std::sync::{Arc, Mutex};

#[allow(dead_code)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_images(device: &wgpu::Device, queue: &wgpu::Queue, paths: [&str; 6]) -> Self {
        let mut rgba_images = Vec::new();
        let mut texture_size = wgpu::Extent3d {
            width: 0,
            height: 0,
            depth_or_array_layers: 6, // Set depth to 6 for cubemap
        };

        for (i, path) in paths.iter().enumerate() {
            let img = image::open(path).expect("Failed to open texture image: Check file path and existence");
            let rgba = img.to_rgba8();
            rgba_images.push(rgba.clone()); // Clone the rgba image

            // Set texture size based on the first image
            if i == 0 {
                texture_size.width = rgba.width();
                texture_size.height = rgba.height();
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Cubemap Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        for (i, rgba) in rgba_images.iter().enumerate() {
            println!("Writing texture for image {}: width = {}, height = {}", i, rgba.width(), rgba.height());
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32, // Set Z to the index of the image
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * rgba.width()),
                    rows_per_image: std::num::NonZeroU32::new(rgba.height()),
                },
                wgpu::Extent3d {
                    width: rgba.width(),
                    height: rgba.height(),
                    depth_or_array_layers: 1, // Set depth to 1 for each image
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: std::num::NonZeroU8::new(16),
            border_color: None,
            label: Some("Texture Sampler"),
        });

        Self { texture, view, sampler }
    }
}

// Singleton for texture loading
lazy_static::lazy_static! {
    static ref TEXTURE: Mutex<Option<Arc<Texture>>> = Mutex::new(None);
}

pub fn get_texture(device: &wgpu::Device, queue: &wgpu::Queue, paths: [&str; 6]) -> Arc<Texture> {
    let mut texture = TEXTURE.lock().unwrap();
    if texture.is_none() {
        *texture = Some(Arc::new(Texture::from_images(device, queue, paths)));
    }
    texture.as_ref().unwrap().clone() // Return a clone of the Arc
}
