extern crate cgmath;
extern crate genmesh;
#[macro_use]
extern crate glium;
extern crate glutin;
extern crate isosurface;
extern crate obj;

mod init;
mod handle_events;
mod prelude;
mod shader;
mod util;

use std::thread;
use std::time::{Duration, Instant};
use prelude::*;
use cgmath::prelude::*;
use cgmath::Deg;
use std::str;
use util::camera::*;
use handle_events::*;
use glium::*;
use isosurface::source::{CentralDifference, Source};
use isosurface::marching_cubes::MarchingCubes;
use index::PrimitiveType;

/// The distance-field equation for a torus
fn torus(x: f32, y: f32, z: f32) -> f32 {
    const R1: f32 = 1.0 / 4.0;
    const R2: f32 = 1.0 / 10.0;
    let q_x = ((x * x + y * y).sqrt()).abs() - R1;
    let len = (q_x * q_x + z * z).sqrt();
    len - R2
}

pub struct Torus {
    pub counter: f32,
}

impl Source for Torus {
    fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
        torus(x - 0.5, y - 0.5, z - 0.5 + self.counter)
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let viewport = Rect {
        left: 0,
        bottom: 0,
        width: 1024,
        height: 768,
    };
    let display = init::open_display("Planetary destruction simulator", viewport, &events_loop);

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
        2f32,
        1024.0 / 768.0,
    );

    let mut accumulator = Duration::new(0, 0);
    let mut last_frame_time = Instant::now();

    let m_transform = Decomposedf {
        scale: 1f32, //0.005f32,
        rot: Quaternionf::zero(),
        disp: Vector3f::zero(),
    };

    #[derive(Copy, Clone)]
    #[repr(C)]
    struct Vertex {
        position: [f32; 3],
        normal: [f32; 3],
    }
    implement_vertex!(Vertex, position, normal);

    let torus = Torus { counter: 0f32 };
    let mut central_difference = CentralDifference::new(torus);
    let mc_size = 8;
    let mut marching_cubes = MarchingCubes::new(mc_size);

    let mut keep_running = true;
    while keep_running {
        let (vbo, ibo) = {
            let mut vertices = vec![];
            let mut indices = vec![];
            marching_cubes.extract_with_normals(&central_difference, &mut vertices, &mut indices);
            // Re-normalize from [0, 1] to [-1, 1]
            vertices.chunks_mut(6).for_each(|chunk| {
                chunk[0] = chunk[0] * 2f32 - 1f32;
                chunk[1] = chunk[1] * 2f32 - 1f32;
                chunk[2] = chunk[2] * 2f32 - 1f32;
            });
            let vbo: VertexBuffer<Vertex> = {
                VertexBuffer::dynamic(&display, util::reinterpret_cast_slice(&vertices))
                    .expect("failed to create vertex buffer")
            };
            let ibo: IndexBuffer<u32> =
                IndexBuffer::dynamic(&display, PrimitiveType::TrianglesList, &indices)
                    .expect("failed to create index buffer");

            (vbo, ibo)
        };
        {
            let uniforms = shader::project(&camera, &m_transform);

            // Draw parameters
            let params = DrawParameters {
                depth: Depth {
                    test: DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                backface_culling: draw_parameters::BackfaceCullingMode::CullCounterClockwise,
                ..Default::default()
            };

            // Draw frame
            let mut target = display.draw();
            target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
            target
                .draw(&vbo, &ibo, &program, &uniforms, &params)
                .unwrap();
            target.finish().unwrap();
        }

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
            central_difference.source_mut().counter += dt * 0.05f32;
        }
        camera.late_update();

        thread::sleep(fixed_deltatime - accumulator);
    }
}
