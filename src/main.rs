use anyhow::{Context, Result};
use ash::{
    self,
    vk::{
        self, BufferCreateInfo, CommandBufferAllocateInfo, CommandBufferBeginInfo,
        CommandBufferLevel, CommandPoolCreateInfo, DeviceQueueCreateInfo,
        FenceCreateInfo, SubmitInfo,
    },
};
use gpu_allocator::{vulkan::*, MemoryLocation};

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

    let queue = unsafe { device.get_device_queue(0, 0) };

    let mut allocator = Allocator::new(&AllocatorCreateDesc {
        instance: instance.clone(),
        device: device.clone(),
        physical_device,
        debug_settings: Default::default(),
        buffer_device_address: true,
        allocation_sizes: Default::default(),
    })?;

    let value_count = 16;
    let value = 314;

    let buffer = {
        let create_info = BufferCreateInfo::builder()
            .size(value_count * std::mem::size_of::<i32>() as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::TRANSFER_DST)
            .build();
        unsafe { device.create_buffer(&create_info, None) }?
    };

    let allocation = {
        let buffer_memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let allocation_create_desc = AllocationCreateDesc {
            name: "Buffer allocation",
            requirements: buffer_memory_requirements,
            location: MemoryLocation::GpuToCpu,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        };
        let allocation = allocator.allocate(&allocation_create_desc)?;

        unsafe { device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()) }?;
        allocation
    };

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
            .level(CommandBufferLevel::PRIMARY)
            .build();
        unsafe { device.allocate_command_buffers(&allocate_info) }?
    };

    let primary_command_buffer = command_buffers[0];

    // Recording command buffer
    {
        let begin_info = CommandBufferBeginInfo::builder().build();
        unsafe { device.begin_command_buffer(primary_command_buffer, &begin_info) }?;
    }

    unsafe {
        device.cmd_fill_buffer(
            primary_command_buffer,
            buffer,
            allocation.offset(),
            allocation.size(),
            value,
        )
    }

    unsafe { device.end_command_buffer(primary_command_buffer) }?;

    // Creating synchronization object
    let fence = {
        let create_info = FenceCreateInfo::builder().build();
        unsafe { device.create_fence(&create_info, None) }?
    };

    // Execute command buffer
    {
        let submit_info = SubmitInfo::builder()
            .command_buffers(std::slice::from_ref(&primary_command_buffer))
            .build();
        unsafe { device.queue_submit(queue, std::slice::from_ref(&submit_info), fence) }?;
    }

    // Wait for the execution result
    unsafe { device.wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX) }?;

    // Read Data
    for value in allocation
        .mapped_slice()
        .context("Cannot access buffer from Host")?
    {
        println!("{}", value);
    }

    // Clean
    unsafe { device.destroy_fence(fence, None) }
    unsafe { device.destroy_command_pool(command_pool, None) }

    allocator.free(allocation)?;
    unsafe { device.destroy_buffer(buffer, None) }
    drop(allocator);

    unsafe { device.destroy_device(None) }
    unsafe { instance.destroy_instance(None) }
    Ok(())
}
