use std::mem;

use anyhow::{Context, Result};
use ash::{util::Align, vk};

use super::device::RendererDevice;

pub struct IndexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

impl IndexBuffer {
    pub fn new(device: &RendererDevice, index: &[u32]) -> Result<IndexBuffer> {
        let buffer_size = (index.len() * std::mem::size_of::<u32>()) as vk::DeviceSize;

        let index_input_buffer = {
            let create_info = vk::BufferCreateInfo::builder()
                .size(buffer_size)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build();
            unsafe { device.logical_device.create_buffer(&create_info, None) }?
        };

        let index_input_buffer_memory_req = unsafe {
            device
                .logical_device
                .get_buffer_memory_requirements(index_input_buffer)
        };

        let index_input_buffer_memory_index = Self::find_memorytype_index(
            &index_input_buffer_memory_req,
            &device.memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .context("Unable to find suitable memorytype for the index buffer.")?;

        let index_buffer_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: index_input_buffer_memory_req.size,
            memory_type_index: index_input_buffer_memory_index,
            ..Default::default()
        };

        let index_input_buffer_memory = unsafe {
            device
                .logical_device
                .allocate_memory(&index_buffer_allocate_info, None)
        }?;

        unsafe {
            device
                .logical_device
                .bind_buffer_memory(index_input_buffer, index_input_buffer_memory, 0)?;
        }

        // Copy vertex data to buffer
        let vert_ptr = unsafe {
            device.logical_device.map_memory(
                index_input_buffer_memory,
                0,
                index_input_buffer_memory_req.size,
                vk::MemoryMapFlags::empty(),
            )
        }?;

        let mut vert_align = unsafe {
            Align::new(
                vert_ptr,
                mem::align_of::<u32>() as u64,
                index_input_buffer_memory_req.size,
            )
        };

        vert_align.copy_from_slice(&index);

        unsafe {
            device
                .logical_device
                .unmap_memory(index_input_buffer_memory);
        }

        Ok(IndexBuffer {
            buffer: index_input_buffer,
            memory: index_input_buffer_memory,
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
