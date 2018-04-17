mod geometry;
mod unit_cube;

use glium::*;
use glium::texture::SrgbTexture2d;
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
use image;
use image::GenericImage;

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
    planet_texture: SrgbTexture2d,
    polar_texture: SrgbTexture2d,
}

impl Simulation {
    pub fn new(cfg: Settings, display: &mut Display) -> Simulation {
        let program = program!(
            display,
            140 => {
                vertex: str::from_utf8(include_bytes!("../../shader/project.140.vert")).unwrap(),
                fragment: str::from_utf8(include_bytes!("../../shader/triplanar.140.frag")).unwrap(),
            }).unwrap();

        let cam_control = CameraControl::default();
        let window_size = display.gl_window().get_inner_size().unwrap();
        let camera = RotationalCamera::new(
            Point3f::new(0.0, 0.0, 0.0),
            Deg(0f32),
            Deg(0f32),
            2f32,
            window_size.0 as f32 / window_size.1 as f32,
        );

        let m_transform = Decomposedf {
            scale: 1f32,
            rot: Quaternionf::zero(),
            disp: Vector3f::zero(),
        };

        let geom_gen = GeometryGen::new(cfg.scalar_field_dim);

        // TODO: refactor
        // Load the texture for the triplanar mapping
        let planet_img = image::open(&cfg.planet_texture).expect(&format!(
            "cannot open texture file at {}",
            &cfg.planet_texture
        ));
        let polar_img = image::open(&cfg.polar_texture).expect(&format!(
            "cannot open texture file at {}",
            &cfg.polar_texture
        ));
        let planet_img = planet_img
            .to_rgb()
            .pixels()
            .into_iter()
            .map(|p| (p.data[0], p.data[1], p.data[2]))
            .collect::<Vec<(u8, u8, u8)>>()
            .chunks(planet_img.width() as usize)
            .map(|x| x.to_vec())
            .collect::<Vec<Vec<(u8, u8, u8)>>>();
        let polar_img = polar_img
            .to_rgb()
            .pixels()
            .into_iter()
            .map(|p| (p.data[0], p.data[1], p.data[2]))
            .collect::<Vec<(u8, u8, u8)>>()
            .chunks(polar_img.width() as usize)
            .map(|x| x.to_vec())
            .collect::<Vec<Vec<(u8, u8, u8)>>>();
        let planet_texture = SrgbTexture2d::new(display, planet_img).unwrap();
        let polar_texture = SrgbTexture2d::new(display, polar_img).unwrap();

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
            planet_texture,
            polar_texture,
        }
    }
    pub fn draw(&mut self, display: &mut Display) {
        let model_uni = shader::project_triplanar(
            &self.camera,
            &self.m_transform,
            &self.planet_texture,
            &self.polar_texture,
        );

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
    pub fn process_events(&mut self, actions: &[Action]) -> Option<ProgramCommand> {
        let cmd = process_global_events(&mut self.camera, &actions);
        process_camera_events(&mut self.cam_control, &actions);
        self.cam_control.update_camera(&mut self.camera);

        cmd
    }
    pub fn fixed_update(&mut self, dt: f32) {
        self.geom_gen.fixed_update(dt);
    }
    pub fn update(&mut self, dt: f32) {
        self.camera.update(dt);
    }
}
