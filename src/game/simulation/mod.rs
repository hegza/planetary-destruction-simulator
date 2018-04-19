mod geometry;
mod unit_cube;
mod ocl_liquid_sim;

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
    pub fn new(fixed_dt: f32, cfg: Settings, display: &mut Display) -> Simulation {
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
            1.5f32,
            window_size.0 as f32 / window_size.1 as f32,
        );

        let geom_gen = GeometryGen::new(cfg.scalar_field_dim, fixed_dt, cfg.laser_strength);

        let m_transform = Decomposedf {
            scale: 1f32,
            rot: Quaternionf::zero(),
            disp: Vector3f::zero(),
        };

        // Load the textures for the triplanar mapping
        let planet_texture = load_texture(&cfg.planet_texture, display);
        let polar_texture = load_texture(&cfg.polar_texture, display);

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
        // Create the uniforms for triplanar mapping + perspective projection for the planet
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
        self.process_actions(actions);

        cmd
    }
    pub fn fixed_update(&mut self, dt: f32) {
        self.geom_gen.fixed_update(dt);
    }
    pub fn update(&mut self, dt: f32) {
        self.camera.update(dt);
    }
    fn process_actions(&mut self, actions: &[Action]) {
        actions.iter().for_each(|action| {
            use self::Action::*;
            match *action {
                Shoot(set) => self.geom_gen.explode(set),
                _ => {}
            }
        });
    }
}

/// Opens the image file based on the file type and loads it's contents into a glium-compatible SrgbTexture2d.
fn load_texture(filename: &str, display: &Display) -> SrgbTexture2d {
    let img = image::open(filename).expect(&format!("cannot open texture file at {}", filename));
    let data = img.to_rgb()
        .pixels()
        .into_iter()
        .map(|p| (p.data[0], p.data[1], p.data[2]))
        .collect::<Vec<(u8, u8, u8)>>()
        .chunks(img.width() as usize)
        .map(|x| x.to_vec())
        .collect::<Vec<Vec<(u8, u8, u8)>>>();
    SrgbTexture2d::new(display, data).expect(&format!(
        "unable to create texture from file at {}",
        filename
    ))
}
