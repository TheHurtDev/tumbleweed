use hawthorn::math::{Mat3, Mat4, Quat, Vec3};

use crate::app::SurfaceInfo;

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub orientation: Quat,
}

impl Camera {
    pub const FOV: f32 = 75.0_f32.to_radians();
    pub const NEAR_PLANE: f32 = 0.1;
    pub const FAR_PLANE: f32 = 1_000.0;
    pub const SHADOW_DISTANCE: f32 = 50.0;
    pub const SHADOW_BOX_PADDING: f32 = 10.0;

    pub fn to_view_matrix(&self) -> Mat4 {
        let rotation_mat = self.orientation.to_rotation_mat4().transpose();

        let translation_mat = Mat4::IDENTITY.with_col(3, (-self.position).to_vec4(1.0));

        rotation_mat * translation_mat
    }

    pub fn to_perspective_proj_matrix(&self, surface_info: &SurfaceInfo) -> Mat4 {
        const Z_RANGE: f32 = Camera::FAR_PLANE - Camera::NEAR_PLANE;

        let tan_half_fov_recip = (Self::FOV * 0.5).tan().recip();

        #[rustfmt::skip]
        let proj_matrix = Mat4::new(
            tan_half_fov_recip / surface_info.aspect_ratio(),   0.0,                0.0,                        0.0,
            0.0,                                                tan_half_fov_recip, 0.0,                        0.0,
            0.0,                                                0.0,                -Self::FAR_PLANE / Z_RANGE, -(Self::FAR_PLANE * Self::NEAR_PLANE) / Z_RANGE,
            0.0,                                                0.0,                -1.0,                       0.0,
        );

        proj_matrix
    }

    pub fn to_shadow_box_view_proj_matrix(
        &self,
        surface_info: &SurfaceInfo,
        light_dir: Vec3,
    ) -> Mat4 {
        let (frustum_x, frustum_y, frustum_z) = self.orientation.to_orthonormal_basis();

        let tan_half_fov = (Self::FOV / 2.0).tan();
        let near_half_height = Self::NEAR_PLANE * tan_half_fov;
        let near_half_width = near_half_height * surface_info.aspect_ratio();
        let far_half_height = Self::SHADOW_DISTANCE * tan_half_fov;
        let far_half_width = far_half_height * surface_info.aspect_ratio();

        let near_center = self.position - Self::NEAR_PLANE * frustum_z;
        let far_center = self.position - Self::SHADOW_DISTANCE * frustum_z;

        let frustum_corner_world_positions: [Vec3; 8] = [
            near_center - near_half_width * frustum_x - near_half_height * frustum_y,
            near_center - near_half_width * frustum_x + near_half_height * frustum_y,
            near_center + near_half_width * frustum_x - near_half_height * frustum_y,
            near_center + near_half_width * frustum_x + near_half_height * frustum_y,
            far_center - far_half_width * frustum_x - far_half_height * frustum_y,
            far_center - far_half_width * frustum_x + far_half_height * frustum_y,
            far_center + far_half_width * frustum_x - far_half_height * frustum_y,
            far_center + far_half_width * frustum_x + far_half_height * frustum_y,
        ];

        let rotation_mat = Quat::try_look_to_rh(light_dir, Vec3::Y)
            .expect("light_dir must not be colinear with the y axis")
            .inverse()
            .to_rotation_mat3();

        let frustum_corner_rotated_positions_iter = frustum_corner_world_positions
            .iter()
            .map(|&p| rotation_mat * p);

        let shadow_box_rotated_min = frustum_corner_rotated_positions_iter
            .clone()
            .fold(Vec3::splat(f32::MAX), |a, b| a.min(b))
            - Vec3::splat(Self::SHADOW_BOX_PADDING);
        let shadow_box_rotated_max = frustum_corner_rotated_positions_iter
            .clone()
            .fold(Vec3::splat(f32::MIN), |a, b| a.max(b))
            + Vec3::splat(Self::SHADOW_BOX_PADDING);

        let shadow_box_rotated_position = ((shadow_box_rotated_min + shadow_box_rotated_max) / 2.0)
            .with(2, shadow_box_rotated_max.z);

        let shadow_box_half_width = shadow_box_rotated_max.x - shadow_box_rotated_position.x;
        let shadow_box_half_height = shadow_box_rotated_max.y - shadow_box_rotated_position.y;
        let shadow_box_depth = shadow_box_rotated_max.z - shadow_box_rotated_min.z;

        let projection_mat = Mat3::from_diagonal(Vec3::new(
            shadow_box_half_width.recip(),
            shadow_box_half_height.recip(),
            -shadow_box_depth.recip(),
        ))
        .to_mat4_homogeneous();

        let light_view_mat = rotation_mat
            .to_mat4_homogeneous()
            .with_col(3, (-shadow_box_rotated_position).to_vec4(1.0));

        projection_mat * light_view_mat
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
        }
    }
}
