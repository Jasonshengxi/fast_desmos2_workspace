#![allow(unused)]

use std::ops::{Deref, DerefMut};

use apps::text::{GlyphInstance, GpuGlyphDataBindings, TextApp};
use color_eyre::{eyre::OptionExt as _, Result as EyreResult};
use editor::WithLayout;
use fast_desmos2_fonts::{
    glyph_data::{self, CpuGlyphData},
    layout::{InstTree, LayoutNode},
};
use fast_desmos2_gl::{
    gl,
    glfw::{self, Window},
    info::GlString,
    GlError,
};
use fast_desmos2_tree::tree::{EditorTree, EditorTreeSeq, EditorTreeSeqNormal, TreeAction};
use fast_desmos2_utils as utils;
use glam::{IVec2, Vec4};
use input::{KeyExt as _, WindowWithInput};

mod apps;
mod editor;
mod input;

struct Mutated<T> {
    inner: T,
    mutated: bool,
}

impl<T> Mutated<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            mutated: false,
        }
    }

    pub fn poll_mutated(&mut self) -> bool {
        if self.mutated {
            self.mutated = false;
            true
        } else {
            false
        }
    }
}

impl<T> Deref for Mutated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Mutated<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mutated = true;
        &mut self.inner
    }
}

const DEBUG_LAYERS: usize = 5;
struct App {
    window: WindowWithInput,
    window_size: IVec2,

    cpu_glyph_data: CpuGlyphData,
    editor: Mutated<EditorTreeSeqNormal>,

    debug_boxes: [TextApp<'static>; DEBUG_LAYERS],
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
        Self {
            window,
            window_size: IVec2::ONE,
            cpu_glyph_data,
            editor: Mutated::new(EditorTreeSeqNormal::empty()),
            text_app: TextApp::new(glyph_bindings),
            debug_boxes: [(); DEBUG_LAYERS].map(|_| TextApp::new(glyph_bindings)),
        }
        .run_internal();

        Ok(())
    }

    fn run_internal(mut self) {
        while !self.window.should_close() {
            self.tick();
        }
    }

    fn write_text(&mut self, inst_tree: InstTree<GlyphInstance>) {
        let (instances, bboxes) = inst_tree.collect_vec_debug();
        self.text_app.store_data(&instances);

        let mut vec_bboxes = [const { Vec::new() }; DEBUG_LAYERS];
        for (index, bbox) in bboxes {
            vec_bboxes
                .get_mut(index)
                .expect("Not enough debug layers!")
                .push(GlyphInstance::from(bbox));
        }

        for (index, debug_boxes) in self.debug_boxes.iter_mut().enumerate() {
            let percentage = index as f64 / ((DEBUG_LAYERS - 1) as f64);
            let okhsl::Rgb { r, g, b } = okhsl::oklab_to_linear_srgb(
                okhsl::Okhsv {
                    h: 0.4 + 0.3 * percentage,
                    s: 1.0,
                    v: 0.9 - 0.2 * percentage as f32,
                }
                .to_oklab(),
            );
            debug_boxes.set_text_color(Vec4::new(r, g, b, 1.0));
        }

        for (bbox, box_drawer) in vec_bboxes.iter().zip(self.debug_boxes.iter_mut()) {
            box_drawer.store_data(bbox);
        }
    }

    fn check_and_resize(&mut self) {
        let new_size = self.window.get_framebuffer_size();
        if new_size != self.window_size {
            unsafe { gl::Viewport(0, 0, new_size.x, new_size.y) };

            self.debug_boxes
                .iter_mut()
                .for_each(|x| x.on_resize(new_size));
            self.text_app.on_resize(new_size);

            self.window_size = new_size;
        }
    }

    fn tick(&mut self) {
        if let Some(err) = GlError::try_get() {
            println!("error: {err:?}");
        }

        for &key in self.window.keys_pressed().iter() {
            match key {
                glfw::Key::Backspace => {
                    self.editor.apply_action(TreeAction::Delete);
                }
                _ => {
                    if let Some(ch) = key.as_char() {
                        self.editor.apply_action(TreeAction::from_char(ch));
                    }
                }
            }
        }

        if self.editor.poll_mutated() {
            let layout = self.editor.layout();
            let render = layout.render(&self.cpu_glyph_data, 0.5);
            let mut instances = render.into_instances();
            instances.offset.x -= 1.0;
            self.write_text(instances);
        }

        self.check_and_resize();

        self.window.swap_buffers();
        self.window.clear_frame_specific();
        glfw::poll_events();

        if !self.window.is_key_down(glfw::Key::Space) {
            self.debug_boxes.iter().for_each(|x| x.render());
        }

        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        self.text_app.render();
    }
}

fn main() -> EyreResult<()> {
    color_eyre::install()?;
    App::run(1000, 800, "window")
}
