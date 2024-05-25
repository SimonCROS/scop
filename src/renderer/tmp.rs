use std::rc::Rc;

use anyhow::{bail, Context, Result};
use ash::vk::{
    self, Image, ImageAspectFlags, ImageCreateInfo, ImageLayout, ImageMemoryBarrier,
    MemoryPropertyFlags, PipelineStageFlags,
};

use super::{device::RendererDevice, scop_buffer::ScopBuffer, scop_image::ScopImage};

pub fn begin_single_time_commands(
    device: &Rc<RendererDevice>,
    command_pool: vk::CommandPool,
) -> Result<vk::CommandBuffer> {
    unsafe {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1)
            .build();

        let command_buffer = device
            .logical_device
            .allocate_command_buffers(&alloc_info)?[0];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        device
            .logical_device
            .begin_command_buffer(command_buffer, &begin_info)?;

        Ok(command_buffer)
    }
}

pub fn end_single_time_commands(
    device: &Rc<RendererDevice>,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    command_buffer: vk::CommandBuffer,
) -> Result<()> {
    unsafe {
        device.logical_device.end_command_buffer(command_buffer)?;

        let submit_info =
            vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&command_buffer));

        device
            .logical_device
            .queue_submit(queue, &[submit_info.build()], vk::Fence::null())?;

        device.logical_device.queue_wait_idle(queue)?;
        device
            .logical_device
            .free_command_buffers(command_pool, std::slice::from_ref(&command_buffer));

        Ok(())
    }
}
