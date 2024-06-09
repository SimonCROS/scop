#![feature(lint_reasons)]

mod app;
mod engine;
mod parsing;
mod renderer;
mod utils;

use anyhow::Result;
use app::App;
use engine::Engine;

fn main() -> Result<()> {
    let mut engine = Engine::new()?;
    App::default().start(&mut engine)
}
