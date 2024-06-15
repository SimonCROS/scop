use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

use math::{Vec2, Vec3};

use crate::engine::mesh::{Mesh, Vertex};
use crate::engine::Engine;
use crate::{bail, ensure};
use crate::utils::{Context, Result};

fn get_content_of<'a>(line: &'a String, prefix: &'static str) -> Result<Option<&'a str>> {
    if line.starts_with(prefix) {
        ensure!(line.len() >= prefix.len() + 1, "Prefix has no value"); // Prefix size + not empty
        return Ok(Some(&line[prefix.len()..]));
    }

    Ok(None)
}

pub fn read_obj_file<'a>(engine: &Engine, path: &'a str) -> Result<Rc<Mesh>> {
    let mut object_name = String::new();
    let mut vertices = Vec::<Vertex>::new();
    let mut uvs = Vec::<Vec2>::new();
    let mut normals = Vec::<Vec3>::new();
    let mut indices = Vec::<u32>::new();
    let mut indices_group: [u32; 3] = Default::default();
    let mut faces = HashMap::<(u32, u32, u32), u32>::new();
    let mut unique_vertices = Vec::<Vertex>::new();

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

        if let Some(content) = get_content_of(&line, "vn ")? {
            let mut values = content.splitn(3, ' ').map(str::parse::<f32>);

            let mut normal = Vec3::default();
            normal[0] = values.next().context("Not enough values for normal")??;
            normal[1] = values.next().context("Not enough values for normal")??;
            normal[2] = values.next().context("Not enough values for normal")??;
            ensure!(values.next().is_none(), "Too many parts in normal");
            normals.push(normal);

            continue;
        }

        if let Some(content) = get_content_of(&line, "f ")? {
            let mut indices_group_id = 0;
            for entry in content.split(' ') {
                let mut parts = entry.split('/');
                if let Some(vertex_str) = parts.next() {
                    let vertex_id = vertex_str.parse::<u32>()?;
                    let uv_id = parts
                        .next()
                        .filter(|e| !e.is_empty())
                        .map_or(Ok(0), |f| f.parse::<u32>())?;
                    let normal_id = parts
                        .next()
                        .filter(|e| !e.is_empty())
                        .map_or(Ok(0), |f| f.parse::<u32>())?;

                    if vertex_id < 1
                        || vertex_id as usize > vertices.len()
                        || uv_id as usize > uvs.len()
                        || normal_id as usize > normals.len()
                    {
                        bail!("Invalid index");
                    }

                    let key = (vertex_id, uv_id, normal_id);
                    let (unique_vertex_index, vertex) = match faces.entry(key) {
                        Entry::Occupied(o) => (*o.get(), &mut unique_vertices[*o.get() as usize]),
                        Entry::Vacant(v) => {
                            let index = unique_vertices.len();
                            unique_vertices.push(vertices[(vertex_id - 1) as usize]);
                            v.insert(index as u32);
                            (index as u32, unique_vertices.last_mut().unwrap())
                        }
                    };

                    if uv_id > 0 {
                        vertex.uv = uvs[(uv_id - 1) as usize];
                    }
                    if normal_id > 0 {
                        vertex.normal = normals[(normal_id - 1) as usize];
                    }

                    indices_group[indices_group_id] = unique_vertex_index;
                    if indices_group_id < 2 {
                        indices_group_id += 1;
                    } else {
                        indices.extend_from_slice(&indices_group);
                        indices_group[1] = indices_group[2];
                    }
                } else {
                    bail!("Invalid index");
                }
            }

            if indices_group_id < 2 {
                bail!("Not enough values for index");
            }

            continue;
        }

        if let Some(_content) = get_content_of(&line, "mtllib ")? {
            continue;
        }

        if let Some(_content) = get_content_of(&line, "usemtl ")? {
            continue;
        }

        if let Some(_content) = get_content_of(&line, "g ")? {
            continue;
        }

        if let Some(_content) = get_content_of(&line, "s ")? {
            continue;
        }

        bail!(format!("Unknown key in line `{}`", line))
    }

    Mesh::builder(engine.renderer.main_device.clone())
        .vertices(if unique_vertices.len() > 0 {
            &unique_vertices
        } else {
            &vertices
        })
        .indices(&indices)
        .build()
        .map(Rc::new)
}
