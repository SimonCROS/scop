use std::{ffi::c_void, ptr::null_mut, rc::Rc};

use anyhow::{Context, Ok, Result};
use ash::{util::Align, vk};

use super::{RendererDevice, ScopCommandPool, ScopImage};

pub struct ScopBuffer {
    device: Rc<RendererDevice>,
    mapped: *mut c_void,
    pub buffer: vk::Buffer,
    device_memory: vk::DeviceMemory,
    buffer_size: vk::DeviceSize,
    pub instance_count: usize,
    pub instance_size: vk::DeviceSize,
    alignment_size: vk::DeviceSize,
    usage_flags: vk::BufferUsageFlags,
    memory_property_flags: vk::MemoryPropertyFlags,
}

impl ScopBuffer {
    pub fn new(
        device: Rc<RendererDevice>,
        instance_count: usize,
        instance_size: vk::DeviceSize,
        usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
        min_offset_alignment: vk::DeviceSize,
    ) -> Result<Self> {
        let alignment_size: u64 = Self::get_alignment(instance_size, min_offset_alignment);
        let buffer_size = alignment_size * (instance_count as vk::DeviceSize);
        let (buffer, device_memory) = unsafe {
            Self::create_buffer(&device, buffer_size, usage_flags, memory_property_flags)?
        };

        Ok(Self {
            device,
            mapped: null_mut(),
            buffer,
            device_memory,
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

    pub fn map(&mut self, size: vk::DeviceSize, offset: vk::DeviceSize) -> Result<()> {
        assert!(!self.is_mapped());
        unsafe {
            self.mapped = self.device.logical_device.map_memory(
                self.device_memory,
                offset,
                size,
                vk::MemoryMapFlags::empty(),
            )?
        };
        Ok(())
    }

    pub fn unmap(&mut self) {
        if self.is_mapped() {
            unsafe { self.device.logical_device.unmap_memory(self.device_memory) };
            self.mapped = null_mut();
        }
    }

    pub fn flush(&self, size: vk::DeviceSize, offset: vk::DeviceSize) -> Result<()> {
        assert!(self.is_mapped());

        let range = vk::MappedMemoryRange::builder()
            .memory(self.device_memory)
            .offset(offset)
            .size(size);

        unsafe {
            self.device
                .logical_device
                .flush_mapped_memory_ranges(&[*range])?
        };

        Ok(())
    }

    pub fn write_to_buffer<T: Copy>(&mut self, data: &[T], offset: vk::DeviceSize) {
        assert!(self.is_mapped());

        let size = self.alignment_size * data.len() as vk::DeviceSize;
        let mut align =
            unsafe { Align::new(self.mapped.add(offset as usize), self.alignment_size, size) };
        align.copy_from_slice(data);
    }

    pub fn copy_to_buffer(
        &self,
        command_pool: &ScopCommandPool,
        dst_buffer: vk::Buffer,
        size: vk::DeviceSize,
    ) -> Result<()> {
        let command_buffer = command_pool.begin_single_time_commands()?;

        let region = vk::BufferCopy::builder().size(size);

        unsafe {
            self.device.logical_device.cmd_copy_buffer(
                command_buffer,
                self.buffer,
                dst_buffer,
                &[*region],
            )
        };

        command_pool.end_single_time_commands(command_buffer)?;
        Ok(())
    }

    pub fn copy_to_image(
        &self,
        command_pool: &ScopCommandPool,
        dst_image: &ScopImage,
    ) -> Result<()> {
        assert!(
            dst_image.layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            "Image layout should be TRANSFER_DST_OPTIMAL"
        );

        let command_buffer = command_pool.begin_single_time_commands()?;

        let image_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);

        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_offset(*vk::Offset3D::builder().x(0).y(0).z(0))
            .image_extent(
                *vk::Extent3D::builder()
                    .width(dst_image.width)
                    .height(dst_image.height)
                    .depth(1),
            )
            .image_subresource(*image_subresource);

        unsafe {
            self.device.logical_device.cmd_copy_buffer_to_image(
                command_buffer,
                self.buffer,
                dst_image.image,
                dst_image.layout,
                &[*region],
            )
        };

        command_pool.end_single_time_commands(command_buffer)?;
        Ok(())
    }

    pub fn descriptor_info(
        &self,
        size: vk::DeviceSize,
        offset: vk::DeviceSize,
    ) -> vk::DescriptorBufferInfo {
        vk::DescriptorBufferInfo::builder()
            .buffer(self.buffer)
            .offset(offset)
            .range(size)
            .build()
    }

    pub fn cleanup(&mut self) {
        self.unmap();
        unsafe {
            self.device.logical_device.destroy_buffer(self.buffer, None);
            self.device
                .logical_device
                .free_memory(self.device_memory, None);
        }
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
