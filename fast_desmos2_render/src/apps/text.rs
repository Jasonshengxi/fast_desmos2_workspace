use crate::gl_safe::{
    buffer::{
        AccessNature, Buffer, BufferBaseBinding, BufferBindTarget, DataUsage,
        VecBuffer,
    },
    shader::{Shader, ShaderProgram},
    vertex::{AttrType, VertexArrayObject},
    GlErrorGuard,
};
use fast_desmos2_fonts::glyph_data::GpuGlyphData;
use glam::{IVec2, Vec2};
use std::num::NonZeroU32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlyphInstance {
    pos: Vec2,
    size: Vec2,
    index: u32,
}

impl GlyphInstance {
    pub fn new(pos: Vec2, size: Vec2, index: u32) -> Self {
        Self { pos, size, index }
    }
}

pub struct TextApp {
    ib: VecBuffer<GlyphInstance>,
    vao: VertexArrayObject,
    program: ShaderProgram,

    aspect_transform: Vec2,
    glyph_data_bindings: [BufferBaseBinding; 4],
}

impl TextApp {
    pub fn new(gpu_glyph_data: GpuGlyphData) -> Self {
        let vs = Shader::vertex(include_str!("shader/text.vert"));
        let fs = Shader::fragment(include_str!("shader/text.frag"));
        let program = ShaderProgram::new([vs, fs]);

        let vb = Buffer::new(BufferBindTarget::ArrayBuffer);
        vb.store_realloc(
            &[Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y],
            DataUsage::STATIC_DRAW,
        );

        let mut ib = VecBuffer::new(BufferBindTarget::ArrayBuffer, AccessNature::Draw);
        ib.store_data(&[
            GlyphInstance::new(Vec2::ZERO, Vec2::splat(0.2), 0),
            GlyphInstance::new(Vec2::splat(0.5), Vec2::splat(0.1), 0),
        ]);

        let vao = VertexArrayObject::new();
        let mut vb_attach = vao.attach_vertex_buffer(&vb, size_of::<Vec2>() as i32);
        vb_attach.add_attr(AttrType::Float, 2);

        let mut ib_attach =
            vao.attach_vertex_buffer(ib.buffer(), size_of::<GlyphInstance>() as i32);
        ib_attach.add_attr(AttrType::Float, 2);
        ib_attach.add_attr(AttrType::Float, 2);
        ib_attach.add_attr(AttrType::Uint, 1);
        ib_attach.set_instance_divisor(NonZeroU32::new(1));

        fn make_and_bind_ssbo<T>(data: &[T], binding_index: u32) -> BufferBaseBinding {
            let ssbo = Buffer::new(BufferBindTarget::ShaderStorage);
            ssbo.store_realloc(data, DataUsage::STATIC_DRAW);
            ssbo.into_base_binding(binding_index)
        }
        let glyph_data_bindings = [
            make_and_bind_ssbo(&gpu_glyph_data.points, 0),
            make_and_bind_ssbo(&gpu_glyph_data.verbs, 1),
            make_and_bind_ssbo(&gpu_glyph_data.glyph_starts, 2),
            make_and_bind_ssbo(&gpu_glyph_data.bounds, 3),
        ];

        let aspect_transform = Vec2::ONE;

        Self {
            ib,
            vao,
            program,
            aspect_transform,
            glyph_data_bindings,
        }
    }

    pub fn on_resize(&mut self, new_size: IVec2) {
        let new_size = Vec2::new(new_size.x as f32, new_size.y as f32);
        let min_bound = new_size.min_element();
        self.aspect_transform = min_bound / new_size;
    }

    pub fn render(&self) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };

        GlErrorGuard::guard_named("Program bind", || self.program.use_self());
        GlErrorGuard::guard_named("VAO bind", || self.vao.use_self());
        GlErrorGuard::guard_named("Uniform bind", || {
            self.program.set_uniform_vec2(0, self.aspect_transform)
        });
        GlErrorGuard::guard_named("SSBO bind", || {
            for buf in &self.glyph_data_bindings {
                buf.bind_self();
            }
        });

        GlErrorGuard::guard_named("Draw", || unsafe {
            gl::DrawArraysInstanced(gl::TRIANGLE_FAN, 0, 4, self.ib.len() as i32)
        });
    }
}
