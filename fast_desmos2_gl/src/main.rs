// use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::Window};
use color_eyre::{eyre::OptionExt, Result as EyreResult};
use fast_desmos2_gl::glfw::{self, Window};
//
// struct RenderApp {
//
// }
//
// impl RenderApp {
//     fn new(event_loop: &ActiveEventLoop) -> EyreResult<Self> {
//         let window = event_loop.create_window(Window::default_attributes())?;
//         Ok(RenderApp {
//
//         })
//     }
// }
//
// #[derive(Default)]
// struct App {
//     app: Option<RenderApp>
// }
//
// impl ApplicationHandler for App {
//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         self.app = Some(RenderApp::new(event_loop).unwrap());
//     }
//
//     fn window_event(
//             &mut self,
//             event_loop: &ActiveEventLoop,
//             window_id: winit::window::WindowId,
//             event: WindowEvent,
//         ) {
//     }
// }
//
// fn main() -> EyreResult<()> {
//     let event_loop = EventLoop::builder().build()?;
//     event_loop.run_app(&mut App::default())?;
//     Ok(())
// }

fn main() -> EyreResult<()> {
    glfw::install_errors();
    glfw::init().ok_or_eyre("glfw init failed")?;

    let window = Window::create(1000, 800, "AHHH")?;

    while !window.should_close() {
        glfw::poll_events();
    }

    Ok(())
}
