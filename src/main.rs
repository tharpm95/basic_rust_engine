use glium::{Display, Surface};
use glium::glutin::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, ContextBuilder, GlProfile, GlRequest, window::WindowBuilder};

fn main() {
    // Create a window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Genero");
    let context = ContextBuilder::new().with_gl(GlRequest::Latest).with_gl_profile(GlProfile::Core);
    let display = Display::new(window, context, &event_loop).unwrap();

    // Main game loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            _ => ()
        }

        // Clear the screen
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        // Finish drawing
        target.finish().unwrap();
    });
}