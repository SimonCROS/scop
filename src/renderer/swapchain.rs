use core::slice;

use anyhow::{Context, Result};
use ash::{
    extensions,
    vk::{
        self, FormatFeatureFlags, ImageAspectFlags, ImageCreateInfo, ImageSubresourceRange,
        ImageViewCreateInfo, MemoryAllocateInfo, QueueFlags,
    },
    Device,
};

use super::{device::RendererDevice, window::RendererWindow};

pub struct RendererSwapchain {
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: extensions::khr::Swapchain,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub image_count: usize,
    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,
    image_available: Vec<vk::Semaphore>,
    rendering_finished: Vec<vk::Semaphore>,
    may_begin_drawing: Vec<vk::Fence>,
    current_image: usize,
}

impl RendererSwapchain {
    pub fn new(
        instance: &ash::Instance,
        device: &RendererDevice,
        window: &RendererWindow,
    ) -> Result<Self> {
        dbg!("New swapchain");

        let graphics_queue_family = device.get_queue_family_with(QueueFlags::GRAPHICS).unwrap();

        let capabilities = window.capabilities(device.physical_device)?;

        let extent = capabilities.current_extent;

        let surface_formats = window.formats(device.physical_device)?;
        let surface_format = surface_formats.first().unwrap();

        let swapchain_loader = extensions::khr::Swapchain::new(instance, &device.logical_device);

        let queue_family_indicies = [graphics_queue_family.index];

        let swapchain = {
            let min_image_count = if capabilities.max_image_count > 0 {
                3.min(capabilities.max_image_count)
            } else {
                3.max(capabilities.min_image_count)
            };

            let swapchain_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(window.surface)
                .min_image_count(min_image_count)
                .image_format(surface_format.format)
                .image_color_space(surface_format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .queue_family_indices(&queue_family_indicies)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO);

            unsafe { swapchain_loader.create_swapchain(&swapchain_info, None) }?
        };

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        let mut image_views = Vec::with_capacity(images.len());

        for image in images {
            let image_view = {
                let subresource_range = vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build();

                let image_view_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .subresource_range(subresource_range);

                unsafe {
                    device
                        .logical_device
                        .create_image_view(&image_view_info, None)
                }?
            };

            image_views.push(image_view);
        }

        let image_count = image_views.len();

        let (depth_image, depth_image_memory, depth_image_view) = unsafe {RendererSwapchain::create_depth_resources(device, extent)? };

        let mut swapchain = RendererSwapchain {
            swapchain,
            swapchain_loader,
            image_views,
            extent,
            image_available: vec![],
            rendering_finished: vec![],
            may_begin_drawing: vec![],
            image_count,
            depth_image,
            depth_image_memory,
            depth_image_view,
            current_image: 0,
        };

        swapchain.create_sync(device)?;

        Ok(swapchain)
    }

    pub fn next_image(
        &mut self,
        device: &RendererDevice,
    ) -> Result<(u32, vk::Semaphore, vk::Semaphore, vk::Fence)> {
        let image_available = &self.image_available[self.current_image];
        let rendering_finished = &self.rendering_finished[self.current_image];
        let may_begin_drawing = &self.may_begin_drawing[self.current_image];

        let (image_index, _) = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                *image_available,
                vk::Fence::null(),
            )?
        };

        unsafe {
            device.logical_device.wait_for_fences(
                slice::from_ref(may_begin_drawing),
                true,
                std::u64::MAX,
            )?;
            device
                .logical_device
                .reset_fences(slice::from_ref(may_begin_drawing))?;
        }

        self.current_image = (self.current_image + 1) % self.image_count;

        Ok((
            image_index,
            *image_available,
            *rendering_finished,
            *may_begin_drawing,
        ))
    }

    pub fn present_image(
        &self,
        queue: vk::Queue,
        image_index: u32,
        wait_semaphores: &[vk::Semaphore],
    ) -> Result<()> {
        let swapchains = [self.swapchain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe { self.swapchain_loader.queue_present(queue, &present_info)? };
        Ok(())
    }

    pub unsafe fn cleanup(&self, device: &Device) {
        dbg!("Cleanup swapchain");

        for semaphore in &self.image_available {
            device.destroy_semaphore(*semaphore, None);
        }

        for semaphore in &self.rendering_finished {
            device.destroy_semaphore(*semaphore, None);
        }

        for fence in &self.may_begin_drawing {
            device.destroy_fence(*fence, None);
        }

        for image_view in &self.image_views {
            device.destroy_image_view(*image_view, None);
        }

        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None);
    }

    unsafe fn create_depth_resources(device: &RendererDevice, extent: vk::Extent2D) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
        let depth_format = device.find_supported_format(
            vec![
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )?;

        let extent = vk::Extent3D::builder()
            .width(extent.width)
            .height(extent.height)
            .depth(1);

        let image_create_info = ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(*extent)
            .mip_levels(1)
            .array_layers(1)
            .format(depth_format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let depth_image = device
            .logical_device
            .create_image(&image_create_info, None)?;
        let memory_requirements = device
            .logical_device
            .get_image_memory_requirements(depth_image);
        let memory_type = RendererDevice::find_memorytype_index(
            &memory_requirements,
            device.memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .context("No compatible memory type found for depth buffer")?;

        let allocate_info = MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type);

        let depth_image_memory = device
            .logical_device
            .allocate_memory(&allocate_info, None)?;
        device
            .logical_device
            .bind_image_memory(depth_image, depth_image_memory, 0)?;

        let image_subresource_range = ImageSubresourceRange::builder()
            .aspect_mask(ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let image_view_create_info = ImageViewCreateInfo::builder()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(depth_format)
            .subresource_range(*image_subresource_range);

        let depth_image_view =
                device
                    .logical_device
                    .create_image_view(&image_view_create_info, None)?;

        Ok((depth_image, depth_image_memory, depth_image_view))
    }

    fn create_sync(&mut self, device: &RendererDevice) -> Result<()> {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();

        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..self.image_views.len() {
            let semaphore_available = unsafe {
                device
                    .logical_device
                    .create_semaphore(&semaphore_info, None)
            }?;

            let semaphore_finished = unsafe {
                device
                    .logical_device
                    .create_semaphore(&semaphore_info, None)
            }?;

            self.image_available.push(semaphore_available);
            self.rendering_finished.push(semaphore_finished);

            let fence = unsafe { device.logical_device.create_fence(&fence_info, None) }?;

            self.may_begin_drawing.push(fence);
        }

        Ok(())
    }
}
