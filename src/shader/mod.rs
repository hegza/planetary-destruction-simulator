#![allow(dead_code)]
use glium::uniforms::{EmptyUniforms, UniformsStorage};
use util::camera::Camera;
use cgmath::conv::*;
use prelude::*;

/// Creates the required uniforms for the camera-model projection shader
pub fn project<'c, 't>(
    camera: &'c Camera,
    transform: &'t Decomposedf,
) -> UniformsStorage<
    't,
    f32,
    UniformsStorage<
        't,
        [f32; 4],
        UniformsStorage<'t, [f32; 3], UniformsStorage<'c, [[f32; 4]; 4], EmptyUniforms>>,
    >,
> {
    let projection = camera.perspective() * camera.view();
    uniform! {
        vpmatrix: array4x4(projection),
        translation: array3(transform.disp),
        orientation: array4(transform.rot),
        scale: transform.scale,
    }
}
