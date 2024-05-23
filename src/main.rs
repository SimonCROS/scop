#![feature(lint_reasons)]

mod engine;
mod math;
mod renderer;

use std::cell::Cell;

use anyhow::Result;
use engine::{Engine, GameObject};

struct AAA<'a> {
    pub aaa: &'a mut Cell<GameObject>,
}

fn main() -> Result<()> {
    let mut engine = Engine::new()?;
    engine.run()?;
    Ok(())
}
