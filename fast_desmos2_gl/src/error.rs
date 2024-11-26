use crate::transmutable_u32;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlError {
    InvalidEnum = gl::INVALID_ENUM,
    InvalidValue = gl::INVALID_VALUE,
    InvalidOperation = gl::INVALID_OPERATION,
    InvalidFramebufferOperation = gl::INVALID_FRAMEBUFFER_OPERATION,
    OutOfMemory = gl::OUT_OF_MEMORY,
    StackUnderflow = gl::STACK_UNDERFLOW,
    StackOverflow = gl::STACK_OVERFLOW,
}
transmutable_u32!(GlError);

impl GlError {
    pub fn try_get() -> Option<Self> {
        let err_num = unsafe { gl::GetError() };
        Some(match err_num {
            gl::NO_ERROR => return None,
            gl::INVALID_ENUM => GlError::InvalidEnum,
            gl::INVALID_VALUE => GlError::InvalidValue,
            gl::INVALID_OPERATION => GlError::InvalidOperation,
            gl::INVALID_FRAMEBUFFER_OPERATION => GlError::InvalidFramebufferOperation,
            gl::OUT_OF_MEMORY => GlError::OutOfMemory,
            gl::STACK_UNDERFLOW => GlError::StackUnderflow,
            gl::STACK_OVERFLOW => GlError::StackOverflow,
            _ => unreachable!(),
        })
    }
}

/// Clears any GL errors on creation, asserts
/// that no GL errors occur in its lifetime.
///
/// Checks for GL errors on `Drop`, and panics
/// if any occured.
pub struct GlErrorGuard {
    name: Option<&'static str>,
}
impl Drop for GlErrorGuard {
    fn drop(&mut self) {
        if let Some(err) = GlError::try_get() {
            match self.name {
                Some(name) => panic!("GL error assert \"{name}\" failed: {err:?}"),
                None => panic!("GL error assert failed: {err:?}"),
            }
        }
    }
}

impl Default for GlErrorGuard {
    fn default() -> Self {
        Self::new_internal(None)
    }
}

impl GlErrorGuard {
    pub fn clear_existing(name: Option<&'static str>) {
        if let Some(err) = GlError::try_get() {
            match name {
                Some(name) => println!("Existing error on guard \"{name}\" creation: {err:?}"),
                None => println!("Existing error on guard creation: {err:?}"),
            }
        }
    }

    fn new_internal(name: Option<&'static str>) -> Self {
        Self::clear_existing(name);
        Self { name }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &'static str) -> Self {
        Self::new_internal(Some(name))
    }

    pub fn guarded<T>(run: impl FnOnce() -> T) -> T {
        let guard = Self::new();
        let result = run();
        drop(guard);
        result
    }

    pub fn guard_named<T>(name: &'static str, run: impl FnOnce() -> T) -> T {
        let guard = Self::named(name);
        let result = run();
        drop(guard);
        result
    }
}
