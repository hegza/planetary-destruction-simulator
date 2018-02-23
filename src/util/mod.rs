#![allow(dead_code)]

pub mod camera;

use std::fs::File;
use std::io::Read;
use glium::Display;
use glium::vertex::{VertexBuffer, VertexBufferAny};
use obj;
use genmesh;
use std::slice;
use std::mem;

/// Reads a file into a string.
pub fn read_file(filename: &str) -> String {
    let mut file = File::open(filename).expect("file not found");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");
    contents
}

/// Returns a vertex buffer that should be rendered as `TrianglesList`.
pub fn load_wavefront(display: &Display, data: &[u8]) -> VertexBufferAny {
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 3],
        normal: [f32; 3],
        texture: [f32; 2],
    }

    implement_vertex!(Vertex, position, normal, texture);

    let mut data = ::std::io::BufReader::new(data);
    let data = obj::Obj::load_buf(&mut data).unwrap();

    let mut vertex_data = Vec::new();

    for object in data.objects.iter() {
        for polygon in object.groups.iter().flat_map(|g| g.polys.iter()) {
            match polygon {
                &genmesh::Polygon::PolyTri(genmesh::Triangle {
                    x: v1,
                    y: v2,
                    z: v3,
                }) => for v in [v1, v2, v3].iter() {
                    let position = data.position[v.0];
                    let texture = v.1.map(|index| data.texture[index]);
                    let normal = v.2.map(|index| data.normal[index]);

                    let texture = texture.unwrap_or([0.0, 0.0]);
                    let normal = normal.unwrap_or([0.0, 0.0, 0.0]);

                    vertex_data.push(Vertex {
                        position: position,
                        normal: normal,
                        texture: texture,
                    })
                },
                _ => unimplemented!(),
            }
        }
    }

    VertexBuffer::new(display, &vertex_data)
        .unwrap()
        .into_vertex_buffer_any()
}

/// This is used to reinterpret slices of floats as slices of repr(C) structs, without any
/// copying. It is optimal, but it is also punching holes in the type system. I hope that Rust
/// provides safe functionality to handle this in the future. In the meantime, reproduce
/// this workaround at your own risk.
pub fn reinterpret_cast_slice<S, T>(input: &[S]) -> &[T] {
    let length_in_bytes = input.len() * mem::size_of::<S>();
    let desired_length = length_in_bytes / mem::size_of::<T>();
    unsafe { slice::from_raw_parts(input.as_ptr() as *const T, desired_length) }
}
