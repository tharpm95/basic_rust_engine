use glium::{Display, Surface, Program, implement_vertex, uniform};
use glium::glutin::{event::{Event, WindowEvent, VirtualKeyCode, ElementState}, event_loop::{ControlFlow, EventLoop}, ContextBuilder, GlProfile, GlRequest, window::WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Mutetra");
    let context = ContextBuilder::new().with_gl(GlRequest::Latest).with_gl_profile(GlProfile::Core);
    let display = Display::new(window, context, &event_loop).unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let vertex1 = Vertex { position: [-0.05, -0.05] };
    let vertex2 = Vertex { position: [-0.05,  0.05] };
    let vertex3 = Vertex { position: [ 0.05,  0.05] };
    let vertex4 = Vertex { position: [ 0.05, -0.05] };

    let shape = vec![vertex1, vertex2, vertex3, vertex1, vertex3, vertex4];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        uniform vec2 translation;
        void main() {
            gl_Position = vec4(position + translation, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        out vec4 color;
        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    "#;

    let program = Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut player_x = 0.0;
    let mut player_y = 0.0;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            match key {
                                VirtualKeyCode::W => player_y += 0.1,
                                VirtualKeyCode::S => player_y -= 0.1,
                                VirtualKeyCode::A => player_x -= 0.1,
                                VirtualKeyCode::D => player_x += 0.1,
                                _ => (),
                            }
                        }
                    }
                },
                _ => (),
            },
            _ => (),
        }
        
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let uniforms = uniform! {
            translation: [player_x as f32, player_y as f32], // Ensure type conversion to f32
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    });
}