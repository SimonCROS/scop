mod renderer;
// mod engine;

use anyhow::Result;
use renderer::Renderer;

fn main() -> Result<()> {
    let mut renderer = Renderer::new()?;
    renderer.run()?;
    Ok(())
}
