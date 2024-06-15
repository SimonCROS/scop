use std::rc::Rc;

use ash::vk;

use crate::utils::Result;

use super::RendererDevice;

pub struct ScopFramebuffer {
    device: Rc<RendererDevice>,
    pub framebuffer: vk::Framebuffer,
    pub extent: vk::Extent2D,
}

impl ScopFramebuffer {
    pub fn new(
        device: Rc<RendererDevice>,
        image_view: vk::ImageView,
        depth_image_view: vk::ImageView,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<Self> {
        let attachments = [image_view, depth_image_view];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let framebuffer = unsafe {
            device
                .logical_device
                .create_framebuffer(&framebuffer_info, None)
        }?;

        Ok(Self {
            device,
            framebuffer,
            extent,
        })
    }

    pub fn cleanup(&mut self) {
        unsafe {
            self.device
                .logical_device
                .destroy_framebuffer(self.framebuffer, None)
        };
    }
}
