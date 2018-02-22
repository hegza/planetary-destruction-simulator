use cgmath::{perspective, Rad, Vector3};
use prelude::*;

pub struct Camera {
    aspect_ratio: f32,
    position: Point3f,
    direction: Vector3f,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            aspect_ratio: 1024.0 / 768.0,
            position: Point3f::new(0.1, 0.1, 1.0),
            direction: Vector3::new(0.0, 0.0, -1.0),
        }
    }

    pub fn position(&self) -> &Point3f {
        &self.position
    }

    pub fn set_position(&mut self, pos: Point3f) {
        self.position = pos;
    }

    pub fn set_direction(&mut self, dir: Vector3f) {
        self.direction = dir;
    }

    pub fn get_perspective(&self) -> Matrix4f {
        let zfar = 1024.0;
        let znear = 0.1;

        let fov: Rad<f32> = Rad(3.141592 / 2.0);
        perspective(fov, self.aspect_ratio, znear, zfar)
    }

    pub fn get_view(&self) -> Matrix4f {
        Matrix4f::look_at(
            self.position,
            self.position + self.direction,
            // TODO: self.up or quaternion
            Vector3f::unit_y(),
        )
    }
}
