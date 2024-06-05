use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

use anyhow::{ensure, Context, Result};

use crate::engine::mesh::{Mesh, Vertex};
use crate::renderer::RendererDevice;

fn get_content_of<'a>(line: &'a String, prefix: &'static str) -> Result<Option<&'a str>> {
    if line.starts_with(prefix) {
        ensure!(line.len() >= prefix.len() + 1); // Prefix size + not empty
        return Ok(Some(&line[prefix.len()..]));
    }

    Ok(None)
}

pub fn read_mtl_file(device: Rc<RendererDevice>, path: &'static str) -> Result<Mesh> {
    unimplemented!()
}
