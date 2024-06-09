use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

use anyhow::{ensure, Context, Result};
use math::Vec2;

use crate::engine::mesh::{Mesh, Vertex};
use crate::engine::Engine;

fn get_content_of<'a>(line: &'a String, prefix: &'static str) -> Result<Option<&'a str>> {
    if line.starts_with(prefix) {
        ensure!(line.len() >= prefix.len() + 1); // Prefix size + not empty
        return Ok(Some(&line[prefix.len()..]));
    }

    Ok(None)
}

pub fn read_obj_file(engine: &Engine, path: &'static str) -> Result<Rc<Mesh>> {
    let mut object_name = String::new();
    let mut vertices = Vec::<Vertex>::new();
    let mut uvs = Vec::<Vec2>::new();
    let mut indices = Vec::<u32>::new();
    let mut indices_group: [u32; 3] = Default::default();

    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    for line in buf_reader.lines() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(content) = get_content_of(&line, "o ")? {
            ensure!(object_name.is_empty(), "Only one object allowed");
            object_name = String::from(content);

            continue;
        }

        if let Some(content) = get_content_of(&line, "v ")? {
            let mut values = content.splitn(3, ' ').map(str::parse::<f32>);

            let mut vert = Vertex::default();
            vert.position[0] = values.next().context("Not enough values for vertex")??;
            vert.position[1] = values.next().context("Not enough values for vertex")??;
            vert.position[2] = values.next().context("Not enough values for vertex")??;
            vert.uv[0] = vert.position[2];
            vert.uv[1] = vert.position[1];
            ensure!(values.next().is_none(), "Too many parts in vertex");
            vertices.push(vert);

            continue;
        }

        if let Some(content) = get_content_of(&line, "vt ")? {
            let mut values = content.splitn(3, ' ').map(str::parse::<f32>);

            let mut uv = Vec2::default();
            uv[0] = values.next().context("Not enough values for uv")??;
            uv[1] = values.next().context("Not enough values for uv")??;
            ensure!(values.next().is_none(), "Too many parts in uv");
            uvs.push(uv);

            continue;
        }

        if let Some(content) = get_content_of(&line, "f ")? {
            let mut values = content.split(' ').map(str::parse::<u32>);
//content.splitn(3, ' ').map(|p| p.split('/').map(str::parse::<f32>));
            indices_group[0] = values.next().context("Not enough values for index")?? - 1;
            indices_group[1] = values.next().context("Not enough values for index")?? - 1;
            indices_group[2] = values.next().context("Not enough values for index")?? - 1;
            ensure!(
                (indices_group[0] as usize) < vertices.len(),
                "Index too big"
            );
            ensure!(
                (indices_group[1] as usize) < vertices.len(),
                "Index too big"
            );
            ensure!(
                (indices_group[2] as usize) < vertices.len(),
                "Index too big"
            );
            indices.extend_from_slice(&indices_group);
            for index in values {
                indices_group[1] = indices_group[2];
                indices_group[2] = index? - 1;
                ensure!(
                    (indices_group[2] as usize) <= vertices.len(),
                    "Index too big"
                );
                indices.extend_from_slice(&indices_group);
            }

            continue;
        }

        if let Some(content) = get_content_of(&line, "mtllib ")? {
            continue;
        }

        if let Some(content) = get_content_of(&line, "usemtl ")? {
            continue;
        }
    }

    Mesh::builder(engine.renderer.main_device.clone())
        .vertices(&vertices)
        .indices(&indices)
        .build()
        .map(Rc::new)
}
