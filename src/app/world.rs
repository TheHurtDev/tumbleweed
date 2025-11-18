use std::time::Duration;

use crate::app::{Camera, DrawCommand, Entity, InputState, SurfaceInfo};

pub struct World {
    pub frame_duration: Duration,
    pub surface_info: SurfaceInfo,
    pub input_state: InputState,
    pub camera: Camera,

    // Cleared once per frame at render time
    pub draw_commands: Vec<DrawCommand>,

    pub entities: EntityStore,
}

pub struct EntityStore {
    inner: Vec<Box<dyn Entity>>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            frame_duration: Duration::ZERO,
            surface_info: SurfaceInfo::default(),
            input_state: InputState::default(),
            camera: Camera::default(),

            draw_commands: Vec::new(),

            entities: EntityStore { inner: Vec::new() },
        }
    }
}

impl EntityStore {
    pub fn spawn<T>(&mut self, entity: T)
    where
        T: Entity + 'static,
    {
        self.inner.push(Box::new(entity));
    }

    pub fn despawn_all<F, T>(&mut self, mut predicate: F)
    where
        F: FnMut(&T) -> bool,
        T: Entity + 'static,
    {
        self.inner.retain(|e| {
            let Some(e) = e.as_any().downcast_ref::<T>() else {
                return true;
            };

            !(predicate)(e)
        });
    }

    pub fn each<T>(&self) -> impl Iterator<Item = &T>
    where
        T: Entity + 'static,
    {
        self.inner
            .iter()
            .filter_map(|e| e.as_any().downcast_ref::<T>())
    }

    pub fn each_mut<T>(&mut self) -> impl Iterator<Item = &mut T>
    where
        T: Entity + 'static,
    {
        self.inner
            .iter_mut()
            .filter_map(|e| e.as_any_mut().downcast_mut::<T>())
    }
}
