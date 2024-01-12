mod renderer;

use anyhow::Result;
use renderer::Renderer;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::keyboard::{Key, NamedKey};

fn main() -> Result<()> {
    let mut renderer = unsafe { Renderer::new() }?;

    unsafe {
        // TODO cleanup
    }
    Ok(())
}
