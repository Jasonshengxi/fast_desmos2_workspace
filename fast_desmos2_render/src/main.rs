use apps::text::{TextApp, GpuGlyphDataBindings};
use color_eyre::{eyre::OptionExt, Result as EyreResult};
use fast_desmos2_fonts::{
    glyph_data::{self, CpuGlyphData},
    layout::LayoutNode,
};
use fast_desmos2_gl::{
    gl,
    glfw::{self, Window},
    info::GlString,
    GlError,
};
use fast_desmos2_utils as utils;
use glam::{IVec2, Vec2};
use input::WindowWithInput;

mod apps;
mod input;

struct App {
    window: WindowWithInput,
    window_size: IVec2,

    text_app: TextApp<'static>,
}

impl App {
    fn run(width: i32, height: i32, title: &str) -> EyreResult<()> {
        glfw::install_errors();
        glfw::init().ok_or_eyre("glfw init failed")?;

        let window = Window::create(width, height, title)?;
        window.make_current();

        let window = WindowWithInput::new(window);

        gl::load_with(glfw::get_proc_address);

        let renderer = GlString::Renderer.get_gl();
        let version = GlString::Version.get_gl();
        println!("renderer: {renderer}");
        println!("version: {version}");

        let (gpu_glyph_data, cpu_glyph_data) = glyph_data::new(include_bytes!("../../cmunrm.ttf"))?;
        let glyph_bindings = utils::leak(GpuGlyphDataBindings::new(&gpu_glyph_data));
        let mut text_app = TextApp::new(glyph_bindings);

        let layout = LayoutNode::vertical(vec![
            LayoutNode::horizontal(vec![
                LayoutNode::str("Hello world!"),
                LayoutNode::char(CpuGlyphData::RECT_CHAR),
                LayoutNode::str("This is another."),
            ]),
            LayoutNode::horizontal(vec![LayoutNode::str("Bottom row.")]),
        ]);

        let mut inst_tree = layout.render(&cpu_glyph_data, 0.2).into_instances();
        inst_tree.offset.x -= 1.0;

        // println!("Tree: {inst_tree:#?}");

        let instances = inst_tree.collect_vec();
        text_app.store_data(&instances);

        Self {
            window,
            window_size: IVec2::ONE,
            text_app,
        }
        .run_internal();

        Ok(())
    }

    fn run_internal(mut self) {
        while !self.window.should_close() {
            self.tick();
        }
    }

    fn tick(&mut self) {
        if let Some(err) = GlError::try_get() {
            println!("error: {err:?}");
        }

        let new_size = self.window.get_framebuffer_size();
        if new_size != self.window_size {
            unsafe { gl::Viewport(0, 0, new_size.x, new_size.y) };

            self.text_app.on_resize(new_size);

            self.window_size = new_size;
        }

        self.window.swap_buffers();
        glfw::poll_events();

        self.text_app.render();
    }
}

fn main() -> EyreResult<()> {
    color_eyre::install()?;
    App::run(1000, 800, "window")
}
