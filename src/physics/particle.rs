use hawthorn::math::Vec3;

/// A physics body represented as a single point in space.
#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub inv_mass: f32,

    pub(crate) force_accum: Vec3,
}

impl Particle {
    /// Creates a new dynamic particle.
    pub fn new_dynamic(mass: f32) -> Self {
        Self {
            inv_mass: mass.recip(),
            ..Default::default()
        }
    }

    /// Creates a new static particle with an infinite mass.
    pub fn new_static() -> Self {
        Self::default()
    }

    /// Adds a force applied to the particle.
    ///
    /// The net force applied to the particle will be integrated then reset
    /// to zero at the end of each frame.
    pub fn add_force(&mut self, force: Vec3) {
        self.force_accum += force;
    }

    /// Sets the acceleration of the particle.
    ///
    /// Use caution when calling this method, as it may override other forces
    /// applied to the particle.
    pub fn set_acceleration(&mut self, acceleration: Vec3) {
        let Some(mass) = self.mass() else {
            return;
        };

        self.force_accum = mass * acceleration;
    }

    /// Returns the mass of the particle, or None if the particle is static.
    pub fn mass(&self) -> Option<f32> {
        let mass = self.inv_mass.recip();

        if mass.is_finite() { Some(mass) } else { None }
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;

        self
    }

    pub fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;

        self
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            inv_mass: 0.0,
            force_accum: Vec3::ZERO,
        }
    }
}
