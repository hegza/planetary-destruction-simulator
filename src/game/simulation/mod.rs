mod geometry;
mod unit_cube;

use glium::*;
use cgmath::prelude::*;
use cgmath::Deg;
use std::str;
use util::camera::*;
use handle_events::*;
use index::PrimitiveType;
use util::*;
use prelude::*;
use shader;
use self::geometry::*;
use self::unit_cube::*;
use super::settings::*;

pub struct Simulation {
    program: Program,
    camera: RotationalCamera,
    cam_control: CameraControl,
    m_transform: Decomposedf,
    geom_gen: GeometryGen,
    vbo: VertexBuffer<VertexPN>,
    ibo: IndexBuffer<u32>,
    cube_vbo: VertexBuffer<VertexPN>,
    cube_ibo: IndexBuffer<u32>,
    cfg: Settings,
    ended: bool,
}

impl Simulation {
    pub fn new(cfg: Settings, display: &mut Display) -> Simulation {
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
            Deg(0f32),
            2f32,
            1024.0 / 768.0,
        );

        let m_transform = Decomposedf {
            scale: 1f32,
            rot: Quaternionf::zero(),
            disp: Vector3f::zero(),
        };

        let geom_gen = GeometryGen::new(cfg.scalar_field_dim);

        Simulation {
            program,
            cam_control,
            m_transform,
            camera,
            geom_gen,
            vbo: VertexBuffer::dynamic(display, &[]).unwrap(),
            ibo: IndexBuffer::dynamic(display, PrimitiveType::TrianglesList, &[]).unwrap(),
            cube_vbo: VertexBuffer::new(display, &UNIT_CUBE_VBO).unwrap(),
            cube_ibo: IndexBuffer::new(display, PrimitiveType::TrianglesList, &UNIT_CUBE_IBO)
                .unwrap(),
            cfg,
            ended: false,
        }
    }
    pub fn draw(&mut self, display: &mut Display) {
        let model_uni = shader::project(&self.camera, &self.m_transform);

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
        if self.cfg.render_cube {
            target
                .draw(
                    &self.cube_vbo,
                    &self.cube_ibo,
                    &self.program,
                    &model_uni,
                    &params,
                )
                .unwrap();
        }
        target
            .draw(&self.vbo, &self.ibo, &self.program, &model_uni, &params)
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
