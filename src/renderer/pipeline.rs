use std::{
    ffi,
    mem::{self, offset_of},
};

use anyhow::Result;
use ash::vk;

use super::{device::RendererDevice, shader::Shader, vertex_buffer::Vertex};

pub struct RendererPipeline {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub viewports: [vk::Viewport; 1],
    pub scissors: [vk::Rect2D; 1],
}

impl RendererPipeline {
    pub fn new(
        device: &RendererDevice,
        extent: vk::Extent2D,
        render_pass: vk::RenderPass,
    ) -> Result<RendererPipeline> {
        let vert = Shader::from_code_vert(
            &device.logical_device,
            vk_shader_macros::include_glsl!("./shaders/default.vert"),
        )?;
        let frag = Shader::from_code_frag(
            &device.logical_device,
            vk_shader_macros::include_glsl!("./shaders/default.frag"),
        )?;

        let entry_point = ffi::CString::new("main").unwrap();

        let shader_stages = [
            vert.shader_stage(&entry_point),
            frag.shader_stage(&entry_point),
        ];

        let vertex_input_attribute_descriptions = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
        ];
        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_input_binding_descriptions);

        let (pipeline_layout, pipeline, viewports, scissors) = Self::create_graphics_pipeline(
            &device.logical_device,
            render_pass,
            extent,
            vertex_input_info,
            &shader_stages,
        )?;

        unsafe {
            vert.cleanup(&device.logical_device);
            frag.cleanup(&device.logical_device);
        }

        Ok(RendererPipeline {
            pipeline,
            pipeline_layout,
            viewports,
            scissors
        })
    }

    fn create_graphics_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        vertex_input_info: vk::PipelineVertexInputStateCreateInfoBuilder,
        shader_stages: &[vk::PipelineShaderStageCreateInfo],
    ) -> Result<(vk::PipelineLayout, vk::Pipeline, [vk::Viewport; 1], [vk::Rect2D; 1])> {
        // input:

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        // viewport:

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        }];

        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        // rasterizer:

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(4.2)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        // multisampler:

        let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // color blend:

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build()];

        let color_blend_info =
            vk::PipelineColorBlendStateCreateInfo::builder().attachments(&color_blend_attachments);

        // pipeline:

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder();
        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        let pipeline_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampler_info)
            .color_blend_state(&color_blend_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .build()];

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
                .unwrap()
        }[0];

        Ok((pipeline_layout, pipeline, viewports, scissors))
    }

    pub unsafe fn cleanup(&self, device: &ash::Device) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
    }
}
