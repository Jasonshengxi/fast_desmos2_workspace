use std::{
    cell::RefCell,
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use crate::gl_safe::glfw::{Key, Window};

pub struct WindowWithInput {
    window: Window,
    keys_pressed: &'static RefCell<HashSet<Key>>,
    keys_released: &'static RefCell<HashSet<Key>>,
}

impl Deref for WindowWithInput {
    type Target = Window;
    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
impl DerefMut for WindowWithInput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.window
    }
}

impl WindowWithInput {
    pub fn new(window: Window) -> Self {
        fn leak<T>(x: T) -> &'static T {
            Box::leak(Box::new(x))
        }

        let keys_pressed = leak(RefCell::new(HashSet::new()));
        let keys_released = leak(RefCell::new(HashSet::new()));
        window.install_key_callback(|key, action, _| match action {
            glfw::Action::Press | glfw::Action::Repeat => {
                keys_pressed.borrow_mut().insert(key);
            }
            glfw::Action::Release => {
                keys_released.borrow_mut().insert(key);
            }
        });

        Self {
            window,
            keys_pressed,
            keys_released,
        }
    }

    pub fn clear_frame_specific(&self) {
        self.keys_pressed.borrow_mut().clear();
        self.keys_released.borrow_mut().clear();
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.borrow().contains(&key)
    }

    pub fn is_key_released(&self, key: Key) -> bool {
        self.keys_released.borrow().contains(&key)
    }
}
