use glium::{Display, Rect};
use glutin::{ContextBuilder, EventsLoop, WindowBuilder};

pub fn open_display(title: &str, viewport: Rect, events_loop: &EventsLoop) -> Display {
    let window = WindowBuilder::new()
        .with_dimensions(viewport.width, viewport.height)
        //.with_fullscreen(Some(events_loop.get_primary_monitor()))
        .with_title(title);
    let context = ContextBuilder::new()
        .with_vsync(true)
        .with_depth_buffer(24)
        .with_pixel_format(24, 8)
        .with_srgb(true);
    let display = Display::new(window, context, &events_loop).unwrap();

    display
}
