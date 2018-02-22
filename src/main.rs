extern crate cgmath;
extern crate genmesh;
#[macro_use]
extern crate glium;
extern crate glutin;
extern crate obj;

mod init;
mod prelude;
mod shader;
mod util;

use glium::{index, Surface};
use std::thread;
use std::time::{Duration, Instant};
use prelude::*;
use cgmath::prelude::*;
use cgmath::conv::*;
use std::str;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let display = init::open_display(&events_loop);

    // Build VBO and IBO for the teapot
    let vertex_buffer =
        util::load_wavefront(&display, util::read_file("data/mesh/teapot.obj").as_bytes());

    let program = program!(&display,
        140 => {
            vertex: str::from_utf8(include_bytes!("shader/project.140.vert")).unwrap(),
            fragment: str::from_utf8(include_bytes!("shader/illuminate.140.frag")).unwrap(),
        },
    ).unwrap();

    let camera = util::camera::Camera::new();

    let mut accumulator = Duration::new(0, 0);
    let mut last_frame_time = Instant::now();

    let m_transform = Decomposedf {
        scale: 0.005f32,
        rot: Quaternionf::zero(),
        disp: Vector3f::zero(),
    };

    let mut keep_running = true;
    while keep_running {
        // Build uniforms
        let c_projection = camera.get_perspective() * camera.get_view();
        let uniforms = uniform! {
            vpmatrix: array4x4(c_projection),
            translation: array3(m_transform.disp),
            orientation: array4(m_transform.rot),
            scale: m_transform.scale,
        };

        // Draw parameters
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        // Draw frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target
            .draw(
                &vertex_buffer,
                &index::NoIndices(index::PrimitiveType::TrianglesList),
                &program,
                &uniforms,
                &params,
            )
            .unwrap();
        target.finish().unwrap();

        if !handle_events(&mut events_loop) {
            keep_running = false;
        }

        let now = Instant::now();
        accumulator += now - last_frame_time;
        last_frame_time = now;

        let fixed_deltatime = Duration::new(0, 16666667);
        while accumulator >= fixed_deltatime {
            accumulator -= fixed_deltatime;

            // TODO: update()
        }
        // TODO: late_update()

        thread::sleep(fixed_deltatime - accumulator);
    }
}

fn handle_events(events_loop: &mut glutin::EventsLoop) -> bool {
    let mut ret = true;
    events_loop.poll_events(|event| match event {
        glutin::Event::WindowEvent { event, .. } => match event {
            glutin::WindowEvent::Closed => {
                ret = false;
            }
            _ => {}
        },
        _ => {}
    });
    ret
}
