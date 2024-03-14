use std::mem;

use anyhow::{Context, Result};
use ash::{util::Align, vk, Device};

use super::device::RendererDevice;

const VERTEX_BUFFER_SIZE: vk::DeviceSize = 1024 * 1024 * 10; // 10 MB

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

pub struct VertexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub current_size: vk::DeviceSize, // in bytes
}

impl VertexBuffer {
    pub unsafe fn new(device: &RendererDevice) -> Result<VertexBuffer> {
        let buffer = {
            let create_info = vk::BufferCreateInfo::builder()
                .size(VERTEX_BUFFER_SIZE)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build();
            device.logical_device.create_buffer(&create_info, None)?
        };

        let memory_req = device.logical_device.get_buffer_memory_requirements(buffer);

        let memory = {
            let buffer_allocate_info = {
                let buffer_memory_index = Self::find_memorytype_index(
                    &memory_req,
                    &device.memory_properties,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
                .context("Unable to find suitable memorytype for the index buffer.")?;

                vk::MemoryAllocateInfo::builder()
                    .allocation_size(memory_req.size)
                    .memory_type_index(buffer_memory_index)
            };

            device
                .logical_device
                .allocate_memory(&buffer_allocate_info, None)
        }?;

        device
            .logical_device
            .bind_buffer_memory(buffer, memory, 0)?;

        Ok(VertexBuffer {
            buffer,
            memory,
            current_size: 0,
        })
    }

    pub unsafe fn set_vertices_from_slice(&mut self, device: &Device, vertices: &[Vertex]) -> Result<()> {
        let size = (vertices.len() * mem::size_of::<Vertex>()) as vk::DeviceSize;

        if size > VERTEX_BUFFER_SIZE {
            return Err(anyhow::anyhow!("Too many vertices to copy."));
        }

        let ptr = device.map_memory(
            self.memory,
            0,
            size,
            vk::MemoryMapFlags::empty(),
        )?;

        let mut align = Align::new(ptr, mem::align_of::<u32>() as u64, size);

        align.copy_from_slice(vertices);
        device.unmap_memory(self.memory);

        self.current_size = size;

        Ok(())
    }

    fn find_memorytype_index(
        memory_req: &vk::MemoryRequirements,
        memory_prop: &vk::PhysicalDeviceMemoryProperties,
        flags: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        memory_prop.memory_types[..memory_prop.memory_type_count as _]
            .iter()
            .enumerate()
            .find(|(index, memory_type)| {
                (1 << index) & memory_req.memory_type_bits != 0
                    && memory_type.property_flags & flags == flags
            })
            .map(|(index, _memory_type)| index as _)
    }

    pub unsafe fn cleanup(&self, device: &Device) {
        device.free_memory(self.memory, None);
        device.destroy_buffer(self.buffer, None);
    }
}
