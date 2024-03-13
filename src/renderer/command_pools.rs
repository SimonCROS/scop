use anyhow::Result;
use ash::{vk, Device};

use super::device::RendererDevice;

pub struct CommandPools {
    pub graphics: vk::CommandPool,
}

impl CommandPools {
    pub fn new(device: &RendererDevice) -> Result<CommandPools> {
        dbg!("New command pools");

        let graphics_queue_family = device.main_graphics_queue_family();

        let graphics_command_pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(graphics_queue_family.index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let graphics_command_pool = unsafe {
            device
                .logical_device
                .create_command_pool(&graphics_command_pool_info, None)
        }?;

        Ok(CommandPools {
            graphics: graphics_command_pool,
        })
    }

    pub fn create_command_buffers(
        device: &RendererDevice,
        pool: vk::CommandPool,
        count: u32,
    ) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .command_buffer_count(count);

        unsafe {
            device
                .logical_device
                .allocate_command_buffers(&command_buffer_allocate_info)
        }
    }

    pub unsafe fn cleanup(&self, device: &Device, command_buffers: &Vec<vk::CommandBuffer>) {
        dbg!("Cleanup command pools");

        device.free_command_buffers(self.graphics, command_buffers.as_ref());
        device.destroy_command_pool(self.graphics, None);
    }
}
