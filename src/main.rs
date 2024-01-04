mod renderer;

use anyhow::Result;
use renderer::Renderer;
use window::RendererWindow;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::keyboard::{Key, NamedKey};

fn main() -> Result<()> {
    let (event_loop, window) = RendererWindow::create_window()?;
    let used_extensions = ash_window::enumerate_required_extensions(window.raw_display_handle())?;

    // let mut renderer = unsafe { Renderer::new() }?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            event:
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                    ..
                },
            ..
        } => elwt.exit(),
        _ => (),
    })?;
    unsafe {
        app.destroy();
    }
    Ok(())
}
