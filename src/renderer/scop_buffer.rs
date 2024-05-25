use std::{ffi::c_void, mem, ptr::null_mut, rc::Rc};

use anyhow::{Context, Ok, Result};
use ash::{util::Align, vk};

use super::device::RendererDevice;

pub struct ScopBuffer {
    device: Rc<RendererDevice>,
    mapped: *mut c_void,
    pub buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    buffer_size: vk::DeviceSize,
    pub instance_count: usize,
    instance_size: vk::DeviceSize,
    alignment_size: vk::DeviceSize,
    usage_flags: vk::BufferUsageFlags,
    memory_property_flags: vk::MemoryPropertyFlags,
}

impl ScopBuffer {
    pub unsafe fn new(
        device: Rc<RendererDevice>,
        instance_count: usize,
        instance_size: vk::DeviceSize,
        usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
        min_offset_alignment: vk::DeviceSize,
    ) -> Result<Self> {
        let alignment_size: u64 = Self::get_alignment(instance_size, min_offset_alignment);
        let buffer_size = alignment_size * (instance_count as vk::DeviceSize);
        let (buffer, memory) =
            Self::create_buffer(&device, buffer_size, usage_flags, memory_property_flags)?;
        Ok(Self {
            device,
            mapped: null_mut(),
            buffer,
            memory,
            buffer_size,
            instance_count,
            instance_size,
            alignment_size,
            usage_flags,
            memory_property_flags,
        })
    }

    pub fn is_mapped(&self) -> bool {
        !self.mapped.is_null()
    }

    pub unsafe fn map(&mut self, size: vk::DeviceSize, offset: vk::DeviceSize) -> Result<()> {
        assert!(!self.is_mapped());
        self.mapped = self.device.logical_device.map_memory(
            self.memory,
            offset,
            size,
            vk::MemoryMapFlags::empty(),
        )?;
        Ok(())
    }

    pub unsafe fn unmap(&mut self) {
        if self.is_mapped() {
            self.device.logical_device.unmap_memory(self.memory);
        }
    }

    pub unsafe fn write_to_buffer<T: Copy>(&mut self, data: &[T], offset: vk::DeviceSize) {
        assert!(self.is_mapped());

        let size = self.alignment_size * data.len() as vk::DeviceSize;
        let mut align = Align::new(self.mapped.add(offset as usize), self.alignment_size, size);
        align.copy_from_slice(data);
    }

    pub unsafe fn descriptor_info(&self, size: vk::DeviceSize, offset: vk::DeviceSize) -> vk::DescriptorBufferInfo {
        vk::DescriptorBufferInfo::builder()
            .buffer(self.buffer)
            .offset(offset)
            .range(size)
            .build()
    }

    pub unsafe fn cleanup(mut self) {
        self.unmap();
        self.device.logical_device.free_memory(self.memory, None);
        self.device.logical_device.destroy_buffer(self.buffer, None);
    }

    fn get_alignment(
        instance_size: vk::DeviceSize,
        min_offset_alignment: vk::DeviceSize,
    ) -> vk::DeviceSize {
        if min_offset_alignment > 0 {
            (instance_size + min_offset_alignment - 1) & !(min_offset_alignment - 1)
        } else {
            instance_size
        }
    }

    unsafe fn create_buffer(
        device: &Rc<RendererDevice>,
        buffer_size: vk::DeviceSize,
        usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer = {
            let create_info = vk::BufferCreateInfo::builder()
                .size(buffer_size)
                .usage(usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build();
            device.logical_device.create_buffer(&create_info, None)?
        };

        let memory_req = device.logical_device.get_buffer_memory_requirements(buffer);

        let buffer_memory_index = RendererDevice::find_memorytype_index(
            &memory_req,
            device.memory_properties,
            memory_property_flags,
        )
        .context("Unable to find suitable memorytype for the index buffer.")?;

        let memory = {
            let allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_req.size)
                .memory_type_index(buffer_memory_index);

            device.logical_device.allocate_memory(&allocate_info, None)
        }?;

        device
            .logical_device
            .bind_buffer_memory(buffer, memory, 0)?;

        Ok((buffer, memory))
    }
}
