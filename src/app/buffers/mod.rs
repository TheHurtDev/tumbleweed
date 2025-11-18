use crevice::std140;
use hawthorn::math::{Mat3, Mat4, Vec3, Vec4};

mod mesh;
pub(in crate::app) use mesh::*;

mod entity_uniforms;
pub(in crate::app) use entity_uniforms::*;

mod global_uniforms;
pub(in crate::app) use global_uniforms::*;

mod skybox_uniforms;
pub(in crate::app) use skybox_uniforms::*;

mod uniforms_buffer;
pub(in crate::app) use uniforms_buffer::*;

#[inline]
fn vec3_std140(v: Vec3) -> std140::Vec3 {
    std140::Vec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

#[inline]
fn vec4_std140(v: Vec4) -> std140::Vec4 {
    std140::Vec4 {
        x: v.x,
        y: v.y,
        z: v.z,
        w: v.w,
    }
}

#[inline]
fn mat3_std140(m: Mat3) -> std140::Mat3 {
    std140::Mat3 {
        x: vec3_std140(m.as_cols()[0]),
        _pad_x: 0.0,
        y: vec3_std140(m.as_cols()[1]),
        _pad_y: 0.0,
        z: vec3_std140(m.as_cols()[2]),
        _pad_z: 0.0,
    }
}

#[inline]
fn mat4_std140(m: Mat4) -> std140::Mat4 {
    std140::Mat4 {
        x: vec4_std140(m.as_cols()[0]),
        y: vec4_std140(m.as_cols()[1]),
        z: vec4_std140(m.as_cols()[2]),
        w: vec4_std140(m.as_cols()[3]),
    }
}
