use crevice::std140;
use hawthorn::math::Quat;

use crate::app::{Camera, SurfaceInfo};

#[derive(Debug, Clone, std140::AsStd140)]
#[repr(C)]
pub(in crate::app) struct SkyboxStd140 {
    projection_matrix: std140::Mat4,
    rotation_matrix: std140::Mat3,
}

impl SkyboxStd140 {
    pub(in crate::app) const VERTEX_BINDING_SIZE: usize =
        size_of::<<Self as std140::AsStd140>::Output>();
    pub(in crate::app) const FRAGMENT_BINDING_SIZE: usize =
        size_of::<<Self as std140::AsStd140>::Output>() - Self::VERTEX_BINDING_SIZE;

    pub(in crate::app) fn new(
        camera: &Camera,
        surface_info: &SurfaceInfo,
        orientation: Quat,
    ) -> Self {
        let projection_matrix = camera.to_perspective_proj_matrix(surface_info);
        let view_matrix = camera.orientation.inverse().to_rotation_mat3();
        let world_matrix = orientation.to_rotation_mat3();

        Self {
            projection_matrix: super::mat4_std140(projection_matrix),
            rotation_matrix: super::mat3_std140((view_matrix * world_matrix).inverse()),
        }
    }
}
