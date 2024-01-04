use anyhow::Result;
use ash::{extensions::khr, vk, Instance};

use super::{window::RendererWindow, device::RendererDevice};

pub struct RendererSwapchain {
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: khr::Swapchain,
    pub image_views: Vec<vk::ImageView>,
}

impl RendererSwapchain {
    pub fn new(instance: &Instance,
        device: RendererDevice,
        window: RendererWindow,
    ) -> Result<Self> {}

    pub unsafe fn cleanup(&self) {}
}
