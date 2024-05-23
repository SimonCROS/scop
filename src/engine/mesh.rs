use std::rc::Rc;

use anyhow::{Context, Result};
use ash::vk::{self, CommandBuffer};

use crate::{
    math::{Vector2, Vector3, Vector4},
    renderer::{device::RendererDevice, index_buffer::IndexBuffer, vertex_buffer::VertexBuffer},
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vector4,
    pub color: Vector4,
    pub normal: Vector3,
    pub uv: Vector2,
    // static std::vector<VkVertexInputBindingDescription> getBindingDescriptions();
    // static std::vector<VkVertexInputAttributeDescription> getAttributeDescriptions();
}

pub struct Mesh {
    device: Rc<RendererDevice>,
    vertex_buffer: VertexBuffer,
    index_buffer: Option<IndexBuffer>,
}

pub struct MeshBuilder<'a> {
    device: Rc<RendererDevice>,
    vertices: Option<&'a [Vertex]>,
    indices: Option<&'a [u32]>,
}

impl Mesh {
    pub fn builder<'a>(device: Rc<RendererDevice>) -> MeshBuilder<'a> {
        MeshBuilder {
            device,
            vertices: None,
            indices: None,
        }
    }

    pub unsafe fn bind(&self, command_buffer: CommandBuffer) {
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

    pub unsafe fn draw(&self, command_buffer: CommandBuffer) {
        if let Some(index_buffer) = &self.index_buffer {
            self.device.logical_device.cmd_draw_indexed(
                command_buffer,
                index_buffer.current_size as u32,
                1,
                0,
                0,
                0,
            );
        } else {
            self.device
                .logical_device
                .cmd_draw(command_buffer, 1, 0, 0, 0);
        }
    }

    pub unsafe fn cleanup(self) {
        let logical_device = &self.device.logical_device;

        self.index_buffer.inspect(|b| b.cleanup(logical_device));
        self.vertex_buffer.cleanup(logical_device);
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
        unsafe {
            let vertices = self
                .vertices
                .context("Cannot build a Mesh without vertices.")?;
            let mut vertex_buffer = VertexBuffer::new(&self.device)?;
            vertex_buffer.set_vertices_from_slice(&self.device.logical_device, &vertices)?;

            let index_buffer = match self.indices {
                Some(indices) => {
                    let mut index_buffer = IndexBuffer::new(&self.device)?;
                    index_buffer.set_indices_from_slice(&self.device.logical_device, indices)?;
                    Some(index_buffer)
                }
                None => None,
            };

            Ok(Mesh {
                device: self.device,
                vertex_buffer,
                index_buffer,
            })
        }
    }
}
