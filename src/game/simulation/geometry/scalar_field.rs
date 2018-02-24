use isosurface::source::Source;
use prelude::*;
use cgmath::*;

/// Represents a 3D scalar field in range [0..1]
pub struct ScalarField {
    elems: Vec<f32>,
    dim: f32,
}

impl ScalarField {
    pub fn new(dim: usize, threshold: f32) -> ScalarField {
        let center = Vector3f::new(0.5f32, 0.5f32, 0.5f32);

        let mut elems: Vec<f32> = vec![0f32; dim * dim * dim];
        for z in 0..dim {
            for y in 0..dim {
                for x in 0..dim {
                    // sample = [(0, 0, 0), (1, 1, 1)]
                    let sample = Vector3f::new(
                        (x as f32 + 0.5f32) / dim as f32,
                        (y as f32 + 0.5f32) / dim as f32,
                        (z as f32 + 0.5f32) / dim as f32,
                    );
                    // dist = [0, 0.86]
                    let dist = (sample - center).magnitude2();
                    elems[dim * dim * z + dim * y + x] = dist - threshold;
                }
            }
        }

        ScalarField {
            elems,
            dim: dim as f32,
        }
    }
}

impl Source for ScalarField {
    fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
        let d: usize = self.dim as usize;
        // TODO: improve algorithm, this should likely be like 2D-texture sampling
        let z_idx = (self.dim * z) as usize;
        let y_idx = (self.dim * y) as usize;
        let x_idx = (self.dim * x) as usize;
        let idx = ((d * d * z_idx + d * y_idx + x_idx) as usize).min(d * d * d - 1);

        self.elems[idx]
    }
}
