use std::rc::Rc;

use anyhow::{Ok, Result};
use ash::vk::{self, CommandPoolCreateFlags};

use super::{QueueFamily, QueueFamilyId, RendererDevice};

pub struct ScopCommandPool {
    device: Rc<RendererDevice>,
    command_pool: vk::CommandPool,
    pub queue_family: QueueFamilyId,
    pub command_buffers: Vec<vk::CommandBuffer>,
}

impl ScopCommandPool {
    pub fn new(
        device: Rc<RendererDevice>,
        queue_family: &QueueFamily,
        flags: CommandPoolCreateFlags,
    ) -> Result<Self> {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family.index)
            .flags(flags);

        let command_pool = unsafe {
            device
                .logical_device
                .create_command_pool(&create_info, None)
        }?;

        Ok(Self {
            device,
            queue_family: queue_family.id,
            command_pool,
            command_buffers: vec![],
        })
    }

    pub fn create_command_buffers(&mut self, amount: u32) -> Result<()> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .command_buffer_count(amount);

        self.command_buffers = unsafe {
            self.device
                .logical_device
                .allocate_command_buffers(&allocate_info)?
        };

        Ok(())
    }

    pub fn get_command_buffer(&self, index: u32) -> vk::CommandBuffer {
        self.command_buffers[index as usize]
    }

    pub fn get_queue_family(&self) -> &QueueFamily {
        self.device.get_queue_family(self.queue_family)
    }

    pub fn begin_single_time_commands(&self) -> Result<vk::CommandBuffer> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.command_pool)
            .command_buffer_count(1)
            .build();

        let command_buffer = unsafe {
            self.device
                .logical_device
                .allocate_command_buffers(&alloc_info)?[0]
        };

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        unsafe {
            self.device
                .logical_device
                .begin_command_buffer(command_buffer, &begin_info)?
        };

        Ok(command_buffer)
    }

    pub fn end_single_time_commands(&self, command_buffer: vk::CommandBuffer) -> Result<()> {
        unsafe {
            self.device
                .logical_device
                .end_command_buffer(command_buffer)?
        };

        let submit_info =
            vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&command_buffer));

        let queue = self.get_queue_family().queues[0];

        unsafe {
            self.device.logical_device.queue_submit(
                queue,
                &[submit_info.build()],
                vk::Fence::null(),
            )?
        };

        unsafe { self.device.logical_device.queue_wait_idle(queue)? };
        unsafe {
            self.device
                .logical_device
                .free_command_buffers(self.command_pool, std::slice::from_ref(&command_buffer))
        };

        Ok(())
    }

    pub fn submit(
        &self,
        command_buffers: &[vk::CommandBuffer],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
        wait_stages: &[vk::PipelineStageFlags],
        fence: vk::Fence,
    ) -> Result<()> {
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        let queue = self.get_queue_family().queues[0];

        unsafe {
            self.device
                .logical_device
                .queue_submit(queue, &[submit_info.build()], fence)?
        };

        Ok(())
    }

    pub fn cleanup(&mut self) {
        unsafe {
            self.device
                .logical_device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            self.device
                .logical_device
                .destroy_command_pool(self.command_pool, None);
        }
    }
}
