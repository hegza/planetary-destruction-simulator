#![allow(dead_code)]
use cgmath::{Decomposed, Deg, Euler, Matrix4, Point3, Quaternion, Rad, Transform3, Vector2,
             Vector3};

pub use std::f32::consts::PI;

pub type Radf = Rad<f32>;
pub type Degf = Deg<f32>;
pub type Point3f = Point3<f32>;
pub type Vector2f = Vector2<f32>;
pub type Vector2r = Vector2<Radf>;
pub type Vector3f = Vector3<f32>;
pub type Matrix4f = Matrix4<f32>;
pub type Transform3f = Transform3<f32>;
pub type Quaternionf = Quaternion<f32>;
pub type Decomposedf = Decomposed<Vector3f, Quaternionf>;
pub type Eulerf = Euler<Rad<f32>>;
