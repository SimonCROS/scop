#![feature(lint_reasons)]

mod app;
mod engine;
mod parsing;
mod renderer;
mod utils;

use std::env;

use app::{custom::AppCustom, objects::AppObjects, samourai::AppSamourai};
use utils::Result;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "samourai" {
        AppSamourai::default().start()
    } else if args.len() > 1 {
        AppCustom::default().start(args[1].as_str())
    } else {
        AppObjects::default().start()
    }
}
