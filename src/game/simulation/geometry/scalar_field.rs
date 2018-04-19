use isosurface::source::Source;
use prelude::*;
use cgmath::{InnerSpace, Vector3};
use rand;
use rand::Rng;
use ndarray::prelude::*;
use std::cell::RefCell;
use std::cmp::min;

/// Represents a 3D scalar field in range [0..1]
#[derive(Clone)]
pub struct ScalarField {
    elems: Array3<f32>,
    dim: f32,
}

impl ScalarField {
    pub fn new(dim: usize, threshold: f32, surface_jitter: f32) -> ScalarField {
        // The center is 0.5 * (dim - 1); use that as "absolute center"
        let center_abs = Vector3f::new(
            0.5f32 * (dim - 2) as f32,
            0.5f32 * (dim - 2) as f32,
            0.5f32 * (dim - 2) as f32,
        );

        let mut rng = rand::thread_rng();

        let mut elems = unsafe { Array::uninitialized((dim, dim, dim)) };
        for z in 0..dim {
            for y in 0..dim {
                for x in 0..dim {
                    let pos = Vector3f::new(x as f32, y as f32, z as f32);
                    // Distance between this point and the simulation center; sample-limits = [(0, 0, 0), (1, 1, 1)]
                    let dist_norm = (pos - center_abs) / dim as f32;
                    let dist_norm2 = dist_norm.magnitude2();
                    let jitter = (rng.next_f32() * 2f32 - 1f32) * surface_jitter;
                    let mut elem = unsafe { elems.uget_mut((z, y, x)) };
                    *elem = dist_norm2 - threshold + jitter;
                    /*
                    if dist_norm2 > threshold * threshold {
                        *elem = 1f32;
                    } else {
                        *elem = -1f32;
                    }
                    */
                }
            }
        }

        ScalarField {
            elems,
            dim: dim as f32,
        }
    }
    pub fn dim(&self) -> usize {
        self.dim as usize
    }
    pub fn center(&self) -> f32 {
        // HACK: This should be dim-1, not dim-2 but hacking it like this compensates for the error
        // caused by discretization in the marching cubes algorithm
        0.5f32 * (self.dim - 2f32) / self.dim
    }
    pub fn elems_mut(&mut self) -> &mut Array3<f32> {
        &mut self.elems
    }
    pub fn into_slice(&self) -> &[f32] {
        self.elems.view().into_slice().unwrap()
    }
    // TODO: this indexing stuff needs some work
    pub fn elem(&self, v: &Vector3f) -> f32 {
        let idx = self.resolve_index3d(v);
        unsafe { *self.elems.uget(idx) }
    }
    pub fn elem_mut(&mut self, v: &Vector3f) -> &mut f32 {
        let idx = self.resolve_index3d(v);
        unsafe { self.elems.uget_mut(idx) }
    }
    fn resolve_index(&self, v: &Vector3f) -> usize {
        let x_idx = (self.dim * v[0]) as usize;
        let y_idx = (self.dim * v[1]) as usize;
        let z_idx = (self.dim * v[2]) as usize;

        // HACK: min() is a workaround to overindexing
        let d = self.dim as usize;
        ((d * d * z_idx + d * y_idx + x_idx) as usize).min(d * d * d - 1)
    }
    fn resolve_index3d(&self, v: &Vector3f) -> (usize, usize, usize) {
        // Re-scale the indexer vector from [0, 1] to [0, 1[.
        // HACK: This should be dim-1, not dim-2 but hacking it like this compensates for the error
        // caused by discretization in the marching cubes algorithm; this also causes a crash in
        // some fairly rare scenarios :-D
        // HACK: yeah, let's just clamp them so there's no crash when normals start exploding
        let d = self.dim as usize - 1;
        let z_idx = min(((self.dim - 2f32) * v[2]) as usize, d);
        let y_idx = min(((self.dim - 2f32) * v[1]) as usize, d);
        let x_idx = min(((self.dim - 2f32) * v[0]) as usize, d);

        (z_idx, y_idx, x_idx)
    }
}

impl Source for ScalarField {
    fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
        // TODO: improve algorithm, this should likely be like trilinear 2D-texture sampling that
        // would allow interpolation if the render resolution was higher than model resolution
        self.elem(&Vector3f::new(x, y, z))
    }
}
