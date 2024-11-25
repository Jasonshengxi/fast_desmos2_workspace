use apps::TextApp;
use color_eyre::{eyre::OptionExt, Result as EyreResult};
use fast_desmos2_fonts::glyph_data;
use fast_desmos2_gl::{
    glfw::{self, Window},
    gl,
    info::GlString,
    GlError,
};
use glam::IVec2;
use input::WindowWithInput;

mod apps;
mod input;

struct App {
    window: WindowWithInput,
    window_size: IVec2,

    text_app: TextApp,
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

        let (gpu_glyph_data, _) = glyph_data::new(include_bytes!("../../times_new_roman.ttf"))?;
        let text_app = TextApp::new(gpu_glyph_data);

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
