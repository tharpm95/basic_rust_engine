use glium::{Display, Program, Surface, implement_vertex, uniform, VertexBuffer, IndexBuffer};
use glium::index::PrimitiveType;
use glium::glutin::{event::{Event, WindowEvent, VirtualKeyCode, ElementState}, 
event_loop::{ControlFlow, EventLoop}, ContextBuilder, GlProfile, GlRequest, window::WindowBuilder};
use nalgebra::{Matrix4, Perspective3};
use glium::glutin::event::DeviceEvent;
use std::collections::HashSet;
use std::time::Instant;
use glium::glutin::window::Fullscreen;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
}

implement_vertex!(Vertex, position);

#[derive(Copy, Clone)]
struct Instance {
    instance_position: [f32; 3],
}

implement_vertex!(Instance, instance_position);

struct Cube {
    position: [f32; 3],
}

impl Cube {
    fn vertices() -> Vec<Vertex> {
        vec![
            Vertex { position: [-0.5, -0.5, -0.5] },
            Vertex { position: [0.5, -0.5, -0.5] },
            Vertex { position: [0.5, 0.5, -0.5] },
            Vertex { position: [-0.5, 0.5, -0.5] },
            Vertex { position: [-0.5, -0.5, 0.5] },
            Vertex { position: [0.5, -0.5, 0.5] },
            Vertex { position: [0.5, 0.5, 0.5] },
            Vertex { position: [-0.5, 0.5, 0.5] }
        ]
    }

    fn indices() -> Vec<u16> {
        vec![
            0, 1, 2, 3,    // Front face
            7, 6, 5, 4,    // Back face
            0, 4, 5, 1,    // Bottom face
            3, 2, 6, 7,    // Top face
            1, 5, 6, 2,    // Right face
            4, 0, 3, 7     // Left face
        ]
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let primary_monitor = event_loop.primary_monitor().unwrap();
    let window = WindowBuilder::new()
        .with_title("Mutetra")
        .with_fullscreen(Some(Fullscreen::Borderless(Some(primary_monitor))));

    let context = ContextBuilder::new()
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Core);

    let display = Display::new(window, context, &event_loop).unwrap();

    let mut cubes = Vec::new();

    // Set up cube positions
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

    // Load vertices and indices for a single cube
    let vertex_buffer = VertexBuffer::new(&display, &Cube::vertices()).unwrap();
    let index_buffer = IndexBuffer::new(&display, PrimitiveType::LineLoop, &Cube::indices()).unwrap();

    // Create an instance buffer for cube positions
    let instance_positions: Vec<Instance> = cubes.iter().map(|c| Instance { instance_position: c.position }).collect();
    let instance_buffer = VertexBuffer::new(&display, &instance_positions).unwrap();

    let program = Program::from_source(&display, "
        #version 140
        in vec3 position;
        in vec3 instance_position;
        uniform mat4 matrix;
        void main() {
            gl_Position = matrix * vec4(position + instance_position, 1.0);
        }
    ", "
        #version 140
        out vec4 color;
        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    ", None).unwrap();

    let mut player_position = [0.0, 0.0, -2.0_f32];
    let perspective = Perspective3::new(4.0 / 3.0, std::f32::consts::FRAC_PI_2, 0.1, 1024.0);

    let mut pitch: f32 = 0.0;
    let mut yaw: f32 = 0.0;

    let mut pressed_keys = HashSet::new();
    let mut last_update = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let elapsed = last_update.elapsed();
        last_update = Instant::now();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Focused(true) => {
                    let gl_window = display.gl_window();
                    let window = gl_window.window();
                    window.set_cursor_grab(true).unwrap_or_else(|_| println!("Unable to grab cursor"));
                    window.set_cursor_visible(false);
                }
                WindowEvent::Focused(false) => {
                    let gl_window = display.gl_window();
                    let window = gl_window.window();
                    window.set_cursor_grab(false).unwrap_or_else(|_| println!("Unable to release cursor"));
                    window.set_cursor_visible(true);
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        match input.state {
                            ElementState::Pressed => { pressed_keys.insert(key); },
                            ElementState::Released => { pressed_keys.remove(&key); },
                        }
                    }
                },
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    let (delta_x, delta_y) = delta;
                    yaw += delta_x as f32 * 0.005;
                    pitch -= delta_y as f32 * 0.005;
                    pitch = pitch.clamp(-1.57, 1.57);
                },
                _ => (),
            },
            _ => (),
        }

        let move_speed = 2.0;
        let move_distance = move_speed * elapsed.as_secs_f32();

        let mut forward = nalgebra::Vector3::new(yaw.sin(), 0.0, -yaw.cos());
        forward.normalize_mut();

        let mut right = nalgebra::Vector3::new(forward.z, 0.0, -forward.x);
        right.normalize_mut();

        if pressed_keys.contains(&VirtualKeyCode::A) {
            player_position[0] += forward.x * move_distance;
            player_position[2] += forward.z * move_distance;
        }
        if pressed_keys.contains(&VirtualKeyCode::D) {
            player_position[0] -= forward.x * move_distance;
            player_position[2] -= forward.z * move_distance;
        }
        if pressed_keys.contains(&VirtualKeyCode::W) {
            player_position[0] -= right.x * move_distance;
            player_position[2] -= right.z * move_distance;
        }
        if pressed_keys.contains(&VirtualKeyCode::S) {
            player_position[0] += right.x * move_distance;
            player_position[2] += right.z * move_distance;
        }

        let direction = nalgebra::Vector3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos()
        ).normalize();

        let view = Matrix4::look_at_rh(
            &nalgebra::Point3::new(player_position[0], player_position[1], player_position[2]),
            &(nalgebra::Point3::from(nalgebra::Vector3::from(player_position) + direction)),
            &nalgebra::Vector3::y_axis(),
        );

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let uniforms = uniform! {
            matrix: <[[f32; 4]; 4]>::from(perspective.to_homogeneous() * view),
        };

        target.draw((&vertex_buffer, instance_buffer.per_instance().unwrap()), &index_buffer, &program, &uniforms, &Default::default()).unwrap();

        target.finish().unwrap();
    });
}