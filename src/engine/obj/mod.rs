use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

use anyhow::{ensure, Context, Result};

use crate::engine::mesh::Vertex;
use crate::renderer::RendererDevice;

use super::mesh::Mesh;

pub fn read_obj_file(device: Rc<RendererDevice>, path: &'static str) -> Result<Mesh> {
    let mut object_name = "";
    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u32>::new();
    let mut indices_group: [u32; 3] = Default::default();

    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    for line in buf_reader.lines() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("v ") {
            ensure!(line.len() >= 2);

            let values = &line[2..];
            let mut values = values.splitn(3, ' ').map(str::parse::<f32>);

            let mut vert = Vertex::default();
            vert.position[0] = values.next().context("Not enough values for vertex")??;
            vert.position[1] = values.next().context("Not enough values for vertex")??;
            vert.position[2] = values.next().context("Not enough values for vertex")??;
            ensure!(values.next().is_none(), "Too many parts in vertex");
            vertices.push(vert);

            continue;
        }

        if line.starts_with("f ") {
            ensure!(line.len() >= 2);

            let values = &line[2..];
            let mut values = values.split(' ').map(str::parse::<u32>);

            indices_group[0] = values.next().context("Not enough values for index")?? - 1;
            indices_group[1] = values.next().context("Not enough values for index")?? - 1;
            indices_group[2] = values.next().context("Not enough values for index")?? - 1;
            ensure!((indices_group[0] as usize) < vertices.len(), "Index too big");
            ensure!((indices_group[1] as usize) < vertices.len(), "Index too big");
            ensure!((indices_group[2] as usize) < vertices.len(), "Index too big");
            indices.extend_from_slice(&indices_group);
            for index in values {
                indices_group[1] = indices_group[2];
                indices_group[2] = index? - 1;
                ensure!((indices_group[2] as usize) <= vertices.len(), "Index too big");
                indices.extend_from_slice(&indices_group);
            }

            continue;
        }
    }

    Mesh::builder(device)
        .vertices(&vertices)
        .indices(&indices)
        .build()
}
