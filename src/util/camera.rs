use prelude::*;
use cgmath::{perspective, Deg, InnerSpace, Rad, Rotation, Vector2, Zero};
use std::ops::{Deref, DerefMut};

pub struct Camera {
    position: Point3f,
    target: Point3f,
    up: Vector3f,
    yfov: Rad<f32>,
    znear: f32,
    zfar: f32,
    // Cached perspective-projection
    c_perspective: Matrix4f,
    // Cached view-projection
    c_view: Matrix4f,
}

impl Camera {
    pub fn new(init_pos: Point3f, init_target: Point3f, aspect: f32) -> Camera {
        let zfar = 1024.0;
        let znear = 0.1;
        let yfov: Rad<f32> = Rad(3.141592 / 2.0);
        let c_perspective = Camera::get_perspective(yfov, aspect, znear, zfar);

        let up = Vector3f::unit_y();
        let c_view = Camera::get_view(init_pos, init_target, up);

        Camera {
            position: init_pos,
            target: init_target,
            up,
            yfov,
            znear,
            zfar,
            c_perspective,
            c_view,
        }
    }

    pub fn position(&self) -> &Point3f {
        &self.position
    }

    pub fn target(&self) -> &Point3f {
        &self.target
    }

    pub fn set_position(&mut self, pos: Point3f) {
        self.position = pos;
    }

    pub fn set_target(&mut self, tgt: Point3f) {
        self.target = tgt;
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.c_perspective = Camera::get_perspective(self.yfov, aspect, self.znear, self.zfar)
    }

    pub fn perspective(&self) -> &Matrix4f {
        &self.c_perspective
    }

    pub fn view(&self) -> &Matrix4f {
        &self.c_view
    }

    pub fn late_update(&mut self) {
        self.c_view = Camera::get_view(self.position, self.target, self.up);
    }

    fn get_perspective(yfov: Radf, aspect: f32, znear: f32, zfar: f32) -> Matrix4f {
        perspective(yfov, aspect, znear, zfar)
    }

    fn get_view(pos: Point3f, tgt: Point3f, up: Vector3f) -> Matrix4f {
        Matrix4f::look_at(pos, tgt, up)
    }
}

const MAX_LAT: Degf = Deg(70f32);
pub struct RotationalCamera {
    inner: Camera,
    dist: f32,
    // (long, lat)
    angle: Vector2f,
    // Angular velocity
    avel: Vector2f,
}

impl RotationalCamera {
    pub fn new(
        origo: Point3f,
        init_lat: Deg<f32>,
        init_long: Deg<f32>,
        dist: f32,
        aspect: f32,
    ) -> RotationalCamera {
        let init_pos = origo + dist * -Vector3f::unit_z();
        RotationalCamera {
            inner: Camera::new(init_pos, origo, aspect),
            dist,
            angle: Vector2::new(Rad::from(init_long).0, Rad::from(init_lat).0),
            avel: Vector2::zero(),
        }
    }
    pub fn set_avel(&mut self, avel: Vector2f) {
        self.avel = avel;
    }
    pub fn update(&mut self, dt: f32) {
        self.angle += self.avel * dt;
    }
    pub fn late_update(&mut self) {
        // Clamp latitude
        if self.angle[1] > Rad::from(MAX_LAT).0 {
            self.angle[1] = Rad::from(MAX_LAT).0;
        } else if self.angle[1] < Rad::from(-MAX_LAT).0 {
            self.angle[1] = Rad::from(-MAX_LAT).0;
        }
        // Wrap longtitude
        if self.angle[0] >= PI {
            self.angle[0] -= 2f32 * PI;
        }
        if self.angle[0] <= -PI {
            self.angle[0] += 2f32 * PI;
        }

        // TODO: convert to use quaternions 100 % of the way
        let euler = Eulerf {
            x: Rad(-self.angle[1]),
            y: Rad(-self.angle[0]),
            z: Rad(0f32),
        };
        let q = Quaternionf::from(euler).conjugate();
        // Rotate around from the default position based on the current euler-angles (long, lat)
        self.position = self.target + q.rotate_vector(-Vector3f::unit_z() * self.dist);

        self.inner.late_update();
    }
}

impl<'s> Deref for RotationalCamera {
    type Target = Camera;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'s> DerefMut for RotationalCamera {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Max. rotational speed per second in radians
const ROT_SPEED: f32 = PI / 2f32;
pub struct CameraControl {
    pub cw: bool,
    pub ccw: bool,
    pub n: bool,
    pub s: bool,
}

impl CameraControl {
    /// Sets angular velocity for camera based on control state
    pub fn update_camera(&self, camera: &mut RotationalCamera) {
        let long_vel;
        match (self.cw, self.ccw) {
            (true, true) | (false, false) => {
                long_vel = 0f32;
            }
            (true, false) => {
                long_vel = -1f32;
            }
            (false, true) => {
                long_vel = 1f32;
            }
        }
        let lat_vel;
        match (self.n, self.s) {
            (true, true) | (false, false) => {
                lat_vel = 0f32;
            }
            (false, true) => {
                lat_vel = -1f32;
            }
            (true, false) => {
                lat_vel = 1f32;
            }
        }

        let mut avel = Vector2f::new(long_vel, lat_vel);
        if avel.magnitude2() >= 0.1f32 {
            avel = avel.normalize();
        }
        camera.set_avel(avel * ROT_SPEED);
    }
}

impl Default for CameraControl {
    fn default() -> CameraControl {
        CameraControl {
            cw: false,
            ccw: false,
            n: false,
            s: false,
        }
    }
}
