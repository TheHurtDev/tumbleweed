//! This module contains the physics engine.

use hawthorn::math::{Quat, Vec3};
use std::time::Instant;

use crate::app::{Body, World};

mod particle_system;
pub use particle_system::*;

mod particle;
pub use particle::*;

/// Defines our physics pipeline.
///
/// This function is intended to be called many times per frame and drives
/// the game's physics simulation.
pub fn physics_pipeline(world: &mut World) {
    let now = Instant::now();
    let delta_secs = world.frame_duration.as_secs_f32();

    // Update particle systems
    let mut to_spawn: Vec<Body> = Vec::new();

    for (i, particle_system) in &mut world.entities.each_mut::<ParticleSystem>().enumerate() {
        if let Some(particle) = particle_system.spawn_particle(now) {
            let (color, scale) = match i {
                0 => (Vec3::new(0.9961, 0.5686, 0.6392), Vec3::splat(0.2)),
                1 => (Vec3::new(0.3569, 0.7843, 0.6980), Vec3::splat(0.3)),
                _ => (Vec3::new(0.6275, 0.8510, 0.9098), Vec3::splat(0.6)),
            };

            to_spawn.push(Body {
                particle,

                // These fields are only used by the renderer
                orientation: Quat::IDENTITY,
                scale,
                mesh_name: String::from("cube"),
                color,
            });
        }
    }

    for body in to_spawn {
        world.entities.spawn(body);
    }

    // Integrate kinematics properties
    for body in &mut world.entities.each_mut::<Body>() {
        let particle = &mut body.particle;

        particle.velocity += delta_secs * particle.acceleration;
        particle.position += delta_secs * particle.velocity;
    }
}
