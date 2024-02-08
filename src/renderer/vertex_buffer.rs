use std::mem;

use anyhow::{Context, Result};
use ash::{util::Align, vk};

use super::device::RendererDevice;

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

pub struct VertexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

impl VertexBuffer {
    pub fn new(device: &RendererDevice, vertices: &[Vertex]) -> Result<VertexBuffer> {
        let buffer_size = (vertices.len() * std::mem::size_of::<Vertex>()) as vk::DeviceSize;

        let vertex_input_buffer = {
            let create_info = vk::BufferCreateInfo::builder()
                .size(buffer_size)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build();
            unsafe { device.logical_device.create_buffer(&create_info, None) }?
        };

        let vertex_input_buffer_memory_req = unsafe {
            device
                .logical_device
                .get_buffer_memory_requirements(vertex_input_buffer)
        };

        let vertex_input_buffer_memory_index = Self::find_memorytype_index(
            &vertex_input_buffer_memory_req,
            &device.memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .context("Unable to find suitable memorytype for the vertex buffer.")?;

        let vertex_buffer_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: vertex_input_buffer_memory_req.size,
            memory_type_index: vertex_input_buffer_memory_index,
            ..Default::default()
        };

        let vertex_input_buffer_memory = unsafe {
            device
                .logical_device
                .allocate_memory(&vertex_buffer_allocate_info, None)
        }?;

        unsafe {
            device
                .logical_device
                .bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0)?;
        }

        // Copy vertex data to buffer
        let vert_ptr = unsafe {
            device.logical_device.map_memory(
                vertex_input_buffer_memory,
                0,
                vertex_input_buffer_memory_req.size,
                vk::MemoryMapFlags::empty(),
            )
        }?;

        let mut vert_align = unsafe {
            Align::new(
                vert_ptr,
                mem::align_of::<Vertex>() as u64,
                vertex_input_buffer_memory_req.size,
            )
        };

        vert_align.copy_from_slice(&vertices);

        unsafe {
            device
                .logical_device
                .unmap_memory(vertex_input_buffer_memory);
        }

        Ok(VertexBuffer {
            buffer: vertex_input_buffer,
            memory: vertex_input_buffer_memory,
            size: buffer_size,
        })
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
}
