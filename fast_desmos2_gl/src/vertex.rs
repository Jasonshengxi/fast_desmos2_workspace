use std::{cell::Cell, num::NonZeroU32};

use crate::{has_handle, transmutable_u32};
use gl::types::*;

use super::buffer::Buffer;

pub struct VertexArrayObject {
    handle: u32,
    next_vbo_binding: Cell<u32>,
    next_attr_binding: Cell<u32>,
}
has_handle!(VertexArrayObject);

// impl Drop for VertexArrayObject {
//     fn drop(&mut self) {
//         unsafe {
//             gl::DeleteVertexArrays(1, &self.handle);
//         }
//     }
// }

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum AttrType {
    Float = gl::FLOAT,
    Uint = gl::UNSIGNED_INT,
    Int = gl::INT,
}
transmutable_u32!(AttrType);

enum GlVertexFormatFunc {
    Float,
    Int,
    Long,
}

impl AttrType {
    pub fn size(&self) -> u32 {
        match self {
            Self::Float => 4,
            Self::Uint => 4,
            Self::Int => 4,
        }
    }

    fn function_to_use(&self) -> GlVertexFormatFunc {
        match self {
            Self::Float => GlVertexFormatFunc::Float,
            Self::Int => GlVertexFormatFunc::Int,
            Self::Uint => GlVertexFormatFunc::Int,
        }
    }
}

pub struct VertexBufferBinding<'vao, 'buf> {
    vertex_array: &'vao VertexArrayObject,
    bind_index: u32,
    buffer: &'buf Buffer,
    next_offet: u32,
}

impl VertexBufferBinding<'_, '_> {
    pub fn set_instance_divisor(&self, divisor: Option<NonZeroU32>) {
        unsafe {
            gl::VertexArrayBindingDivisor(
                self.vertex_array.as_handle(),
                self.bind_index,
                divisor.map_or(0, NonZeroU32::get),
            )
        }
    }

    pub fn add_attr(&mut self, attr_type: AttrType, component_count: u32) {
        let attr_index = self.vertex_array.next_attr_index();
        unsafe {
            match attr_type.function_to_use() {
                GlVertexFormatFunc::Float => {
                    gl::VertexArrayAttribFormat(
                        self.vertex_array.as_handle(),
                        attr_index,
                        component_count as i32,
                        attr_type.to_u32(),
                        gl::FALSE,
                        self.next_offet,
                    )
                }
                GlVertexFormatFunc::Int => {
                    gl::VertexArrayAttribIFormat(
                        self.vertex_array.as_handle(),
                        attr_index,
                        component_count as i32,
                        attr_type.to_u32(),
                        self.next_offet,
                    )
                }
                GlVertexFormatFunc::Long => {
                    gl::VertexArrayAttribLFormat(
                        self.vertex_array.as_handle(), 
                        attr_index, 
                        component_count as i32, 
                        attr_type.to_u32(), 
                        self.next_offet
                    )
                }
            }
            self.next_offet += attr_type.size() * component_count;
            gl::VertexArrayAttribBinding(
                self.vertex_array.as_handle(),
                attr_index,
                self.bind_index,
            );
            gl::EnableVertexArrayAttrib(self.vertex_array.as_handle(), attr_index);
        }
    }
}

impl Default for VertexArrayObject {
    fn default() -> Self {
        Self::new()
    }
}

impl VertexArrayObject {
    fn next_attr_index(&self) -> u32 {
        let attr_index = self.next_attr_binding.get();
        self.next_attr_binding.set(attr_index + 1);
        attr_index
    }

    pub fn new() -> Self {
        let mut handle = 0;
        unsafe { gl::GenVertexArrays(1, &mut handle) };
        assert_ne!(handle, 0, "vertex array creation failed.");
        unsafe { gl::BindVertexArray(handle) };
        Self {
            handle,
            next_vbo_binding: Cell::new(0),
            next_attr_binding: Cell::new(0),
        }
    }

    pub fn use_self(&self) {
        unsafe {
            gl::BindVertexArray(self.handle);
        }
    }

    pub fn attach_vertex_buffer<'buf>(
        &self,
        buffer: &'buf Buffer,
        stride: i32,
    ) -> VertexBufferBinding<'_, 'buf> {
        let bind_index = self.next_vbo_binding.get();
        self.next_vbo_binding.set(bind_index + 1);
        unsafe {
            gl::VertexArrayVertexBuffer(self.handle, bind_index, buffer.as_handle(), 0, stride);
        }
        VertexBufferBinding {
            vertex_array: self,
            bind_index,
            buffer,
            next_offet: 0,
        }
    }
}
