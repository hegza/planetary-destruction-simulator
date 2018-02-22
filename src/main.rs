extern crate cgmath;
extern crate genmesh;
#[macro_use]
extern crate glium;
extern crate glutin;
extern crate obj;

mod init;
mod handle_events;
mod prelude;
mod shader;
mod util;

use glium::{index, Surface};
use std::thread;
use std::time::{Duration, Instant};
use prelude::*;
use cgmath::prelude::*;
use cgmath::conv::*;
use cgmath::Deg;
use std::str;
use util::camera::*;
use handle_events::*;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let viewport = glium::Rect {
        left: 0,
        bottom: 0,
        width: 1024,
        height: 768,
    };
    let display = init::open_display("Planetary destruction simulator", viewport, &events_loop);

    // Build VBO and IBO for the teapot
    let vertex_buffer =
        util::load_wavefront(&display, util::read_file("data/mesh/teapot.obj").as_bytes());

    let program = program!(&display,
        140 => {
            vertex: str::from_utf8(include_bytes!("shader/project.140.vert")).unwrap(),
            fragment: str::from_utf8(include_bytes!("shader/illuminate.140.frag")).unwrap(),
        },
    ).unwrap();

    let mut camera_control = CameraControl::default();
    let mut camera = RotationalCamera::new(
        Point3f::new(0.0, 0.0, 0.0),
        Deg(0f32),
        Deg(0f32),
        1f32,
        1024.0 / 768.0,
    );

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
        let c_projection = camera.perspective() * camera.view();
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

        let user_actions = poll_events(&mut events_loop);
        if !process_global_events(&mut camera, &user_actions) {
            keep_running = false
        }
        process_camera_events(&mut camera_control, &user_actions);
        camera_control.update_camera(&mut camera);

        let now = Instant::now();
        accumulator += now - last_frame_time;
        last_frame_time = now;

        let fixed_deltatime = Duration::new(0, 16666667);
        while accumulator >= fixed_deltatime {
            accumulator -= fixed_deltatime;

            let dt = (fixed_deltatime.as_secs() as f64
                + fixed_deltatime.subsec_nanos() as f64 * 1e-9) as f32;
            camera.update(dt);
        }
        camera.late_update();

        thread::sleep(fixed_deltatime - accumulator);
    }
}
