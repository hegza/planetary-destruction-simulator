use glutin;
use glium;

pub fn open_display(events_loop: &glutin::EventsLoop) -> glium::Display {
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    display
}
