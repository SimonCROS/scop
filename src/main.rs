#![feature(lint_reasons)]

mod renderer;
mod engine;

use anyhow::Result;
use matrix::{Matrix, Vector};
use renderer::Renderer;

pub type Matrix4 = Matrix<4, 4, f32>;
pub type Vector2 = Vector<3, f32>;
pub type Vector3 = Vector<3, f32>;

fn main() -> Result<()> {
    let mut renderer = Renderer::new()?;
    renderer.run()?;
    Ok(())
}
