use fast_desmos2_fonts::{
    glyph_data::{BoundingBox, GpuGlyphData},
    layout,
};
use fast_desmos2_gl::{
    buffer::{AccessNature, Buffer, BufferBaseBinding, BufferBindTarget, DataUsage, VecBuffer},
    gl,
    shader::{Shader, ShaderProgram},
    vertex::{AttrType, VertexArrayObject},
    GlErrorGuard,
};
use glam::{IVec2, Vec2, Vec4};
use std::{ffi::CString, num::NonZeroU32};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlyphInstance {
    pos: Vec2,
    size: Vec2,
    index: u32,
}

impl GlyphInstance {
    pub const fn new(pos: Vec2, size: Vec2, index: u32) -> Self {
        Self { pos, size, index }
    }
}

impl From<BoundingBox> for GlyphInstance {
    fn from(value: BoundingBox) -> Self {
        Self::new(value.offset, value.size, 0)
    }
}

impl layout::GlyphInstance for GlyphInstance {
    fn new(pos: Vec2, size: Vec2, id: u32) -> Self {
        Self::new(pos, size, id)
    }
    fn offset_by(mut self, offset: Vec2) -> Self {
        self.pos += offset;
        self
    }
}

pub struct GpuGlyphDataBindings {
    bindings: [BufferBaseBinding; 4],
}

impl GpuGlyphDataBindings {
    pub fn new(gpu_glyph_data: &GpuGlyphData) -> Self {
        fn make_and_bind_ssbo<T>(data: &[T], binding_index: u32) -> BufferBaseBinding {
            let ssbo = Buffer::new(BufferBindTarget::ShaderStorage);
            ssbo.store_realloc(data, DataUsage::STATIC_DRAW);
            ssbo.into_base_binding(binding_index)
        }
        let bindings = [
            make_and_bind_ssbo(&gpu_glyph_data.points, 0),
            make_and_bind_ssbo(&gpu_glyph_data.verbs, 1),
            make_and_bind_ssbo(&gpu_glyph_data.glyph_starts, 2),
            make_and_bind_ssbo(&gpu_glyph_data.bounds, 3),
        ];
        Self { bindings }
    }

    pub fn bind_self_guarded(&self) {
        GlErrorGuard::guard_named("Glyph Data SSBO Binding", || self.bind_self_unguarded());
    }

    pub fn bind_self_unguarded(&self) {
        for buf in &self.bindings {
            buf.bind_self();
        }
    }
}

pub struct TextApp<'glyph> {
    ib: VecBuffer<GlyphInstance>,
    vao: VertexArrayObject,
    program: ShaderProgram,

    text_color: Vec4,

    aspect_transform: Vec2,
    glyph_data_bindings: &'glyph GpuGlyphDataBindings,
}

impl<'glyph> TextApp<'glyph> {
    pub fn new(glyph_data_bindings: &'glyph GpuGlyphDataBindings) -> Self {
        let vs = Shader::vertex(include_str!("shader/text.vert"));
        let fs = Shader::fragment(include_str!("shader/text.frag"));
        let program = ShaderProgram::new([vs, fs]);

        let vb = Buffer::new(BufferBindTarget::ArrayBuffer);
        vb.store_realloc(
            &[Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y],
            DataUsage::STATIC_DRAW,
        );

        let ib = VecBuffer::new(BufferBindTarget::ArrayBuffer, AccessNature::Draw);

        let vao = VertexArrayObject::new();
        let mut vb_attach = vao.attach_vertex_buffer(&vb, size_of::<Vec2>() as i32);
        vb_attach.add_attr(AttrType::Float, 2);

        let mut ib_attach =
            vao.attach_vertex_buffer(ib.buffer(), size_of::<GlyphInstance>() as i32);
        ib_attach.add_attr(AttrType::Float, 2);
        ib_attach.add_attr(AttrType::Float, 2);
        ib_attach.add_attr(AttrType::Uint, 1);
        ib_attach.set_instance_divisor(NonZeroU32::new(1));

        let aspect_transform = Vec2::ONE;

        Self {
            ib,
            vao,
            program,
            aspect_transform,
            glyph_data_bindings,
            text_color: Vec4::ONE,
        }
        .and_check()
    }

    fn and_check(self) -> Self {
        self.check();
        self
    }

    fn check(&self) {
        GlErrorGuard::clear_existing(Some("existing failure on check"));
        self.bind_data();
        self.program.validate();
    }

    pub fn store_data(&mut self, instances: &[GlyphInstance]) {
        self.ib.store_data(instances);
    }

    pub fn set_text_color(&mut self, color: Vec4) {
        self.text_color = color;
    }

    pub fn on_resize(&mut self, new_size: IVec2) {
        let new_size = Vec2::new(new_size.x as f32, new_size.y as f32);
        let min_bound = new_size.min_element();
        self.aspect_transform = min_bound / new_size;
    }

    fn bind_data(&self) {
        GlErrorGuard::guard_named("VAO bind", || self.vao.use_self());
        self.glyph_data_bindings.bind_self_guarded();
    }

    fn bind_program(&self) {
        GlErrorGuard::guard_named("Program bind", || self.program.use_self());
    }

    fn bind_uniform(&self) {
        GlErrorGuard::guard_named("Uniform bind", || {
            self.program.set_uniform_vec2(0, self.aspect_transform);
            self.program.set_uniform_vec4(1, self.text_color);
        });
    }

    fn draw(&self) {
        GlErrorGuard::guard_named("Draw", || unsafe {
            gl::DrawArraysInstanced(gl::TRIANGLE_FAN, 0, 4, self.ib.len() as i32)
        });
    }

    pub fn render(&self) {
        self.bind_data();
        self.bind_program();
        self.bind_uniform();
        self.draw();
    }
}
