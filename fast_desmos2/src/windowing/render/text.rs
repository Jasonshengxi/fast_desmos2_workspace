use std::borrow::Cow;

use glam::Vec2;
use glium::{
    glutin::surface::WindowSurface,
    index::{NoIndices, PrimitiveType},
    uniforms::EmptyUniforms,
    vertex::AttributeType,
    Display, DrawParameters, Program, Surface, Vertex, VertexBuffer,
};

use crate::{text::glyph_data::GpuGlyphData, windowing::render_old};

impl From<render_old::text::GlyphInstance> for GlyphInstance {
    fn from(value: render_old::text::GlyphInstance) -> Self {
        Self {
            position: value.position,
            size: value.size,
            index: value.index,
        }
    }
}

#[derive(Clone, Copy)]
pub struct GlyphInstance {
    pub position: Vec2,
    pub size: Vec2,
    pub index: u32,
}

impl Vertex for GlyphInstance {
    fn build_bindings() -> glium::VertexFormat {
        &[
            (
                Cow::Borrowed("inst_pos"),
                0,
                1,
                AttributeType::F32F32,
                false,
            ),
            (Cow::Borrowed("size"), 8, 2, AttributeType::F32F32, false),
            (
                Cow::Borrowed("glyph_index"),
                16,
                3,
                AttributeType::U32,
                false,
            ),
        ]
    }
}

#[derive(Clone, Copy)]
pub struct ExprVertex {
    position: Vec2,
}

impl ExprVertex {
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
        }
    }
}

impl Vertex for ExprVertex {
    fn build_bindings() -> glium::VertexFormat {
        &[(
            Cow::Borrowed("position"),
            0,
            0,
            AttributeType::F32F32,
            false,
        )]
    }
}

pub struct ExprRender {
    glyph_data: GpuGlyphData,

    program: Program,
    draw_params: DrawParameters<'static>,
    vertex_buffer: VertexBuffer<ExprVertex>,

    instance_buffer: VertexBuffer<GlyphInstance>,
}

impl ExprRender {
    pub fn new(display: &Display<WindowSurface>, glyph_data: GpuGlyphData) -> Self {
        let program = match Program::from_source(
            display,
            include_str!("text.vert"),
            include_str!("text.frag"),
            None,
        ) {
            Ok(x) => x,
            Err(glium::ProgramCreationError::CompilationError(error, shader_type)) => {
                println!("shader type: {shader_type:?}");
                println!("{error}");
                panic!("Shader compilation failed!");
            }
            Err(err) => {
                println!("{err}");
                panic!("Shader creation failed!");
            }
        };

        // println!("Attributes:");
        // for (name, attr) in program.attributes() {
        //     println!(" - {name}: {attr:?}");
        // }

        let draw_params = DrawParameters {
            ..Default::default()
        };

        let vertex_buffer = VertexBuffer::new(
            display,
            &[
                ExprVertex::new(0., 0.),
                ExprVertex::new(1., 0.),
                ExprVertex::new(0., 1.),
                ExprVertex::new(1., 1.),
            ],
        )
        .unwrap();
        let instance_buffer = VertexBuffer::empty_dynamic(display, 10).unwrap();

        Self {
            program,
            draw_params,
            vertex_buffer,
            instance_buffer,
            glyph_data,
        }
    }

    pub fn write_instances(&mut self, data: &[GlyphInstance]) {
        self.instance_buffer.write(data);
    }

    pub fn render(&mut self, display: &Display<WindowSurface>) {
        let mut frame = display.draw();

        let vb = (
            &self.vertex_buffer,
            self.instance_buffer.per_instance().unwrap(),
        );

        let glyph_data_uniform = self.glyph_data.as_uniforms();
        frame
            .draw(
                vb,
                NoIndices(PrimitiveType::TriangleStrip),
                &self.program,
                &glyph_data_uniform,
                &self.draw_params,
            )
            .unwrap();

        frame.finish().unwrap();
    }
}
