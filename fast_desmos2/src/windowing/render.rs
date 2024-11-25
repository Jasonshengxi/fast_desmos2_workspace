use crate::text::glyph_data;
use crate::text::render::AstNodeRenderContext;
use color_eyre::Result as EyreResult;
use fast_desmos2_parser::{self as parser, IdentStorer};
use fast_desmos2_utils::OptExt;
use glam::Vec2;
use glium::{backend::glutin::SimpleWindowBuilder, glutin::surface::WindowSurface, Display};
use text::{ExprRender, GlyphInstance};
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::Window};

use super::input::InputTracker;
mod dyn_buf;
mod text;

pub struct RenderApp {
    window: Window,
    display: Display<WindowSurface>,

    expr: ExprRender,
}

impl RenderApp {
    pub fn new(event_loop: &ActiveEventLoop) -> EyreResult<Self> {
        let (window, display) = SimpleWindowBuilder::new()
            .with_title("fast_desmos2")
            .build(event_loop);

        // expr.write_instances(&[GlyphInstance {
        //     position: Vec2::ZERO,
        //     size: Vec2::splat(0.1),
        //     index: 0,
        // }]);
        let (gpu_glyph_data, cpu_glyph_data) =
            glyph_data::new_maybe_gpu(Some(&display), include_bytes!("../../JetBrainsMono.ttf"))?;
        let mut expr = ExprRender::new(&display, gpu_glyph_data.unwrap_unreach());
        let source = r#"(\frac{1+2-3}{x+y})"#;
        let id_storer = IdentStorer::default();
        let parsed = parser::parse(&id_storer, source).unwrap();
        let mut ctx = AstNodeRenderContext::new(&cpu_glyph_data);
        let ast_node = parsed.borrow_dependent().as_ref().unwrap();
        let node = ctx.render(source, ast_node);
        let instances: Vec<_> = node.into_instances().into_iter().map(From::from).collect();
        expr.write_instances(&instances);

        Ok(Self {
            window,
            display,
            expr,
        })
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn render(&mut self) {
        self.expr.render(&self.display)
    }

    pub fn tick(&mut self, input: &InputTracker) {}

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {}
}
