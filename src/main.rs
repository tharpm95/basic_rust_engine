use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

mod app;
mod camera;
mod world;
mod vertex;
mod uniforms;
mod chunk;
mod world_update;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let monitor = window.primary_monitor();
    window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(monitor)));

    // Attempt to grab the cursor and make it invisible
    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);

    pollster::block_on(app::run(event_loop, window));
}
