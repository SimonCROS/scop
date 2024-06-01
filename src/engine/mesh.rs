use std::{
    mem::{self, offset_of, size_of},
    rc::Rc,
};

use anyhow::{Context, Ok, Result};
use ash::vk::{
    self, BufferUsageFlags, CommandBuffer, MemoryPropertyFlags, VertexInputAttributeDescription,
    VertexInputBindingDescription, WHOLE_SIZE,
};

use crate::{
    math::{Vector2, Vector3},
    renderer::{device::RendererDevice, scop_buffer::ScopBuffer},
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vector3,
    pub color: Vector3,
    pub normal: Vector3,
    pub uv: Vector2,
}

pub struct Mesh {
    device: Rc<RendererDevice>,
    vertex_buffer: ScopBuffer,
    index_buffer: Option<ScopBuffer>,
}

pub struct MeshBuilder<'a> {
    device: Rc<RendererDevice>,
    vertices: Option<&'a [Vertex]>,
    indices: Option<&'a [u32]>,
}

impl Vertex {
    pub fn get_vertex_input_attribute_descriptions() -> Vec<VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, uv) as u32,
            },
        ]
    }

    pub fn get_vertex_input_binding_descriptions() -> Vec<VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }
}

impl Mesh {
    pub fn builder<'a>(device: Rc<RendererDevice>) -> MeshBuilder<'a> {
        MeshBuilder {
            device,
            vertices: None,
            indices: None,
        }
    }

    pub fn bind(&self, command_buffer: CommandBuffer) {
        unsafe {
            self.device.logical_device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &[self.vertex_buffer.buffer],
                &[0],
            );

            if let Some(index_buffer) = &self.index_buffer {
                self.device.logical_device.cmd_bind_index_buffer(
                    command_buffer,
                    index_buffer.buffer,
                    0,
                    vk::IndexType::UINT32,
                );
            }
        }
    }

    pub fn draw(&self, command_buffer: CommandBuffer) {
        unsafe {
            if let Some(index_buffer) = &self.index_buffer {
                self.device.logical_device.cmd_draw_indexed(
                    command_buffer,
                    index_buffer.instance_count as u32,
                    1,
                    0,
                    0,
                    0,
                );
            } else {
                self.device.logical_device.cmd_draw(
                    command_buffer,
                    self.vertex_buffer.instance_count as u32,
                    1,
                    0,
                    0,
                );
            }
        }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        if let Some(index_buffer) = &mut self.index_buffer {
            index_buffer.cleanup();
        }
        self.vertex_buffer.cleanup();
    }
}

impl<'a> MeshBuilder<'a> {
    pub fn vertices(mut self, vertices: &'a [Vertex]) -> Self {
        self.vertices = Some(vertices);
        self
    }

    pub fn indices(mut self, indices: &'a [u32]) -> Self {
        self.indices = Some(indices);
        self
    }

    pub fn build(self) -> Result<Mesh> {
        let vertices = self
            .vertices
            .context("Cannot build a Mesh without vertices.")?;
        let vertices_count = vertices.len();
        let mut vertex_buffer = ScopBuffer::new(
            self.device.clone(),
            vertices_count,
            size_of::<Vertex>() as vk::DeviceSize,
            BufferUsageFlags::VERTEX_BUFFER,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            1,
        )?;
        vertex_buffer.map(WHOLE_SIZE, 0)?;
        vertex_buffer.write_to_buffer(&vertices, 0);
        vertex_buffer.unmap();

        let index_buffer = self.indices.map_or(Ok(None), |indices| {
            let indices_count = indices.len();
            let mut index_buffer = ScopBuffer::new(
                self.device.clone(),
                indices_count,
                size_of::<u32>() as vk::DeviceSize,
                BufferUsageFlags::INDEX_BUFFER,
                MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
                1,
            )?;
            index_buffer.map(WHOLE_SIZE, 0)?;
            index_buffer.write_to_buffer(&indices, 0);
            index_buffer.unmap();

            Ok(Some(index_buffer))
        })?;

        Ok(Mesh {
            device: self.device,
            vertex_buffer,
            index_buffer,
        })
    }
}
