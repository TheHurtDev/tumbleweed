use hawthorn::math::Vec3;
use rand::prelude::*;
use std::ops::RangeInclusive;
use std::time::{Duration, Instant};

use crate::physics::Particle;

/// Periodically spawns particles with parameterized kinematic properties.
#[derive(Debug, Clone)]
pub struct ParticleSystem {
    pub spawn_period: Duration,
    pub position: Vec3,
    pub launch_angle: RangeInclusive<f32>,
    pub speed_range: RangeInclusive<f32>,
    pub gravity: f32,

    last_spawn_time: Option<Instant>,
}

impl ParticleSystem {
    pub fn new(
        spawn_period: Duration,
        position: Vec3,
        launch_angle: RangeInclusive<f32>,
        speed_range: RangeInclusive<f32>,
        gravity: f32,
    ) -> Self {
        Self {
            spawn_period,
            position,
            launch_angle,
            speed_range,
            gravity,

            last_spawn_time: None,
        }
    }

    pub fn spawn_particle(&mut self, now: Instant) -> Option<Particle> {
        let last_spawn_time = match self.last_spawn_time {
            Some(t) => t,
            None => {
                // Spawn immediately on the first attempt
                self.last_spawn_time = Some(now);

                return Some(self.do_spawn());
            }
        };

        let since_last_spawn = now.saturating_duration_since(last_spawn_time);

        if since_last_spawn < self.spawn_period {
            // Not enough time has passed to spawn a new particle
            return None;
        }

        self.last_spawn_time = Some(now);

        Some(self.do_spawn())
    }

    fn do_spawn(&self) -> Particle {
        let mut rng = rand::rng();

        // theta is vertical angle from the +Y axis
        // phi is the horizontal angle around the +Y axis

        let cos_min = self.launch_angle.end().cos();
        let cos_max = self.launch_angle.start().cos();

        let cos_theta = rng.random_range(cos_min..=cos_max);
        let sin_theta = cos_theta.acos().sin();

        let phi = rng.random_range(0.0..core::f32::consts::TAU);

        let direction = Vec3::new(sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin());

        let speed = rng.random_range(self.speed_range.clone());

        Particle {
            position: self.position,
            velocity: direction * speed,
            acceleration: -self.gravity * Vec3::Y,
        }
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new(
            Duration::from_millis(200),
            Vec3::ZERO,
            0.0_f32.to_radians()..=45.0_f32.to_radians(),
            5.0..=10.0,
            15.0,
        )
    }
}
