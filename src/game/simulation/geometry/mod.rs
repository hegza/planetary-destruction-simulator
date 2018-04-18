#![allow(unused_imports)]
#![allow(dead_code)]
mod torus;
mod scalar_field;
mod sphere;

use glium::*;
use glium::index::*;
use isosurface::source::*;
use isosurface::marching_cubes::*;
use util;
use util::*;
use prelude::*;
use self::torus::*;
use self::scalar_field::*;
use self::sphere::*;
use super::ocl_util as cl;
use ocl;

pub struct GeometryGen {
    marching_cubes: MarchingCubes,
    source: CentralDifference<ScalarField>,
    acc: f32,
    dim: usize,
    kernel: ocl::Kernel,
    buffer: ocl::Buffer<f32>,
}

impl GeometryGen {
    pub fn new(scalar_field_side: usize, fixed_dt: f32) -> GeometryGen {
        // 0.145f32 is a nice default
        let model = ScalarField::new(scalar_field_side, 0.115f32);
        let mut source =
            CentralDifference::new_with_epsilon(model, 1f32 / scalar_field_side as f32);
        let marching_cubes = MarchingCubes::new(scalar_field_side);

        // Initialize OpenCL
        debug!("initializing OpenCL");
        let (kernel, buffer) = cl::bind(
            "src/game/simulation/cl/liquid_sim.cl",
            "add",
            source.inner_mut().into_slice(),
            ocl::SpatialDims::Three(scalar_field_side, scalar_field_side, scalar_field_side),
            fixed_dt,
        ).expect("unable to initialize OpenCL");

        GeometryGen {
            marching_cubes,
            source,
            acc: 0f32,
            dim: scalar_field_side,
            kernel,
            buffer,
        }
    }

    pub fn fixed_update(&mut self, dt: f32) {
        // Run the OpenCL kernel
        cl::call(&self.kernel, &self.buffer).unwrap();
        self.acc += dt;

        /*
        const MIN: f32 = 0.04f32;
        const MAX: f32 = 0.25f32;
        const PERIOD: f32 = 1.5f32;
        // s -> [0, 1]
        let s = (f32::sin(self.acc * PI * 2f32 * (1f32 / PERIOD)) + 1f32) * 0.5f32;
        // p -> [MIN, MAX]
        let p = s * (MAX - MIN) + MIN;
        *self.source.inner_mut() = ScalarField::new(self.dim, p);
        */
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
        self.marching_cubes
            .extract_with_normals(&self.source, &mut vertices, &mut indices);
        // HACK: offset on CPU based on the physical center of the scalar field
        let offset = 1f32 - self.source.inner().center();
        // Re-normalize from [0, 1] to [-1, 1]
        // TODO: this shouldn't be done on the CPU probably, could do it in vertex shader
        vertices.chunks_mut(6).for_each(|chunk| {
            chunk[0] = chunk[0] * 2f32 - (2f32 * offset);
            chunk[1] = chunk[1] * 2f32 - (2f32 * offset);
            chunk[2] = chunk[2] * 2f32 - (2f32 * offset);
        });
        *vbo = VertexBuffer::dynamic(display, util::reinterpret_cast_slice(&vertices))
            .expect("failed to create vertex buffer");
        *ibo = IndexBuffer::dynamic(display, PrimitiveType::TrianglesList, &indices)
            .expect("failed to create index buffer");
    }
}
