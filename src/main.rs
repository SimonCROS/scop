#![feature(lint_reasons)]

use anyhow::Result;
use engine::Engine;

mod engine;
mod parsing;
mod renderer;
mod utils;
mod app;

fn main() -> Result<()> {
    let mut engine = Engine::new()?;
    engine.run()?;
    Ok(())
}
