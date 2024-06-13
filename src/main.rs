#![feature(lint_reasons)]

mod app;
mod engine;
mod parsing;
mod renderer;
mod utils;

use std::env;

use anyhow::Result;
use app::{objects::AppObjects, samourai::AppSamourai};
use engine::Engine;

fn main() -> Result<()> {
    let mut engine = Engine::new()?;

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "samourai" {
        AppSamourai::default().start(&mut engine)
    } else {
        AppObjects::default().start(&mut engine)
    }
}
