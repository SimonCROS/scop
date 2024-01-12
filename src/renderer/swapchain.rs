use anyhow::Result;
use ash::{
    self,
    extensions::khr::{self, Swapchain},
    vk::{
        CompositeAlphaFlagsKHR, Format, ImageAspectFlags, ImageSubresourceRange, ImageUsageFlags,
        ImageView, ImageViewCreateInfo, ImageViewType, PresentModeKHR, SharingMode,
        SwapchainCreateInfoKHR, SwapchainKHR,
    },
    Instance,
};

use super::{device::RendererDevice, window::RendererWindow};

pub struct RendererSwapchain {
    pub swapchain: SwapchainKHR,
    pub swapchain_loader: khr::Swapchain,
    pub image_views: Vec<ImageView>,
}

impl RendererSwapchain {
    pub fn new(
        instance: &Instance,
        device: RendererDevice,
        window: RendererWindow,
    ) -> Result<Self> {
        let graphics_queue_family = device.main_graphics_queue_family();

        let capabilities = window.capabilities(device.physical_device)?;

        let surface_formats = window.formats(device.physical_device)?;
        let surface_format = surface_formats.first().unwrap();

        let swapchain_loader = Swapchain::new(instance, &device.logical_device);

        let queue_family_indicies = [graphics_queue_family.index];

        let swapchain = {
            let swapchain_info = SwapchainCreateInfoKHR::builder()
                .surface(window.surface)
                .min_image_count(
                    3.clamp(capabilities.min_image_count, capabilities.max_image_count),
                )
                .image_format(surface_format.format)
                .image_color_space(surface_format.color_space)
                .image_extent(capabilities.current_extent)
                .image_array_layers(1)
                .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(SharingMode::EXCLUSIVE)
                .queue_family_indices(&queue_family_indicies)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(PresentModeKHR::FIFO);

            unsafe { swapchain_loader.create_swapchain(&swapchain_info, None) }?
        };

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        let mut image_views = Vec::with_capacity(images.len());

        for image in images {
            let image_view = {
                let subresource_range = ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build();

                let image_view_info = ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(ImageViewType::TYPE_2D)
                    .format(Format::B8G8R8A8_UNORM)
                    .subresource_range(subresource_range);

                unsafe {
                    device
                        .logical_device
                        .create_image_view(&image_view_info, None)
                }?
            };

            image_views.push(image_view);
        }

        Ok(RendererSwapchain {
            swapchain,
            swapchain_loader,
            image_views,
        })
    }

    pub unsafe fn cleanup(&self, device: RendererDevice) {
        for image_view in &self.image_views {
            device.logical_device.destroy_image_view(*image_view, None);
        }

        self.swapchain_loader.destroy_swapchain(self.swapchain, None);
    }
}
