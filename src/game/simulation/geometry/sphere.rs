use isosurface::source::Source;
use prelude::*;
use cgmath::*;

pub struct Sphere {}

impl Source for Sphere {
    fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
        let center = Vector3f::new(0.5f32, 0.5f32, 0.5f32);
        let sample = Vector3f::new(x, y, z);
        let dist = (sample - center).magnitude2();
        const SIZE: f32 = 0.2f32;
        let ret = dist - SIZE;
        ret
    }
}
