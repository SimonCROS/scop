use std::rc::Rc;

use anyhow::{Ok, Result};
use ash::vk;

use super::{RendererDevice, RendererWindow, ScopFramebuffer, ScopSwapchain};

pub struct ScopRenderPass {
    device: Rc<RendererDevice>,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<ScopFramebuffer>,
}

impl ScopRenderPass {
    pub fn new(
        device: Rc<RendererDevice>,
        window: &RendererWindow,
        swapchain: &ScopSwapchain,
    ) -> Result<Self> {
        let surface_formats = window.formats(device.physical_device)?;
        let surface_format = surface_formats.first().unwrap();
        let depth_format = device.find_supported_format(
            vec![
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )?;

        let attachments = [
            vk::AttachmentDescription::builder()
                .format(surface_format.format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .build(),
            vk::AttachmentDescription::builder()
                .format(depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build(),
        ];

        let color_attachment_references = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];

        let depth_attachment_references = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_references)
            .depth_stencil_attachment(&depth_attachment_references)
            .build()];

        let subpass_dependencies = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .build()];

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        let render_pass = unsafe {
            device
                .logical_device
                .create_render_pass(&render_pass_info, None)
        }?;

        let framebuffers = ScopRenderPass::create_framebuffers(&device, render_pass, swapchain)?;

        Ok(Self {
            device,
            render_pass,
            framebuffers,
        })
    }

    pub fn change_swapchain(&mut self, swapchain: &ScopSwapchain) -> Result<()> {
        self.destroy_framebuffers();
        self.framebuffers =
            ScopRenderPass::create_framebuffers(&self.device, self.render_pass, swapchain)?;

        Ok(())
    }

    pub fn begin(&self, command_buffer: vk::CommandBuffer, image_index: u32) {
        let framebuffer = &self.framebuffers[image_index as usize];

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [1.0, 1.0, 1.0, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1f32,
                    stencil: 0,
                },
            },
        ];

        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: framebuffer.extent,
        };

        let render_pass_begin = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(framebuffer.framebuffer)
            .render_area(render_area)
            .clear_values(&clear_values);

        unsafe {
            self.device.logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin,
                vk::SubpassContents::INLINE,
            )
        };
    }

    pub fn end(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device
                .logical_device
                .cmd_end_render_pass(command_buffer);
        }
    }

    pub fn cleanup(&mut self) {
        self.destroy_framebuffers();

        unsafe {
            self.device
                .logical_device
                .destroy_render_pass(self.render_pass, None)
        };
    }

    fn create_framebuffers(
        device: &Rc<RendererDevice>,
        render_pass: vk::RenderPass,
        swapchain: &ScopSwapchain,
    ) -> Result<Vec<ScopFramebuffer>> {
        let mut framebuffers = Vec::with_capacity(swapchain.image_count);

        for i in 0..swapchain.image_count {
            framebuffers.push(ScopFramebuffer::new(
                device.clone(),
                swapchain.image_views[i],
                swapchain.depth_image_view,
                render_pass,
                swapchain.extent,
            )?);
        }

        Ok(framebuffers)
    }

    fn destroy_framebuffers(&mut self) {
        for framebuffer in &mut self.framebuffers {
            framebuffer.cleanup();
        }

        self.framebuffers.clear();
    }
}
