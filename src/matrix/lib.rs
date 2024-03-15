#![feature(trait_alias)]

mod matrix;
mod vector;

pub mod traits;
pub mod common;

pub use matrix::Matrix;
pub use vector::Vector;
