#![feature(lint_reasons)]

mod engine;
mod math;
mod renderer;
mod utils;

use anyhow::Result;
use engine::Engine;

fn main() -> Result<()> {
    let mut engine = Engine::new()?;
    engine.run()?;
    Ok(())
}
