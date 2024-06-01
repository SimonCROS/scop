use core::slice;
use std::{ffi, mem, rc::Rc};

use anyhow::Result;
use ash::vk::{self, PushConstantRange, ShaderStageFlags};

use crate::{
    engine::mesh::Vertex,
    math::{Matrix3, Matrix4},
    utils::read_shader,
};

use super::{device::RendererDevice, shader::Shader};

pub struct SimplePushConstantData {
    pub model_matrix: Matrix4,
    pub normal_matrix: Matrix3,
}

#[derive(Copy, Clone)]
pub struct ScopGlobalUbo {
    pub projection_matrix: Matrix4,
    pub view_matrix: Matrix4,
    pub inverse_view_matrix: Matrix4,
}

pub struct RendererPipeline {
    device: Rc<RendererDevice>,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}

impl RendererPipeline {
    pub fn new(
        device: Rc<RendererDevice>,
        extent: vk::Extent2D,
        render_pass: vk::RenderPass,
    ) -> Result<RendererPipeline> {
        let vert = Shader::from_code_vert(
            &device.logical_device,
            &read_shader("./shaders/default.vert.spv")?,
        )?;
        let frag = Shader::from_code_frag(
            &device.logical_device,
            &read_shader("./shaders/default.frag.spv")?,
        )?;

        let entry_point = ffi::CString::new("main").unwrap();

        let shader_stages = [
            vert.shader_stage(&entry_point),
            frag.shader_stage(&entry_point),
        ];

        let (pipeline_layout, pipeline) = {
            let vertex_input_attribute_descriptions =
                Vertex::get_vertex_input_attribute_descriptions();
            let vertex_input_binding_descriptions = Vertex::get_vertex_input_binding_descriptions();
            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_attribute_descriptions(vertex_input_attribute_descriptions.as_slice())
                .vertex_binding_descriptions(vertex_input_binding_descriptions.as_slice());

            Self::create_graphics_pipeline(
                &device.logical_device,
                render_pass,
                extent,
                vertex_input_info,
                &shader_stages,
            )?
        };

        unsafe {
            vert.cleanup(&device.logical_device);
            frag.cleanup(&device.logical_device);
        }

        Ok(RendererPipeline {
            device,
            pipeline,
            pipeline_layout,
        })
    }

    pub fn bind(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
    ) {
        unsafe {
            self.device.logical_device.cmd_bind_pipeline(
                command_buffer,
                pipeline_bind_point,
                self.pipeline,
            );
        }
    }

    pub fn cleanup(&self) {
        unsafe {
            self.device
                .logical_device
                .destroy_pipeline(self.pipeline, None);
            self.device
                .logical_device
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }

    fn create_graphics_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        vertex_input_info: vk::PipelineVertexInputStateCreateInfoBuilder,
        shader_stages: &[vk::PipelineShaderStageCreateInfo],
    ) -> Result<(vk::PipelineLayout, vk::Pipeline)> {
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
            .line_width(1f32)
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

        let push_constant_range = PushConstantRange::builder()
            .stage_flags(ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(mem::size_of::<SimplePushConstantData>() as u32);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(slice::from_ref(&push_constant_range));
        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0f32)
            .max_depth_bounds(1f32)
            .stencil_test_enable(false);

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
            .depth_stencil_state(&depth_stencil_state)
            .subpass(0)
            .build()];

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
                .unwrap()
        }[0];

        Ok((pipeline_layout, pipeline))
    }
}
