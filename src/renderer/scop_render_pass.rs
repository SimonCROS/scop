use std::rc::Rc;

use anyhow::{Ok, Result};
use ash::vk;

use super::{device::RendererDevice, window::RendererWindow};

pub struct ScopRenderPass {
    device: Rc<RendererDevice>,
    pub render_pass: vk::RenderPass,
}

impl ScopRenderPass {
    pub fn new(device: Rc<RendererDevice>, window: &RendererWindow) -> Result<Self> {
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
                .final_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
                .build(),
        ];

        let color_attachment_references = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];

        let depth_attachment_references = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::STENCIL_ATTACHMENT_OPTIMAL)
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

        Ok(Self {
            device,
            render_pass,
        })
    }

    pub fn begin(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        extent: vk::Extent2D,
    ) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1f32,
                    stencil: 0,
                },
            },
        ];

        let offset = vk::Offset2D { x: 0, y: 0 };
        let render_pass_begin = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D { offset, extent })
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
        unsafe {
            self.device
                .logical_device
                .destroy_render_pass(self.render_pass, None)
        };
    }
}
