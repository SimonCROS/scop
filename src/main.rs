mod renderer;

use anyhow::Result;
use renderer::Renderer;

fn main() -> Result<()> {
    let mut renderer = Renderer::new()?;
    renderer.run()?;

    unsafe {
        // TODO cleanup
    }
    Ok(())
}
