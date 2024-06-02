use core::slice;
use std::rc::Rc;

use anyhow::Result;
use ash::{
    extensions,
    vk::{self, FormatFeatureFlags, QueueFlags},
};

use super::{RendererDevice, RendererWindow, ScopImage};

pub struct ScopSwapchain {
    device: Rc<RendererDevice>,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: extensions::khr::Swapchain,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub image_count: usize,
    pub depth_image: ScopImage,
    pub depth_image_view: vk::ImageView,
    image_available: Vec<vk::Semaphore>,
    rendering_finished: Vec<vk::Semaphore>,
    may_begin_drawing: Vec<vk::Fence>,
    current_image: usize,
}

impl ScopSwapchain {
    pub fn new(
        instance: &ash::Instance,
        device: Rc<RendererDevice>,
        window: &RendererWindow,
    ) -> Result<Self> {
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

        let (depth_image, depth_image_view) =
            unsafe { ScopSwapchain::create_depth_resources(&device, extent)? };

        let mut swapchain = ScopSwapchain {
            device,
            swapchain,
            swapchain_loader,
            image_views,
            extent,
            image_available: vec![],
            rendering_finished: vec![],
            may_begin_drawing: vec![],
            image_count,
            depth_image,
            depth_image_view,
            current_image: 0,
        };

        swapchain.create_sync()?;

        Ok(swapchain)
    }

    pub fn next_image(&mut self) -> Result<(u32, vk::Semaphore, vk::Semaphore, vk::Fence)> {
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
            self.device.logical_device.wait_for_fences(
                slice::from_ref(may_begin_drawing),
                true,
                std::u64::MAX,
            )?;
            self.device
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

    pub fn cleanup(&mut self) {
        for semaphore in &self.image_available {
            unsafe {
                self.device
                    .logical_device
                    .destroy_semaphore(*semaphore, None)
            };
        }

        for semaphore in &self.rendering_finished {
            unsafe {
                self.device
                    .logical_device
                    .destroy_semaphore(*semaphore, None)
            };
        }

        for fence in &self.may_begin_drawing {
            unsafe { self.device.logical_device.destroy_fence(*fence, None) };
        }

        for image_view in &self.image_views {
            unsafe {
                self.device
                    .logical_device
                    .destroy_image_view(*image_view, None)
            };
        }

        unsafe {
            self.device
                .logical_device
                .destroy_image_view(self.depth_image_view, None)
        };
        self.depth_image.cleanup();

        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None)
        };
    }

    unsafe fn create_depth_resources(
        device: &Rc<RendererDevice>,
        extent: vk::Extent2D,
    ) -> Result<(ScopImage, vk::ImageView)> {
        let depth_format = device.find_supported_format(
            vec![
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )?;

        let depth_image = ScopImage::new(
            device.clone(),
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            extent.width,
            extent.height,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let depth_image_view = depth_image.create_image_view(vk::ImageAspectFlags::DEPTH)?;

        Ok((depth_image, depth_image_view))
    }

    fn create_sync(&mut self) -> Result<()> {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();

        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..self.image_views.len() {
            let semaphore_available = unsafe {
                self.device
                    .logical_device
                    .create_semaphore(&semaphore_info, None)
            }?;

            let semaphore_finished = unsafe {
                self.device
                    .logical_device
                    .create_semaphore(&semaphore_info, None)
            }?;

            self.image_available.push(semaphore_available);
            self.rendering_finished.push(semaphore_finished);

            let fence = unsafe { self.device.logical_device.create_fence(&fence_info, None) }?;

            self.may_begin_drawing.push(fence);
        }

        Ok(())
    }
}
