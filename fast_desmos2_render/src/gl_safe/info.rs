use std::ffi::CStr;

use crate::transmutable_u32;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum GlString {
    Vendor = gl::VENDOR,
    Renderer = gl::RENDERER,
    Version = gl::VERSION,
    ShadingLanguageVersion = gl::SHADING_LANGUAGE_VERSION,
}
transmutable_u32!(GlString);

impl GlString {
    pub fn get_gl(&self) -> &'static str {
        let str_ptr = unsafe { gl::GetString(self.to_u32()) };
        unsafe { CStr::from_ptr(str_ptr.cast()).to_str().unwrap() }
    }
}
