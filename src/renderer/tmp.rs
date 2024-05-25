use std::rc::Rc;

use anyhow::{bail, Context, Result};
use ash::vk::{
    self, Image, ImageAspectFlags, ImageCreateInfo, ImageLayout, ImageMemoryBarrier,
    MemoryPropertyFlags, PipelineStageFlags,
};

use super::{device::RendererDevice, scop_buffer::ScopBuffer};

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

pub fn transition_image_layout(
    device: &Rc<RendererDevice>,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) -> Result<()> {
    unsafe {
        let command_buffer = begin_single_time_commands(device, command_pool)?;

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
            match (old_layout, new_layout) {
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::TRANSFER_WRITE,
                    PipelineStageFlags::TOP_OF_PIPE,
                    PipelineStageFlags::TRANSFER,
                ),
                (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::READ_ONLY_OPTIMAL) => (
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::AccessFlags::SHADER_READ,
                    PipelineStageFlags::TRANSFER,
                    PipelineStageFlags::FRAGMENT_SHADER,
                ),
                _ => bail!("Image transition unsupported"),
            };

        let image_memory_barrier = ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .image(image)
            .subresource_range(*subresource_range);

        device.logical_device.cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[*image_memory_barrier],
        );

        end_single_time_commands(device, command_pool, queue, command_buffer)?;
    }
    Ok(())
}

pub fn copy_buffer(
    device: &Rc<RendererDevice>,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    unsafe {
        let command_buffer = begin_single_time_commands(device, command_pool)?;

        let region = vk::BufferCopy::builder().size(size);

        device
            .logical_device
            .cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[*region]);

        end_single_time_commands(device, command_pool, queue, command_buffer)?;
        Ok(())
    }
}

pub fn copy_buffer_to_image(
    device: &Rc<RendererDevice>,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    src_buffer: vk::Buffer,
    dst_image: vk::Image,
    width: u32,
    height: u32,
) -> Result<()> {
    unsafe {
        let command_buffer = begin_single_time_commands(device, command_pool)?;

        let image_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);

        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_offset(*vk::Offset3D::builder().x(0).y(0).z(0))
            .image_extent(*vk::Extent3D::builder().width(width).height(height).depth(1))
            .image_subresource(*image_subresource);

        device.logical_device.cmd_copy_buffer_to_image(
            command_buffer,
            src_buffer,
            dst_image,
            ImageLayout::TRANSFER_DST_OPTIMAL,
            &[*region],
        );

        end_single_time_commands(device, command_pool, queue, command_buffer)?;
    }
    Ok(())
}

pub fn create_image(
    device: &Rc<RendererDevice>,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    width: u32,
    height: u32,
    memory_property_flags: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory)> {
    unsafe {
        let image = {
            let create_info = ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .extent(*vk::Extent3D::builder().width(width).height(height).depth(1))
                .mip_levels(1)
                .array_layers(1)
                .format(format)
                .tiling(tiling)
                .usage(usage)
                .samples(vk::SampleCountFlags::TYPE_1)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            device.logical_device.create_image(&create_info, None)?
        };

        let device_memory = {
            let memory_requirements = device.logical_device.get_image_memory_requirements(image);
            let memory_type_index = RendererDevice::find_memorytype_index(
                &memory_requirements,
                device.memory_properties,
                memory_property_flags,
            )
            .context("Could not find a valid memory type.")?;
            let allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_requirements.size)
                .memory_type_index(memory_type_index);
            device
                .logical_device
                .allocate_memory(&allocate_info, None)?
        };

        device
            .logical_device
            .bind_image_memory(image, device_memory, 0)?;

        Ok((image, device_memory))
    }
}

pub fn create_texture_image(
    device: &Rc<RendererDevice>,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    data: &[u8],
    width: u32,
    height: u32,
    channels: u32,
) -> Result<(vk::Image, vk::DeviceMemory)> {
    unsafe {
        let size = (width as vk::DeviceSize) * (height as vk::DeviceSize);
        let mut staging_buffer = ScopBuffer::new(
            device.clone(),
            1,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            1,
        )?;

        staging_buffer.map(vk::WHOLE_SIZE, 0)?;
        staging_buffer.write_to_buffer(data, 0);
        staging_buffer.unmap();

        let (image, memory) = create_image(
            device,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            width,
            height,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        transition_image_layout(
            device,
            command_pool,
            queue,
            image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;
        copy_buffer_to_image(
            device,
            command_pool,
            queue,
            staging_buffer.buffer,
            image,
            width,
            height,
        )?;
        transition_image_layout(
            device,
            command_pool,
            queue,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::READ_ONLY_OPTIMAL,
        )?;

        staging_buffer.cleanup();

        Ok((image, memory))
    }
}
// void createTextureImage()
// {
//     int texWidth, texHeight, texChannels;
//     stbi_uc* pixels = stbi_load("textures/texture.jpg", &texWidth, &texHeight, &texChannels, STBI_rgb_alpha);
//     VkDeviceSize imageSize = texWidth * texHeight * 4;

//     transitionImageLayout(textureImage, VK_FORMAT_R8G8B8A8_SRGB, VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL);
//     copyBufferToImage(stagingBuffer, textureImage, static_cast<uint32_t>(texWidth), static_cast<uint32_t>(texHeight));
//     transitionImageLayout(textureImage, VK_FORMAT_R8G8B8A8_SRGB, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL);

//     vkDestroyBuffer(device, stagingBuffer, nullptr);
//     vkFreeMemory(device, stagingBufferMemory, nullptr);
// }
