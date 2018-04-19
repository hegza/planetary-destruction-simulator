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
use super::ocl_liquid_sim as cl;
use ocl;
use ndarray::prelude::*;

pub struct GeometryGen {
    marching_cubes: MarchingCubes,
    // Double buffered
    sources: [CentralDifference<ScalarField>; 2],
    flows: [Array3<f32>; 2],
    temperatures: [Array3<f32>; 2],
    frame_count: usize,
    dim: usize,
    kernel: ocl::Kernel,
    mass_dbl_buf: [ocl::Buffer<f32>; 2],
    flow_dbl_buf: [ocl::Buffer<f32>; 2],
    temperature_dbl_buf: [ocl::Buffer<f32>; 2],
    laser: bool,
    laser_strength: f32,
}

impl GeometryGen {
    /// dim: scalar field side length
    pub fn new(dim: usize, fixed_dt: f32, laser_strength: f32) -> GeometryGen {
        // p = 0.145f32 is a nice default for max-size
        const PLANET_RADIUS: f32 = 0.1f32;
        const SURFACE_UNEVENNESS: f32 = PLANET_RADIUS * 0.1f32;
        let model = ScalarField::new(dim, PLANET_RADIUS, SURFACE_UNEVENNESS);

        let cell_dist = 1f32 / (dim - 2) as f32;
        let mut source_0 = CentralDifference::new_with_epsilon(model.clone(), cell_dist);
        let mut source_1 = CentralDifference::new_with_epsilon(model, cell_dist);

        // Reserve space for float3's instead of floats
        let flow_0 = Array::default((3 * dim, 3 * dim, 3 * dim));
        let flow_1 = Array::default((3 * dim, 3 * dim, 3 * dim));

        let temperature_0 = Array::from_elem((dim, dim, dim), 0f32);
        let temperature_1 = Array::from_elem((dim, dim, dim), 0f32);

        let marching_cubes = MarchingCubes::new(dim);

        // Initialize OpenCL
        debug!("initializing OpenCL");
        // TODO: refactor these into an OpenCL-struct
        let (kernel, mass_dbl_buf, flow_dbl_buf, temperature_dbl_buf) = cl::bind(
            "src/game/simulation/cl/liquid_sim.cl",
            "simulate_liquid",
            [
                source_0.inner_mut().into_slice(),
                source_1.inner_mut().into_slice(),
            ],
            [
                flow_0.view().into_slice().unwrap(),
                flow_1.view().into_slice().unwrap(),
            ],
            [
                temperature_0.view().into_slice().unwrap(),
                temperature_1.view().into_slice().unwrap(),
            ],
            ocl::SpatialDims::Three(dim, dim, dim),
            fixed_dt,
            cell_dist,
        );
        debug!("OpenCL init success");

        GeometryGen {
            marching_cubes,
            sources: [source_0, source_1],
            flows: [flow_0, flow_1],
            temperatures: [temperature_0, temperature_1],
            frame_count: 0,
            dim: dim,
            kernel,
            mass_dbl_buf,
            flow_dbl_buf,
            temperature_dbl_buf,
            laser: false,
            laser_strength,
        }
    }

    pub fn fixed_update(&mut self, _: f32) {
        if self.laser {
            let src = &mut self.temperatures[self.frame_count % 2];
            let len = src.len_of(Axis(0));
            let pos = len / 2;
            let line = src.slice_mut(s![pos..pos + 1, pos..pos + 1, ..pos;-1]);

            for elem in line {
                *elem = 1f32;
            }
        }

        // Run the OpenCL kernel for the liquid simulation
        cl::call(
            &self.kernel,
            &self.mass_dbl_buf,
            &self.flow_dbl_buf,
            &self.temperature_dbl_buf,
            self.frame_count,
        );
        self.frame_count += 1;
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
        // Get the latest buffer
        let nbuf = &self.sources[(self.frame_count + 1) % 2];
        self.marching_cubes
            .extract_with_normals(nbuf, &mut vertices, &mut indices);

        // Offset on CPU based on the physical center of the scalar field
        let offset = 1f32 - nbuf.inner().center();
        // Re-normalize from [0, 1] to [-1, 1]
        // TODO: this would be efficient to do on the GPU => move to vertex shader
        vertices.chunks_mut(6).for_each(|chunk| {
            chunk[0] = 2f32 * (chunk[0] - offset);
            chunk[1] = 2f32 * (chunk[1] - offset);
            chunk[2] = 2f32 * (chunk[2] - offset);
        });

        *vbo = VertexBuffer::dynamic(display, util::reinterpret_cast_slice(&vertices))
            .expect("failed to create vertex buffer");
        *ibo = IndexBuffer::dynamic(display, PrimitiveType::TrianglesList, &indices)
            .expect("failed to create index buffer");
    }

    pub fn explode(&mut self, set: bool) {
        self.laser = set;
    }
}
