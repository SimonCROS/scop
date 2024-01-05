mod renderer;

use anyhow::Result;
use renderer::window::RendererWindow;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::keyboard::{Key, NamedKey};

fn main() -> Result<()> {
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
        // TODO cleanup
    }
    Ok(())
}
