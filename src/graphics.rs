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
use glium::implement_vertex;

#[derive(Copy, Clone)]
struct CrosshairVertex {
    position: [f32; 2],
}

implement_vertex!(CrosshairVertex, position);

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
    highlight_color: [f32; 4],
    crosshair_vertex_buffer: VertexBuffer<CrosshairVertex>,
    crosshair_program: Program,
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

        display.gl_window().window().set_cursor_visible(false);

        let img: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(64, 64, |x, y| {
            if x < 4 || x > 59 || y < 4 || y > 59 {
                image::Rgba([77, 77, 77, 255])
            } else {
                image::Rgba([0, 0, 0, 255])
            }
        });

        let dimensions = img.dimensions();
        let raw_data = img.into_raw();
        let raw_image = RawImage2d::from_raw_rgba_reversed(&raw_data, dimensions);
        let texture = Texture2d::new(&display, raw_image).unwrap();

        let vertex_buffer = VertexBuffer::new(&display, &Cube::vertices()).unwrap();
        let index_buffer = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &Cube::indices()).unwrap();

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
            uniform vec4 u_highlight_color;
            void main() {
                color = texture(tex, v_tex_coords) * u_highlight_color;
            }
        ", None).unwrap();

        let crosshair_program = Program::from_source(&display, "
            #version 140
            in vec2 position;
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ", "
            #version 140
            out vec4 color;
            void main() {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ", None).unwrap();

        let crosshair_vertices = [
            CrosshairVertex { position: [-0.011, 0.0] }, CrosshairVertex { position: [0.0103, 0.0] },
            CrosshairVertex { position: [0.0, -0.02] }, CrosshairVertex { position: [0.0, 0.019] },
        ];

        let crosshair_vertex_buffer = VertexBuffer::new(&display, &crosshair_vertices).unwrap();

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
            highlight_color: [0.5_f32, 0.5_f32, 1.0_f32, 1.0_f32],
            crosshair_vertex_buffer,
            crosshair_program,
        }
    }

    fn find_targeted_cube(&self, direction: nalgebra::Vector3<f32>) -> Option<usize> {
        self.instance_buffer.read().unwrap().iter().enumerate().filter_map(|(i, inst)| {
            let position = nalgebra::Vector3::new(inst.instance_position[0], inst.instance_position[1], inst.instance_position[2]);
            let to_cube = position - nalgebra::Vector3::from(self.player_position);
            let distance = to_cube.magnitude();
            let to_cube_norm = to_cube.normalize();
            let dot_product = to_cube_norm.dot(&direction);

            if dot_product > 0.98 && distance < 8.0 {
                Some((i, distance))
            } else {
                None
            }
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(i, _)| i)
    }

    pub fn draw_frame(&mut self, input: &mut Input, yaw: &mut f32, pitch: &mut f32) {
        let elapsed = self.last_update.elapsed();
        self.last_update = Instant::now();

        input.process_input(elapsed, &mut self.player_position, yaw, pitch);

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
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let uniforms = uniform! {
            matrix: <[[f32; 4]; 4]>::from(self.perspective.to_homogeneous() * view),
            tex: &self.texture,
            u_highlight_color: [1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32],
        };

        target.draw(
            (&self.vertex_buffer, self.instance_buffer.per_instance().unwrap()), 
            &self.index_buffer, 
            &self.program, 
            &uniforms, 
            &glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    .. Default::default()
                },
                .. Default::default()
            }
        ).unwrap();

        // Inside the draw_frame method in graphics.rs
        if let Some(targeted_index) = self.find_targeted_cube(direction) {
            if input.can_remove_cube() {
                let instance_positions: Vec<Instance> = self.instance_buffer.read()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &inst)| if i != targeted_index { Some(inst) } else { None })
                    .collect();
                self.instance_buffer = VertexBuffer::new(&self.display, &instance_positions).unwrap();
            } else {
                let instance_slice = self.instance_buffer.slice(targeted_index..targeted_index + 1).unwrap();
                let highlight_uniforms = uniform! {
                    matrix: <[[f32; 4]; 4]>::from(self.perspective.to_homogeneous() * view),
                    tex: &self.texture,
                    u_highlight_color: self.highlight_color,
                };
                target.draw(
                    (&self.vertex_buffer, instance_slice.per_instance().unwrap()), 
                    &self.index_buffer, 
                    &self.program, 
                    &highlight_uniforms,
                    &glium::DrawParameters {
                        polygon_mode: glium::PolygonMode::Line,
                        line_width: Some(3.0),
                        .. Default::default()
                    }
                ).unwrap();
            }
        }

        target.draw(
            &self.crosshair_vertex_buffer,
            glium::index::NoIndices(glium::index::PrimitiveType::LinesList),
            &self.crosshair_program,
            &uniform! {},
            &glium::DrawParameters {
                blend: glium::draw_parameters::Blend::alpha_blending(),
                ..Default::default()
            }
        ).unwrap();

        target.finish().unwrap();
    }
}