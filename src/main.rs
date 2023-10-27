use std::mem::size_of;

use anyhow::{Context, Result};
use ash::{
    self,
    vk::{
        self, BufferCreateInfo, CommandBufferAllocateInfo, CommandPoolCreateFlags,
        CommandPoolCreateInfo, DeviceQueueCreateInfo,
    },
};

fn main() -> Result<()> {
    let entry = unsafe { ash::Entry::load() }?;

    let instance = {
        let application_info = vk::ApplicationInfo::builder()
            .api_version(vk::API_VERSION_1_3)
            .build();
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .build();
        unsafe { entry.create_instance(&create_info, None) }?
    };

    let physical_device = unsafe { instance.enumerate_physical_devices() }?
        .into_iter()
        .next()
        .context("No physical device found")?;

    let queue_family_index =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) }
            .into_iter()
            .enumerate()
            .filter(|item| {
                item.1.queue_flags.intersects(
                    vk::QueueFlags::TRANSFER | vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE,
                )
            })
            .max_by_key(|item| (item.1.queue_flags.as_raw().count_ones(), item.1.queue_count))
            .context("No suitable queue family")?
            .0 as u32;

    let device = {
        let queue_priorities = [1.0];
        let queue_create_infos = [DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities)
            .build()];
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .build();
        unsafe { instance.create_device(physical_device, &create_info, None) }?
    };

    // let queue = unsafe { device.get_device_queue(0, 0) };

    let command_pool = {
        let create_info = CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .build();
        unsafe { device.create_command_pool(&create_info, None) }?
    };

    let command_buffers = {
        let allocate_info = CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(1)
            .build();
        unsafe { device.allocate_command_buffers(&allocate_info) }?
    };

    let buffer = {
        let create_info = BufferCreateInfo::builder()
            .size(16 * std::mem::size_of::<i32>() as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .build();
        unsafe { device.create_buffer(&create_info, None) }?
    };

    unsafe { device.destroy_buffer(buffer, None) }
    unsafe { device.free_command_buffers(command_pool, &command_buffers) }
    unsafe { device.destroy_command_pool(command_pool, None) }
    unsafe { device.destroy_device(None) }
    unsafe { instance.destroy_instance(None) }
    Ok(())
}
