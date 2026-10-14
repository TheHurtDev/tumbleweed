use hawthorn::math::{Quat, Vec3};

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
