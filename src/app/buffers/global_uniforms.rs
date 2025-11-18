use crevice::std140;
use hawthorn::math::Vec3;
use std::mem::offset_of;

use crate::app::{Camera, SurfaceInfo};

#[derive(Debug, Clone, std140::AsStd140)]
#[repr(C)]
pub(in crate::app) struct GlobalsStd140 {
    perspective_view_proj_matrix: std140::Mat4,
    shadow_box_view_proj_matrix: std140::Mat4,
    camera_position: std140::Vec3,
}

const SUNLIGHT_DIRECTION: Vec3 = Vec3::new(-0.371391, -0.742781, -0.557086);

impl GlobalsStd140 {
    pub(in crate::app) const VERTEX_BINDING_SIZE: usize =
        offset_of!(<Self as std140::AsStd140>::Output, camera_position);
    pub(in crate::app) const FRAGMENT_BINDING_SIZE: usize =
        size_of::<<Self as std140::AsStd140>::Output>() - Self::VERTEX_BINDING_SIZE;

    pub(in crate::app) fn new(camera: &Camera, surface_info: &SurfaceInfo) -> Self {
        let camera_view_matrix = camera.to_view_matrix();
        let camera_proj_matrix = camera.to_perspective_proj_matrix(surface_info);

        let shadow_box_view_proj_matrix =
            camera.to_shadow_box_view_proj_matrix(surface_info, SUNLIGHT_DIRECTION);

        Self {
            perspective_view_proj_matrix: super::mat4_std140(
                camera_proj_matrix * camera_view_matrix,
            ),
            shadow_box_view_proj_matrix: super::mat4_std140(shadow_box_view_proj_matrix),
            camera_position: super::vec3_std140(camera.position),
        }
    }
}
