use std::{cell::Cell, num::NonZeroU32};

use color_eyre::{eyre::OptionExt, Result as EyreResult};
use fast_desmos2_gl::{
    buffer::{Buffer, BufferBindTarget, DataUsage},
    glfw::{self, Window},
    shader::{Shader, ShaderProgram, ShaderType},
    vertex::{AttrType, VertexArrayObject},
};
use fast_desmos2_utils as utils;
use glam::{DVec2, Vec2};

fn main() -> EyreResult<()> {
    const INSTANCE_COUNT_WANT: usize = 10_000_000;
    const WORKGROUP_SIZE: usize = 128;
    const WORKGROUP_COUNT: usize = INSTANCE_COUNT_WANT / WORKGROUP_SIZE;
    const INSTANCE_COUNT: usize = WORKGROUP_COUNT * WORKGROUP_SIZE;

    println!("Starting program");
    println!("Actual instance count: {INSTANCE_COUNT}");

    glfw::install_errors();
    glfw::init().ok_or_eyre("glfw init failed")?;

    println!("Creating window");
    const SIZE: i32 = 1920;
    let window = Window::create(SIZE, SIZE, "AHHH")?;
    println!("Window created");
    window.make_current();
    println!("Now current");

    let scroll = utils::leak(Cell::new(DVec2::ZERO));
    window.install_scroll_callback(|this_scroll| scroll.set(scroll.get() + this_scroll));

    gl::load_with(glfw::get_proc_address);
    println!("GL initialized");

    unsafe {
        ::gl::Enable(::gl::BLEND);
        ::gl::BlendFunc(::gl::SRC_ALPHA, ::gl::ONE_MINUS_SRC_ALPHA);
    }

    let fs = Shader::new(ShaderType::Fragment, include_str!("main/main.frag"));
    let vs = Shader::new(ShaderType::Vertex, include_str!("main/main.vert"));
    let render_program = ShaderProgram::new([fs, vs]);

    let cs = Shader::new(ShaderType::Compute, include_str!("main/main.comp"));
    let compute_program = ShaderProgram::new([cs]);

    let cs = Shader::new(ShaderType::Compute, include_str!("main/init.comp"));
    let init_program = ShaderProgram::new([cs]);

    let vertex_buffer = Buffer::new(BufferBindTarget::ArrayBuffer);
    vertex_buffer.store_realloc(
        &[Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y],
        DataUsage::STATIC_DRAW,
    );

    let data = Buffer::new(BufferBindTarget::ShaderStorage);
    data.realloc::<Vec2>(2 * INSTANCE_COUNT, DataUsage::DYNAMIC_DRAW);

    init_program.use_self();
    data.bind_base_directly(0);
    unsafe { ::gl::DispatchCompute(WORKGROUP_COUNT as u32, 1, 1) };
    unsafe { ::gl::MemoryBarrier(::gl::SHADER_STORAGE_BARRIER_BIT) };

    let empty_data = Buffer::new(BufferBindTarget::ShaderStorage);
    empty_data.realloc::<Vec2>(2 * INSTANCE_COUNT, DataUsage::DYNAMIC_DRAW);

    let vao1 = VertexArrayObject::new();
    let mut vb_attach = vao1.attach_vertex_buffer(&vertex_buffer, size_of::<Vec2>() as i32);
    vb_attach.add_attr(AttrType::Float, 2);
    let mut ib_attach = vao1.attach_vertex_buffer(&data, size_of::<Vec2>() as i32 * 2);
    ib_attach.set_instance_divisor(NonZeroU32::new(1));
    ib_attach.add_attr(AttrType::Float, 2);

    let vao2 = VertexArrayObject::new();
    let mut vb_attach = vao2.attach_vertex_buffer(&vertex_buffer, size_of::<Vec2>() as i32);
    vb_attach.add_attr(AttrType::Float, 2);
    let mut ib_attach = vao2.attach_vertex_buffer(&empty_data, size_of::<Vec2>() as i32 * 2);
    ib_attach.set_instance_divisor(NonZeroU32::new(1));
    ib_attach.add_attr(AttrType::Float, 2);

    struct Things {
        ssbo: Buffer,
        vao: VertexArrayObject,
    }

    let mut present_buffer = Things {
        ssbo: data,
        vao: vao1,
    };
    let mut next_buffer = Things {
        ssbo: empty_data,
        vao: vao2,
    };

    let mut scale = 1.0;
    let mut shift = Vec2::ZERO;
    let mut running = false;
    let mut last_mouse = DVec2::ZERO;

    while !window.should_close() {
        glfw::poll_events();

        let this_scroll = scroll.get();
        scale *= (1.1f32).powf(this_scroll.y as f32);
        scroll.set(DVec2::ZERO);

        let this_mouse = window.get_mouse_pos();
        let mouse_delta = this_mouse - last_mouse;
        if window.is_mouse_down(glfw::MouseButton::Button2) {
            let window_size = window.get_framebuffer_size().as_vec2() * Vec2::new(1.0, -1.0);
            shift += 2.0 * mouse_delta.as_vec2() / (window_size * scale);
        }
        last_mouse = this_mouse;

        if window.is_key_down(glfw::Key::A) {
            running = true;
        }
        if window.is_key_down(glfw::Key::C) {
            running = false;
        }

        if running {
            compute_program.use_self();
            compute_program.set_uniform_f32(0, 0.002); // dt

            for _ in 0..8 {
                present_buffer.ssbo.bind_base_directly(0);
                next_buffer.ssbo.bind_base_directly(1);
                unsafe { ::gl::DispatchCompute(WORKGROUP_COUNT as u32, 1, 1) };
                unsafe { ::gl::MemoryBarrier(::gl::SHADER_STORAGE_BARRIER_BIT) };
                std::mem::swap(&mut next_buffer, &mut present_buffer);
            }
        }

        unsafe { ::gl::Clear(::gl::COLOR_BUFFER_BIT) };
        render_program.use_self();
        present_buffer.vao.use_self();
        render_program.set_uniform_f32(0, scale); // scale
        render_program.set_uniform_f32(1, 0.001); // size
        render_program.set_uniform_vec2(2, shift); // shift
                                                   // unsafe { ::gl::DrawArraysInstanced(::gl::POINTS, 0, 1, INSTANCE_COUNT as i32) };
        unsafe { ::gl::DrawArraysInstanced(::gl::TRIANGLE_FAN, 0, 4, INSTANCE_COUNT as i32) };

        window.swap_buffers();
    }

    Ok(())
}
