use glium::{Display, Program, Surface, VertexBuffer, IndexBuffer, texture::Texture2d, uniform};
use glium::glutin::{ContextBuilder, GlProfile, GlRequest, window::WindowBuilder};
use glium::index::PrimitiveType;
use glium::texture::RawImage2d;
use crate::cube::{Cube, Vertex, Instance};
use crate::input::Input;
use nalgebra::Perspective3;
use std::time::Instant;
use glium::glutin::window::Fullscreen;
use image::ImageBuffer;

pub struct Graphics {
    display: Display,
    program: Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    instance_buffer: VertexBuffer<Instance>,
    texture: Texture2d,
    player_position: [f32; 3],
    perspective: Perspective3<f32>,
    last_update: Instant,
}

impl Graphics {
    pub fn new(event_loop: &glium::glutin::event_loop::EventLoop<()>) -> Self {
        let primary_monitor = event_loop.primary_monitor().unwrap();
        let window = WindowBuilder::new()
            .with_title("Cube Renderer")
            .with_fullscreen(Some(Fullscreen::Borderless(Some(primary_monitor))));

        let context = ContextBuilder::new()
            .with_gl(GlRequest::Latest)
            .with_gl_profile(GlProfile::Core);

        let display = Display::new(window, context, event_loop).unwrap();

        // Creating a black square texture with a dark grey border
        let img: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(64, 64, |x, y| {
            if x < 4 || x > 59 || y < 4 || y > 59 {
                image::Rgba([77, 77, 77, 255]) // Dark grey border
            } else {
                image::Rgba([0, 0, 0, 255]) // Black
            }
        });

        let dimensions = img.dimensions(); // Get dimensions before moving
        let raw_data = img.into_raw(); // This moves img
        let raw_image = RawImage2d::from_raw_rgba_reversed(&raw_data, dimensions);
        let texture = Texture2d::new(&display, raw_image).unwrap();

        // Load vertices and indices for a single cube
        let vertex_buffer = VertexBuffer::new(&display, &Cube::vertices()).unwrap();
        let index_buffer = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &Cube::indices()).unwrap();

        // Set up cube positions
        let mut cubes = Vec::new();
        for x in 0..10 {
            for z in 0..10 {
                cubes.push(Cube { position: [x as f32, 0.0, z as f32] });
            }
        }
        for x in 0..10 {
            cubes.push(Cube { position: [x as f32, 0.0, 0.0] });
            cubes.push(Cube { position: [x as f32, 0.0, 9.0] });
        }
        for z in 0..10 {
            cubes.push(Cube { position: [0.0, 0.0, z as f32] });
            cubes.push(Cube { position: [9.0, 0.0, z as f32] });
        }

        // Create an instance buffer for cube positions
        let instance_positions: Vec<Instance> = cubes.iter().map(|c| Instance { instance_position: c.position }).collect();
        let instance_buffer = VertexBuffer::new(&display, &instance_positions).unwrap();

        let program = Program::from_source(&display, "
            #version 140
            in vec3 position;
            in vec2 tex_coords;
            in vec3 instance_position;
            out vec2 v_tex_coords;
            uniform mat4 matrix;
            void main() {
                v_tex_coords = tex_coords;
                gl_Position = matrix * vec4(position + instance_position, 1.0);
            }
        ", "
            #version 140
            in vec2 v_tex_coords;
            out vec4 color;
            uniform sampler2D tex;
            void main() {
                color = texture(tex, v_tex_coords);
            }
        ", None).unwrap();

        Graphics {
            display,
            program,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            texture,
            player_position: [0.0, 0.0, -2.0_f32],
            perspective: Perspective3::new(1920.0 / 1080.0, std::f32::consts::FRAC_PI_2, 0.1, 1024.0),
            last_update: Instant::now(),
        }
    }

    // Inside graphics.rs
    pub fn draw_frame(&mut self, input: &Input, yaw: &mut f32, pitch: &mut f32) {
        let elapsed = self.last_update.elapsed();
        self.last_update = Instant::now();

        input.process_input(elapsed, &mut self.player_position, yaw, pitch);

        // Camera direction computation with updated yaw and pitch
        let direction = nalgebra::Vector3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos()
        ).normalize();

        let view = nalgebra::Matrix4::look_at_rh(
            &nalgebra::Point3::new(
                self.player_position[0],
                self.player_position[1],
                self.player_position[2]
            ),
            &(nalgebra::Point3::from(nalgebra::Vector3::from(self.player_position) + direction)),
            &nalgebra::Vector3::y_axis(),
        );

        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let uniforms = uniform! {
            matrix: <[[f32; 4]; 4]>::from(self.perspective.to_homogeneous() * view),
            tex: &self.texture
        };

        target.draw(
            (&self.vertex_buffer, self.instance_buffer.per_instance().unwrap()), 
            &self.index_buffer, 
            &self.program, 
            &uniforms, 
            &Default::default()
        ).unwrap();

        target.finish().unwrap();
    }
}