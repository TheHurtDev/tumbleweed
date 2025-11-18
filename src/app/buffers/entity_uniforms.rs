use crevice::std140;
use hawthorn::math::Mat3;
use std::mem::offset_of;

use crate::app::DrawCommand;

#[derive(Debug, Clone, std140::AsStd140)]
#[repr(C)]
pub(in crate::app) struct EntityStd140 {
    world_matrix: std140::Mat4,
    world_normal_matrix: std140::Mat3,
    color: std140::Vec3,
}

impl EntityStd140 {
    pub(in crate::app) const VERTEX_BINDING_SIZE: usize =
        offset_of!(<Self as std140::AsStd140>::Output, color);
    pub(in crate::app) const FRAGMENT_BINDING_SIZE: usize =
        size_of::<<Self as std140::AsStd140>::Output>() - Self::VERTEX_BINDING_SIZE;

    pub(in crate::app) fn from_draw_command(draw_command: DrawCommand) -> Self {
        let world_normal_matrix = draw_command
            .world_matrix
            .to_upper_left_mat3()
            .try_inverse()
            .unwrap_or(Mat3::IDENTITY)
            .transpose();

        Self {
            world_matrix: super::mat4_std140(draw_command.world_matrix),
            world_normal_matrix: super::mat3_std140(world_normal_matrix),
            color: super::vec3_std140(draw_command.color),
        }
    }
}
