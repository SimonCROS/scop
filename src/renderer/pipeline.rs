use core::slice;
use std::{ffi, mem, rc::Rc};

use anyhow::{ensure, Result};
use ash::vk::{self, PushConstantRange, ShaderStageFlags};

use crate::{
    engine::mesh::Vertex,
    math::{Matrix3, Matrix4},
};

use super::{RendererDevice, ScopRenderPass, Shader};

pub struct SimplePushConstantData {
    pub model_matrix: Matrix4,
    pub normal_matrix: Matrix3,
}

#[derive(Copy, Clone)]
pub struct ScopGpuCameraData {
    pub projection: Matrix4,
    pub view: Matrix4,
}

pub struct RendererPipeline {
    pub device: Rc<RendererDevice>,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}

pub struct ScopPipelineBuilder<'a> {
    device: Rc<RendererDevice>,
    render_pass: Option<&'a ScopRenderPass>,
    vert_shader: Option<Shader>,
    frag_shader: Option<Shader>,
    set_layouts: &'a [vk::DescriptorSetLayout],
    extent: Option<vk::Extent2D>,
}

impl RendererPipeline {
    pub fn builder<'a>(device: Rc<RendererDevice>) -> ScopPipelineBuilder<'a> {
        ScopPipelineBuilder {
            device,
            render_pass: None,
            vert_shader: None,
            frag_shader: None,
            extent: None,
            set_layouts: &[],
        }
    }

    pub fn new(
        device: Rc<RendererDevice>,
        extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        set_layouts: &[vk::DescriptorSetLayout],
        shader_stages: &[vk::PipelineShaderStageCreateInfo],
    ) -> Result<RendererPipeline> {
        let vertex_input_attribute_descriptions = Vertex::get_vertex_input_attribute_descriptions();
        let vertex_input_binding_descriptions = Vertex::get_vertex_input_binding_descriptions();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(vertex_input_attribute_descriptions.as_slice())
            .vertex_binding_descriptions(vertex_input_binding_descriptions.as_slice());

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
            .cull_mode(vk::CullModeFlags::BACK)
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
            .push_constant_ranges(slice::from_ref(&push_constant_range))
            .set_layouts(set_layouts);
        let pipeline_layout = unsafe {
            device
                .logical_device
                .create_pipeline_layout(&pipeline_layout_info, None)?
        };

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0f32)
            .max_depth_bounds(1f32)
            .stencil_test_enable(false);

        let pipeline_infos = [*vk::GraphicsPipelineCreateInfo::builder()
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
            .subpass(0)];

        let pipeline = unsafe {
            device
                .logical_device
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
                .unwrap()
        }[0];

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

    pub fn bind_descriptor_sets(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
        descriptor_sets: &[vk::DescriptorSet],
    ) {
        unsafe {
            self.device.logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                pipeline_bind_point,
                self.pipeline_layout,
                0,
                descriptor_sets,
                &[],
            )
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
}

impl<'a> ScopPipelineBuilder<'a> {
    pub fn render_pass(mut self, render_pass: &'a ScopRenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }

    pub fn vert_shader(mut self, shader: Shader) -> Self {
        self.vert_shader = Some(shader);
        self
    }

    pub fn frag_shader(mut self, shader: Shader) -> Self {
        self.frag_shader = Some(shader);
        self
    }

    pub fn set_layouts(mut self, set_layouts: &'a [vk::DescriptorSetLayout]) -> Self {
        self.set_layouts = set_layouts;
        self
    }

    pub fn extent(mut self, extent: vk::Extent2D) -> Self {
        self.extent = Some(extent);
        self
    }

    pub fn build(self) -> Result<RendererPipeline> {
        ensure!(
            self.render_pass.is_some(),
            "ScopPipelineBuilder: No render pass"
        );
        ensure!(
            self.vert_shader
                .is_some_and(|s| s.stage.contains(vk::ShaderStageFlags::VERTEX)),
            "ScopPipelineBuilder: No vertex shader, or does not contains vertex stage"
        );
        ensure!(
            self.frag_shader
                .is_some_and(|s| s.stage.contains(vk::ShaderStageFlags::FRAGMENT)),
            "ScopPipelineBuilder: No fragment shader, or does not contains fragment stage"
        );
        ensure!(self.extent.is_some(), "ScopPipelineBuilder: No extent");

        let entry_point = ffi::CString::new("main")?;
        let shader_stages = [
            self.vert_shader.unwrap().shader_stage(&entry_point),
            self.frag_shader.unwrap().shader_stage(&entry_point),
        ];

        RendererPipeline::new(
            self.device,
            self.extent.unwrap(),
            self.render_pass.unwrap().render_pass,
            self.set_layouts,
            &shader_stages,
        )
    }
}
