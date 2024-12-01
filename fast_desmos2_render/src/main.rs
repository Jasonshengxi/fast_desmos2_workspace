use apps::text::{GlyphInstance, GpuGlyphDataBindings, TextApp};
use color_eyre::{eyre::OptionExt, Result as EyreResult};
use fast_desmos2_fonts::{
    glyph_data::{self, BoundingBox, CpuGlyphData},
    layout::{InstTree, LayoutNode},
};
use fast_desmos2_gl::{
    gl,
    glfw::{self, Window},
    info::GlString,
    GlError,
};
use fast_desmos2_utils as utils;
use glam::{IVec2, Vec2, Vec4};
use input::WindowWithInput;
use okhsl::Okhsv;

mod apps;
mod input;

const DEBUG_LAYERS: usize = 5;
struct App {
    window: WindowWithInput,
    window_size: IVec2,

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
        let mut text_app = TextApp::new(glyph_bindings);

        let layout = LayoutNode::sandwich_vertical(
            LayoutNode::horizontal(vec![
                LayoutNode::str("x"),
                LayoutNode::char('+'),
                LayoutNode::str("cos(y)"),
            ]),
            0.1,
            LayoutNode::str("128"),
        );
        // let layout = LayoutNode::sandwich_vertical(LayoutNode::str("a+b=2"), ('i', 1.0), LayoutNode::str("sin(x)"));

        let mut inst_tree1 = layout.render(&cpu_glyph_data, 0.5).into_instances();
        inst_tree1.offset.x -= 0.5;
        // let mut inst_tree2 = layout.render(&cpu_glyph_data, 1.0).into_instances();
        // inst_tree2.offset.x += 0.5;
        // let inst_tree = InstTree::new_children(
        //     BoundingBox {
        //         offset: Vec2::new(-0.5, 0.0),
        //         size: Vec2::new(1.0, 0.1),
        //     },
        //     vec![inst_tree1, inst_tree2],
        // );
        let inst_tree = inst_tree1;

        // println!("Tree: {inst_tree:#?}");

        let (instances, bboxes) = inst_tree.collect_vec_debug();
        text_app.store_data(&instances);

        let mut debug_boxes = [(); DEBUG_LAYERS].map(|_| TextApp::new(glyph_bindings));
        let mut vec_bboxes = [const { Vec::new() }; DEBUG_LAYERS];
        for (index, bbox) in bboxes {
            vec_bboxes[index].push(GlyphInstance::from(bbox));
        }

        for (index, debug_boxes) in debug_boxes.iter_mut().enumerate() {
            let percentage = index as f64 / ((DEBUG_LAYERS - 1) as f64);
            let okhsl::Rgb { r, g, b } = okhsl::oklab_to_linear_srgb(
                okhsl::Okhsv {
                    h: 0.4 + 0.1 * percentage,
                    s: 1.0,
                    v: 0.7 - 0.2 * percentage as f32,
                }
                .to_oklab(),
            );
            debug_boxes.set_text_color(Vec4::new(r, g, b, 1.0));
        }

        for (bbox, box_drawer) in vec_bboxes.iter().zip(debug_boxes.iter_mut()) {
            box_drawer.store_data(bbox);
        }

        Self {
            window,
            window_size: IVec2::ONE,
            text_app,
            debug_boxes,
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

            self.debug_boxes
                .iter_mut()
                .for_each(|x| x.on_resize(new_size));
            self.text_app.on_resize(new_size);

            self.window_size = new_size;
        }

        self.window.swap_buffers();
        glfw::poll_events();

        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        self.debug_boxes.iter().for_each(|x| x.render());
        self.text_app.render();
    }
}

fn main() -> EyreResult<()> {
    color_eyre::install()?;
    App::run(1000, 800, "window")
}
