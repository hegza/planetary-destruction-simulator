#![allow(dead_code)]
use glium::uniforms::{EmptyUniforms, Sampler, UniformsStorage};
use glium::texture::SrgbTexture2d;
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

/// Creates the required uniforms for the projection + triplanar mapping
pub fn project_triplanar<'c, 'tfm, 'tex1, 'tex2>(
    camera: &'c Camera,
    transform: &'tfm Decomposedf,
    horizontal_texture: &'tex1 SrgbTexture2d,
    vertical_texture: &'tex2 SrgbTexture2d,
) -> UniformsStorage<
    'tex2,
    Sampler<'tex2, SrgbTexture2d>,
    UniformsStorage<
        'tex1,
        Sampler<'tex1, SrgbTexture2d>,
        UniformsStorage<
            'tfm,
            f32,
            UniformsStorage<
                'tfm,
                [f32; 4],
                UniformsStorage<'tfm, [f32; 3], UniformsStorage<'c, [[f32; 4]; 4], EmptyUniforms>>,
            >,
        >,
    >,
> {
    let projection = camera.perspective() * camera.view();
    uniform! {
        vpmatrix: array4x4(projection),
        translation: array3(transform.disp),
        orientation: array4(transform.rot),
        scale: transform.scale,
        t_horizontal: horizontal_texture.sampled(),
        t_vertical: vertical_texture.sampled()
    }
}
