use std::rc::Rc;

use anyhow::{bail, Context, Ok, Result};
use ash::vk;

use super::{RendererDevice, ScopCommandPool};

pub struct ScopImage {
    device: Rc<RendererDevice>,
    pub image: vk::Image,
    pub device_memory: vk::DeviceMemory,
    pub format: vk::Format,
    pub layout: vk::ImageLayout,
    pub width: u32,
    pub height: u32,
    mip_levels: u32,
    array_layers: u32,
}

impl ScopImage {
    pub fn new(
        device: Rc<RendererDevice>,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        width: u32,
        height: u32,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        let mip_levels = 1u32;
        let array_layers = 1u32;

        let image = {
            let create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .extent(*vk::Extent3D::builder().width(width).height(height).depth(1))
                .mip_levels(mip_levels)
                .array_layers(array_layers)
                .format(format)
                .tiling(tiling)
                .usage(usage)
                .samples(vk::SampleCountFlags::TYPE_1)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            unsafe { device.logical_device.create_image(&create_info, None)? }
        };

        let device_memory = {
            let memory_requirements =
                unsafe { device.logical_device.get_image_memory_requirements(image) };
            let memory_type_index = RendererDevice::find_memorytype_index(
                &memory_requirements,
                device.memory_properties,
                memory_property_flags,
            )
            .context("Could not find a valid memory type.")?;
            let allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_requirements.size)
                .memory_type_index(memory_type_index);
            unsafe {
                device
                    .logical_device
                    .allocate_memory(&allocate_info, None)?
            }
        };

        unsafe {
            device
                .logical_device
                .bind_image_memory(image, device_memory, 0)?
        };

        Ok(Self {
            device,
            image,
            device_memory,
            format,
            layout: vk::ImageLayout::UNDEFINED,
            width,
            height,
            mip_levels,
            array_layers,
        })
    }

    pub fn change_layout(
        &mut self,
        command_pool: &ScopCommandPool,
        new_layout: vk::ImageLayout,
    ) -> Result<()> {
        unsafe {
            let command_buffer = command_pool.begin_single_time_commands()?;

            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
                match (self.layout, new_layout) {
                    (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                        vk::AccessFlags::empty(),
                        vk::AccessFlags::TRANSFER_WRITE,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::TRANSFER,
                    ),
                    (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::READ_ONLY_OPTIMAL) => {
                        (
                            vk::AccessFlags::TRANSFER_WRITE,
                            vk::AccessFlags::SHADER_READ,
                            vk::PipelineStageFlags::TRANSFER,
                            vk::PipelineStageFlags::FRAGMENT_SHADER,
                        )
                    }
                    _ => bail!("Image transition unsupported"),
                };

            let image_memory_barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(self.layout)
                .new_layout(new_layout)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .src_access_mask(src_access_mask)
                .dst_access_mask(dst_access_mask)
                .image(self.image)
                .subresource_range(*subresource_range);

            self.device.logical_device.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[*image_memory_barrier],
            );

            command_pool.end_single_time_commands(command_buffer)?;
        }

        self.layout = new_layout;
        Ok(())
    }

    pub fn create_image_view(&self, aspect_mask: vk::ImageAspectFlags) -> Result<vk::ImageView> {
        // Access all levels, all layers
        let image_subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_mip_level(0)
            .level_count(self.mip_levels)
            .base_array_layer(0)
            .layer_count(self.array_layers);

        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(self.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(self.format)
            .subresource_range(*image_subresource_range);

        let image_view = unsafe {
            self.device
                .logical_device
                .create_image_view(&image_view_create_info, None)?
        };

        Ok(image_view)
    }

    pub fn cleanup_image_view(&self, image_view: vk::ImageView) {
        unsafe {
            self.device
                .logical_device
                .destroy_image_view(image_view, None);
        }
    }

    pub fn cleanup(&mut self) {
        unsafe {
            self.device.logical_device.destroy_image(self.image, None);
            self.device
                .logical_device
                .free_memory(self.device_memory, None);
        }
    }
}
