use color_eyre::{eyre::eyre, Result as EyreResult};
use fast_desmos2_utils::OptExt;
use glam::{DVec2, IVec2};
use std::{
    cell::RefCell,
    ffi::{c_void, CStr, CString},
    ptr::{self, NonNull},
};

use glfw::ffi;
pub use glfw::{Action, Key, Modifiers};

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum GlfwError {
    NotInitialized,
    NoCurrentContext,
    InvalidEnum,
    InvalidValue,
    OutOfMemory,
    ApiUnavailable,
    VersionUnavailable,
    PlatformError,
    FormatUnavailable,
    NoWindowContext,
    CursorUnavilable,
    FeatureUnavailable,
    FeatureUnimplemented,
    PlatformUnavailable,
}

impl GlfwError {
    pub fn from_num(err: i32) -> Option<Self> {
        Some(match err {
            0 => return None,
            0x00010001 => Self::NotInitialized,
            0x00010002 => Self::NoCurrentContext,
            0x00010003 => Self::InvalidEnum,
            0x00010004 => Self::InvalidValue,
            0x00010005 => Self::OutOfMemory,
            0x00010006 => Self::ApiUnavailable,
            0x00010007 => Self::VersionUnavailable,
            0x00010008 => Self::PlatformError,
            0x00010009 => Self::FormatUnavailable,
            0x0001000a => Self::NoWindowContext,
            0x0001000b => Self::CursorUnavilable,
            0x0001000c => Self::FeatureUnavailable,
            0x0001000d => Self::FeatureUnimplemented,
            0x0001000e => Self::PlatformUnavailable,
            _ => unreachable!(),
        })
    }
}

extern "C" fn err_callback(err: i32, desc: *const i8) {
    let err = GlfwError::from_num(err).unwrap_unreach();
    let desc = unsafe { CStr::from_ptr(desc).to_str().unwrap() };
    println!("Error occured: {err:?}");
    println!("Description: {desc}");
}

pub fn init() -> Option<()> {
    let err = unsafe { ffi::glfwInit() };
    (err == 1).then_some(())
}

pub fn install_errors() {
    unsafe { ffi::glfwSetErrorCallback(Some(err_callback)) };
}

pub fn get_proc_address(name: &'static str) -> *const c_void {
    let name = CString::new(name).unwrap_unreach();
    unsafe { ffi::glfwGetProcAddress(name.as_ptr()) }
}

pub fn poll_events() {
    unsafe { ffi::glfwPollEvents() }
}

pub struct Window {
    window: NonNull<ffi::GLFWwindow>,
}

impl Window {
    pub fn create(width: i32, height: i32, title: &str) -> EyreResult<Self> {
        let as_c_str = CString::new(title)?;
        let window = unsafe {
            ffi::glfwCreateWindow(
                width,
                height,
                as_c_str.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };

        if let Some(window) = NonNull::new(window) {
            Ok(Self { window })
        } else {
            Err(eyre!("Window creation failed: nullptr"))
        }
    }

    pub fn should_close(&self) -> bool {
        (unsafe { ffi::glfwWindowShouldClose(self.window.as_ptr()) }) > 0
    }

    pub fn swap_buffers(&self) {
        unsafe { ffi::glfwSwapBuffers(self.window.as_ptr()) }
    }

    pub fn make_current(&self) {
        unsafe { ffi::glfwMakeContextCurrent(self.window.as_ptr()) };
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        let points =
            unsafe { ffi::glfwGetKey(self.window.as_ptr(), std::mem::transmute::<Key, i32>(key)) };
        match points {
            ffi::PRESS => true,
            ffi::RELEASE => false,
            _ => unreachable!(),
        }
    }

    pub fn get_mouse_pos(&self) -> DVec2 {
        let mut result = DVec2::ZERO;
        unsafe { ffi::glfwGetCursorPos(self.window.as_ptr(), &mut result.x, &mut result.y) };
        result
    }

    pub fn get_framebuffer_size(&self) -> IVec2 {
        let mut result = IVec2::ZERO;
        unsafe { ffi::glfwGetFramebufferSize(self.window.as_ptr(), &mut result.x, &mut result.y) };
        result
    }

    pub fn install_key_callback(&self, callback: impl KeyCallback) {
        set_key_callback(callback);
        unsafe { ffi::glfwSetKeyCallback(self.window.as_ptr(), Some(key_callback)) };
    }

    pub fn install_framebuffer_size_callback(&self, callback: impl FramebufferSizeCallback) {
        set_framebuffer_size_callback(callback);
        unsafe {
            ffi::glfwSetFramebufferSizeCallback(
                self.window.as_ptr(),
                Some(framebuffer_size_callback),
            )
        };
    }

    pub fn install_scroll_callback(&self, callback: impl ScrollCallback) {
        set_scroll_callback(callback);
        unsafe { ffi::glfwSetScrollCallback(self.window.as_ptr(), Some(scroll_callback)) };
    }
}

extern "C" fn scroll_callback(_window: *mut ffi::GLFWwindow, x: f64, y: f64) {
    let scroll = DVec2::new(x, y);
    SCROLL_CALLBACK.with_borrow_mut(|callback| {
        if let Some(callback) = callback.as_mut() {
            callback(scroll);
        }
    })
}

extern "C" fn key_callback(
    _window: *mut ffi::GLFWwindow,
    key: i32,
    _scancode: i32,
    action: i32,
    mods: i32,
) {
    let action = match action {
        ffi::PRESS => glfw::Action::Press,
        ffi::RELEASE => glfw::Action::Release,
        ffi::REPEAT => glfw::Action::Repeat,
        _ => unreachable!(),
    };
    let key = unsafe { std::mem::transmute::<i32, glfw::Key>(key) };
    let mods = glfw::Modifiers::from_bits(mods).unwrap_unreach();

    KEY_CALLBACK.with_borrow_mut(|callback| {
        if let Some(callback) = callback.as_mut() {
            callback(key, action, mods);
        }
    });
}

extern "C" fn framebuffer_size_callback(_window: *mut ffi::GLFWwindow, x: i32, y: i32) {
    let pos = IVec2::new(x, y);
    FRAMEBUFFER_SIZE_CALLBACK.with_borrow_mut(|callback| {
        if let Some(callback) = callback.as_mut() {
            callback(pos);
        }
    })
}

macro_rules! trait_alias {
    ($v:vis trait $alias:ident = $($tr:tt)*) =>{
        $v trait $alias: $($tr)* {}
        impl <T: $($tr)*> $alias for T {}
    };
}
macro_rules! store_callback {
    ( $v:vis static $callback_name:ident : $type_name:ident = $fn_name:ident) => {
        $v fn $fn_name(callback: impl $type_name) {
            $callback_name.with_borrow_mut(|value| *value = Some(Box::new(callback)));
        }
        thread_local! {
            $v static $callback_name: RefCell<Option<Box<dyn $type_name>>> = const { RefCell::new(None) };
        }
    };
}

trait_alias!(pub trait KeyCallback = FnMut(glfw::Key, glfw::Action, glfw::Modifiers) + 'static);
store_callback!(static KEY_CALLBACK: KeyCallback = set_key_callback);

trait_alias!(pub trait FramebufferSizeCallback = FnMut(IVec2) + 'static);
store_callback!(static FRAMEBUFFER_SIZE_CALLBACK: FramebufferSizeCallback = set_framebuffer_size_callback);

trait_alias!(pub trait ScrollCallback = FnMut(DVec2) + 'static);
store_callback!(static SCROLL_CALLBACK: ScrollCallback = set_scroll_callback);
