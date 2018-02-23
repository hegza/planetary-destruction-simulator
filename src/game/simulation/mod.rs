mod torus;

use glium::*;
use prelude::*;
use cgmath::prelude::*;
use cgmath::Deg;
use std::str;
use util::camera::*;
use handle_events::*;
use isosurface::source::*;
use isosurface::marching_cubes::MarchingCubes;
use index::PrimitiveType;
use util;
use util::*;
use shader;
use self::torus::*;

pub struct Simulation {
    program: Program,
    camera: RotationalCamera,
    cam_control: CameraControl,
    m_transform: Decomposedf,
    geom_gen: GeometryGen,
    vbo: VertexBuffer<VertexPN>,
    ibo: IndexBuffer<u32>,
    ended: bool,
}

impl Simulation {
    pub fn new(display: &mut Display) -> Simulation {
        let program = program!(display,
        140 => {
            vertex: str::from_utf8(include_bytes!("../../shader/project.140.vert")).unwrap(),
            fragment: str::from_utf8(include_bytes!("../../shader/illuminate.140.frag")).unwrap(),
        },
    ).unwrap();

        let cam_control = CameraControl::default();
        let camera = RotationalCamera::new(
            Point3f::new(0.0, 0.0, 0.0),
            Deg(0f32),
            Deg(90f32),
            3f32,
            1024.0 / 768.0,
        );

        let m_transform = Decomposedf {
            scale: 1f32,
            rot: Quaternionf::zero(),
            disp: Vector3f::zero(),
        };

        let geom_gen = GeometryGen::new(8);

        Simulation {
            program,
            cam_control,
            m_transform,
            camera,
            geom_gen,
            vbo: VertexBuffer::dynamic(display, &[]).unwrap(),
            ibo: IndexBuffer::dynamic(display, PrimitiveType::TrianglesList, &[]).unwrap(),
            ended: false,
        }
    }
    pub fn draw(&mut self, display: &mut Display) {
        let uniforms = shader::project(&self.camera, &self.m_transform);

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
            .draw(&self.vbo, &self.ibo, &self.program, &uniforms, &params)
            .unwrap();
        target.finish().unwrap();
    }
    pub fn late_update(&mut self, display: &mut Display) {
        self.camera.late_update();
        self.geom_gen
            .update_vbo(&mut self.vbo, &mut self.ibo, display);
    }
    pub fn process_events(&mut self, actions: &[Action]) -> bool {
        let keep_running = process_global_events(&mut self.camera, &actions);
        process_camera_events(&mut self.cam_control, &actions);
        self.cam_control.update_camera(&mut self.camera);

        keep_running
    }
    pub fn fixed_update(&mut self, dt: f32) {
        self.geom_gen.fixed_update(dt);
    }
    pub fn update(&mut self, dt: f32) {
        self.camera.update(dt);
    }
    pub fn ended(&self) -> bool {
        self.ended
    }
}

struct GeometryGen {
    marching_cubes: MarchingCubes,
    central_difference: CentralDifference<Torus>,
}

impl GeometryGen {
    pub fn new(scalar_field_side: usize) -> GeometryGen {
        let torus = Torus { counter: 0f32 };
        let central_difference = CentralDifference::new(torus);
        let marching_cubes = MarchingCubes::new(scalar_field_side);
        GeometryGen {
            marching_cubes,
            central_difference,
        }
    }

    pub fn fixed_update(&mut self, dt: f32) {
        self.central_difference.source_mut().counter += dt * 0.05f32;
    }

    pub fn update_vbo(
        &mut self,
        vbo: &mut VertexBuffer<VertexPN>,
        ibo: &mut IndexBuffer<u32>,
        display: &Display,
    ) {
        // Note: the n:o vertices/indices changes over time.
        let mut vertices = vec![];
        let mut indices = vec![];
        self.marching_cubes.extract_with_normals(
            &self.central_difference,
            &mut vertices,
            &mut indices,
        );
        // Re-normalize from [0, 1] to [-1, 1]
        vertices.chunks_mut(6).for_each(|chunk| {
            chunk[0] = chunk[0] * 2f32 - 1f32;
            chunk[1] = chunk[1] * 2f32 - 1f32;
            chunk[2] = chunk[2] * 2f32 - 1f32;
        });
        *vbo = VertexBuffer::dynamic(display, util::reinterpret_cast_slice(&vertices))
            .expect("failed to create vertex buffer");
        *ibo = IndexBuffer::dynamic(display, PrimitiveType::TrianglesList, &indices)
            .expect("failed to create index buffer");
    }
}
