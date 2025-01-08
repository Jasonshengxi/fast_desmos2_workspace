use std::{
    cell::RefCell,
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use fast_desmos2_gl::glfw::{self, Action, Key, Window};
use fast_desmos2_utils as utils;

pub trait KeyExt {
    fn as_char(self) -> Option<char>;
}

impl KeyExt for glfw::Key {
    fn as_char(self) -> Option<char> {
        Some(match self {
            Key::Space => ' ',
            Key::Apostrophe => '\'',
            Key::Comma => ',',
            Key::Minus => '-',
            Key::Period => '.',
            Key::Slash => '/',
            Key::Num0 => todo!(),
            Key::Num1 => todo!(),
            Key::Num2 => todo!(),
            Key::Num3 => todo!(),
            Key::Num4 => todo!(),
            Key::Num5 => todo!(),
            Key::Num6 => todo!(),
            Key::Num7 => todo!(),
            Key::Num8 => todo!(),
            Key::Num9 => todo!(),
            Key::Semicolon => ';',
            Key::Equal => '=',
            Key::A => 'a',
            Key::B => 'b',
            Key::C => 'c',
            Key::D => 'd',
            Key::E => 'e',
            Key::F => 'f',
            Key::G => 'g',
            Key::H => 'h',
            Key::I => 'i',
            Key::J => 'j',
            Key::K => 'k',
            Key::L => 'l',
            Key::M => 'm',
            Key::N => 'n',
            Key::O => 'o',
            Key::P => 'p',
            Key::Q => 'q',
            Key::R => 'r',
            Key::S => 's',
            Key::T => 't',
            Key::U => 'u',
            Key::V => 'v',
            Key::W => 'w',
            Key::X => 'x',
            Key::Y => 'y',
            Key::Z => 'z',
            Key::LeftBracket => '[',
            Key::Backslash => '\\',
            Key::RightBracket => ']',
            Key::GraveAccent => '`',
            Key::World1 => todo!(),
            Key::World2 => todo!(),
            Key::Kp0 => todo!(),
            Key::Kp1 => todo!(),
            Key::Kp2 => todo!(),
            Key::Kp3 => todo!(),
            Key::Kp4 => todo!(),
            Key::Kp5 => todo!(),
            Key::Kp6 => todo!(),
            Key::Kp7 => todo!(),
            Key::Kp8 => todo!(),
            Key::Kp9 => todo!(),
            Key::KpDecimal => todo!(),
            Key::KpDivide => todo!(),
            Key::KpMultiply => todo!(),
            Key::KpSubtract => todo!(),
            Key::KpAdd => todo!(),
            Key::KpEnter => todo!(),
            Key::KpEqual => todo!(),
            _ => return None,
        })
    }
}

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
        let keys_pressed = utils::leak(RefCell::new(HashSet::new()));
        let keys_released = utils::leak(RefCell::new(HashSet::new()));
        window.install_key_callback(|key, action, _| match action {
            Action::Press | Action::Repeat => {
                keys_pressed.borrow_mut().insert(key);
            }
            Action::Release => {
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

    pub fn keys_pressed(&self) -> std::cell::Ref<HashSet<glfw::Key>> {
        self.keys_pressed.borrow()
    }

    pub fn is_key_released(&self, key: Key) -> bool {
        self.keys_released.borrow().contains(&key)
    }
}
