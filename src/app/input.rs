use std::collections::HashSet;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Default, Debug, Clone)]
pub struct InputState {
    keys_down: HashSet<KeyCode>,
    keys_pressed: HashSet<KeyCode>,
    keys_released: HashSet<KeyCode>,

    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_released: HashSet<MouseButton>,

    cursor_position: (u32, u32),
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

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&button)
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_released.contains(&button)
    }

    pub fn cursor_position(&self) -> (u32, u32) {
        self.cursor_position
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

    pub(in crate::app) fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.mouse_buttons_down.insert(button);
                self.mouse_buttons_pressed.insert(button);
            }
            ElementState::Released => {
                self.mouse_buttons_down.remove(&button);
                self.mouse_buttons_released.insert(button);
            }
        }
    }

    pub(in crate::app) fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_position = (position.x as u32, position.y as u32);
    }

    pub(in crate::app) fn end_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();

        self.mouse_buttons_pressed.clear();
        self.mouse_buttons_released.clear();
    }
}
