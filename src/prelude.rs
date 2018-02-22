#![allow(dead_code)]
use cgmath::{Decomposed, Matrix4, Point3, Quaternion, Rad, Transform3, Vector3};

pub type Point3f = Point3<f32>;
pub type Vector3f = Vector3<f32>;
pub type Matrix4f = Matrix4<f32>;
pub type Radf = Rad<f32>;
pub type Transform3f = Transform3<f32>;
pub type Quaternionf = Quaternion<f32>;
pub type Decomposedf = Decomposed<Vector3f, Quaternionf>;
