mod graphics;
mod input;
mod cube;

use graphics::Graphics;
use input::Input;
use glium::glutin::event_loop::EventLoop;

// In main.rs
fn main() {
    let event_loop = EventLoop::new();
    let mut graphics = Graphics::new(&event_loop);
    let mut input = Input::new();
    let (mut yaw, mut pitch) = (0.0_f32, 0.0_f32);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = glium::glutin::event_loop::ControlFlow::Poll;

        input.process_event(&event, &mut yaw, &mut pitch);
        graphics.draw_frame(&input, &mut yaw, &mut pitch);

        if let glium::glutin::event::Event::WindowEvent { event, .. } = &event {
            if let glium::glutin::event::WindowEvent::CloseRequested = event {
                *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                return;
            }
        }
    });
}