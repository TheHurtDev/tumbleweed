//! This module contains the physics engine.

use hawthorn::math::Vec3;

use crate::app::{Body, World};

mod particle;
pub use particle::*;

/// Defines our physics pipeline.
///
/// This function is intended to be called many times per second and drives
/// the game's physics simulation.
pub fn physics_pipeline(world: &mut World) {
    let delta_secs = world.frame_duration.as_secs_f32();

    // Integration
    for body in &mut world.entities.each_mut::<Body>() {
        let particle = &mut body.particle;

        let acceleration = particle.inv_mass * particle.force_accum;

        particle.force_accum = Vec3::ZERO;

        particle.velocity += delta_secs * acceleration;
        particle.position += delta_secs * particle.velocity;
    }
}
