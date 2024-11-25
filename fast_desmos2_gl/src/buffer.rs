use std::marker::PhantomData;

use crate::{has_handle, transmutable_u32};
use gl::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccessFrequency {
    /// Modified once, used a few times
    Stream,
    /// Modified once, used many times
    #[default]
    Static,
    /// Modified many times, used many times
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccessNature {
    /// Modified by app, used in draw and spec commands
    #[default]
    Draw,
    /// Modified by reading from GL, used to return data to app
    Read,
    /// Modified by reading from GL, used in draw and spec commands
    Copy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DataUsage {
    frequency: AccessFrequency,
    nature: AccessNature,
}

use AccessFrequency as Freq;
use AccessNature as Nat;

use super::error::GlErrorGuard;
impl DataUsage {
    pub const STATIC_DRAW: Self = Self::new(Freq::Static, Nat::Draw);

    pub const fn new(frequency: Freq, nature: Nat) -> Self {
        Self { frequency, nature }
    }

    pub const fn to_u32(self) -> u32 {
        match (self.frequency, self.nature) {
            (Freq::Stream, Nat::Draw) => gl::STREAM_DRAW,
            (Freq::Static, Nat::Draw) => gl::STATIC_DRAW,
            (Freq::Dynamic, Nat::Draw) => gl::DYNAMIC_DRAW,

            (Freq::Stream, Nat::Copy) => gl::STREAM_COPY,
            (Freq::Static, Nat::Copy) => gl::STATIC_COPY,
            (Freq::Dynamic, Nat::Copy) => gl::DYNAMIC_COPY,

            (Freq::Stream, Nat::Read) => gl::STREAM_READ,
            (Freq::Static, Nat::Read) => gl::STATIC_READ,
            (Freq::Dynamic, Nat::Read) => gl::DYNAMIC_READ,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferBindTarget {
    /// Vertex attributes
    ArrayBuffer = gl::ARRAY_BUFFER,
    /// Atomic counter storage
    AtomicCounter = gl::ATOMIC_COUNTER_BUFFER,
    /// Buffer copy source
    CopyRead = gl::COPY_READ_BUFFER,
    /// Buffer copy destination
    CopyWrite = gl::COPY_WRITE_BUFFER,
    /// Indirect compute dispatch commands
    DispatchIndirect = gl::DISPATCH_INDIRECT_BUFFER,
    /// Indirect command arguments
    DrawIndirect = gl::DRAW_INDIRECT_BUFFER,
    /// Vertex array indices
    ElementArray = gl::ELEMENT_ARRAY_BUFFER,
    /// Pixel read target
    PixelPack = gl::PIXEL_PACK_BUFFER,
    /// Texture data source
    PixelUnpack = gl::PIXEL_UNPACK_BUFFER,
    /// Query result buffer
    QueryBuffer = gl::QUERY_BUFFER,
    /// Read-write storage for shaders
    ShaderStorage = gl::SHADER_STORAGE_BUFFER,
    /// Texture data buffer
    Texture = gl::TEXTURE_BUFFER,
    /// Transform feedback buffer
    TransformFeedback = gl::TRANSFORM_FEEDBACK_BUFFER,
    /// Uniform block storage
    Uniform = gl::UNIFORM_BUFFER,
}
transmutable_u32!(BufferBindTarget);

impl BufferBindTarget {
    pub const fn can_bind_base(self) -> bool {
        matches!(
            self,
            Self::ShaderStorage | Self::Uniform | Self::AtomicCounter | Self::TransformFeedback
        )
    }
}

pub struct Buffer {
    handle: GLuint,
    target: BufferBindTarget,
}
has_handle!(Buffer);

#[must_use]
pub struct BufferBaseBinding {
    buffer: Buffer,
    index: u32,
}

impl BufferBaseBinding {
    pub fn bind_self(&self) {
        unsafe {
            gl::BindBufferBase(
                self.buffer.target.to_u32(),
                self.index,
                self.buffer.as_handle(),
            );
        }
    }
}

// impl Drop for Buffer {
//     fn drop(&mut self) {
//         unsafe {
//             gl::DeleteBuffers(1, &self.handle);
//         }
//     }
// }

impl Buffer {
    pub fn new(target: BufferBindTarget) -> Self {
        let mut handle = 0;
        unsafe { gl::GenBuffers(1, &mut handle) };
        assert_ne!(handle, 0, "Buffer generation failed.");
        unsafe { gl::BindBuffer(target.to_u32(), handle) };
        Self { handle, target }
    }

    pub fn store_realloc<T>(&self, data: &[T], usage: DataUsage) {
        unsafe {
            gl::NamedBufferData(
                self.handle,
                size_of_val(data) as GLsizeiptr,
                data.as_ptr().cast(),
                usage.to_u32(),
            );
        }
    }

    pub fn store_in_place<T>(&self, data: &[T]) {
        unsafe {
            gl::NamedBufferSubData(
                self.handle,
                0,
                size_of_val(data) as GLsizeiptr,
                data.as_ptr().cast(),
            )
        }
    }

    pub fn bind_self(&self) {
        unsafe {
            gl::BindBuffer(self.target.to_u32(), self.handle);
        }
    }

    pub fn into_base_binding(self, index: GLuint) -> BufferBaseBinding {
        assert!(self.target.can_bind_base());
        BufferBaseBinding {
            buffer: self,
            index,
        }
    }
}

type Len = usize;
type Cap = usize;
pub struct VecBuffer<T> {
    buffer: Buffer,
    len: Len,
    capacity: Cap,
    _phantom: PhantomData<T>,
    nature: AccessNature,
}

impl<T> VecBuffer<T> {
    pub fn new(target: BufferBindTarget, nature: AccessNature) -> Self {
        Self {
            buffer: Buffer::new(target),
            len: 0,
            capacity: 0,
            nature,
            _phantom: PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> Len {
        self.len
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    fn buffer_usage(&self) -> u32 {
        DataUsage::new(Freq::Dynamic, self.nature).to_u32()
    }

    pub fn store_data(&mut self, data: &[T]) {
        if data.len() > self.capacity {
            let new_cap = data.len().next_power_of_two().max(4);
            let new_cap_bytes = new_cap * size_of::<T>();

            let err_guard = GlErrorGuard::named("VecBuffer realloc");

            unsafe {
                gl::NamedBufferData(
                    self.buffer.as_handle(),
                    new_cap_bytes as GLsizeiptr,
                    std::ptr::null(),
                    self.buffer_usage(),
                );
            }
            self.capacity = new_cap;

            drop(err_guard);
        }

        let err_guard = GlErrorGuard::named("VecBuffer write data");

        unsafe {
            gl::NamedBufferSubData(
                self.buffer.as_handle(),
                0,
                std::mem::size_of_val(data) as GLsizeiptr,
                data.as_ptr().cast(),
            );
        }
        self.len = data.len();
        drop(err_guard);
    }
}
