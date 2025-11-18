use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Default, Debug, Clone)]
pub struct InputState {
    keys_down: HashSet<KeyCode>,
    keys_pressed: HashSet<KeyCode>,
    keys_released: HashSet<KeyCode>,
}

impl InputState {
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    pub(in crate::app) fn handle_key_event(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(key) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    self.keys_down.insert(key);
                    self.keys_pressed.insert(key);
                }
                ElementState::Released => {
                    self.keys_down.remove(&key);
                    self.keys_released.insert(key);
                }
            }
        }
    }

    pub(in crate::app) fn end_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
    }
}
