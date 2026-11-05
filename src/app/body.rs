use hawthorn::math::{Mat3, Mat4, Quat, Vec3};

use crate::physics::Particle;

/// A renderable physics body.
///
/// From the perspective of the physics engine, a [`Particle`] has only a
/// position as part of its transform. This type adds an orientation and
/// scale used solely for rendering.
///
/// It also associates a mesh and color with the [`Particle`] for rendering
/// purposes.
#[derive(Debug, Clone)]
pub struct Body {
    pub tag: String,
    pub particle: Particle,
    pub orientation: Quat,
    pub scale: Vec3,
    pub mesh_name: String,
    pub color: Vec3,
}

impl Body {
    pub fn to_world_matrix(&self) -> Mat4 {
        let scale_matrix = Mat3::from_diagonal(self.scale).to_mat4_homogeneous();
        let rotation_matrix = self.orientation.to_rotation_mat4();
        let translation_matrix = Mat4::IDENTITY.with_col(3, self.particle.position.to_vec4(1.0));

        translation_matrix * rotation_matrix * scale_matrix
    }
}
