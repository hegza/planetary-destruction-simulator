use isosurface::source::Source;

/// The distance-field equation for a torus
pub fn torus(x: f32, y: f32, z: f32) -> f32 {
    const R1: f32 = 1.0 / 4.0;
    const R2: f32 = 1.0 / 10.0;
    let q_x = ((x * x + y * y).sqrt()).abs() - R1;
    let len = (q_x * q_x + z * z).sqrt();
    len - R2
}

pub struct Torus {}

impl Source for Torus {
    fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
        torus(x - 0.5, y - 0.5, z - 0.5)
    }
}
