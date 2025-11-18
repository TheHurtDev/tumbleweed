use std::any::Any;

use crate::{app::Body, physics::ParticleSystem};

pub trait Entity {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

macro_rules! impl_entity {
    ($ty:ty) => {
        impl Entity for $ty {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }
    };
}

impl_entity!(Body);
impl_entity!(ParticleSystem);
