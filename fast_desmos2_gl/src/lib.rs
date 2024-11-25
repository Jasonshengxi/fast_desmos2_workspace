#![allow(dead_code)]

pub mod buffer;
mod error;
pub mod glfw;
pub mod info;
pub mod shader;
pub mod vertex;

pub use error::{GlError, GlErrorGuard};

pub use gl;

#[macro_export]
macro_rules! transmutable_u32 {
    ($name: ident) => {
        impl $name {
            pub const fn to_u32(self) -> u32 {
                unsafe { std::mem::transmute(self) }
            }
        }

        impl From<$name> for u32 {
            fn from(value: $name) -> Self {
                value.to_u32()
            }
        }
    };
}
#[macro_export]
macro_rules! has_handle {
    ($name: ident) => {
        impl $name {
            pub fn as_handle(&self) -> GLuint {
                self.handle
            }
        }
    };
}
