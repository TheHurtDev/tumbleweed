use hawthorn::math::Vec3;

/// A physics body represented as a single point in space.
#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
        }
    }
}
