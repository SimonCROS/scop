use anyhow::{Ok, Result};
use ash::vk::{self};

use crate::{
    parsing::read_spv_file,
    renderer::{Renderer, RendererPipeline, ScopDescriptorSetLayout, Shader},
};

use super::ScopDescriptorWriter;

pub struct Material {
    pub pipeline: RendererPipeline,
    pub material_sets_layouts: Vec<ScopDescriptorSetLayout>,
    pub material_sets: Vec<vk::DescriptorSet>,
}

impl Material {
    pub fn new(renderer: &Renderer, material_sets_layouts: Vec<ScopDescriptorSetLayout>) -> Result<Self> {
        let vert_shader = Shader::from_code(
            &renderer.main_device,
            &read_spv_file("./shaders/default.vert.spv")?,
            vk::ShaderStageFlags::VERTEX,
        )?;
        let frag_shader = Shader::from_code(
            &renderer.main_device,
            &read_spv_file("./shaders/default.frag.spv")?,
            vk::ShaderStageFlags::FRAGMENT,
        )?;

        let mut vk_material_sets_layouts = material_sets_layouts.iter().map(|e| e.set_layout).collect::<Vec<vk::DescriptorSetLayout>>();

        let mut material_sets = Vec::with_capacity(renderer.swapchain.image_count);
        for _ in 0..renderer.swapchain.image_count {
            let allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(renderer.global_descriptor_pool.descriptor_pool)
                .set_layouts(&vk_material_sets_layouts)
                .build();

            material_sets.push(unsafe {
                renderer
                    .main_device
                    .logical_device
                    .allocate_descriptor_sets(&allocate_info)?[0]
            });
        }

        let mut vk_set_layouts = vec![renderer.global_descriptor_set_layout.set_layout];
        vk_set_layouts.append(&mut vk_material_sets_layouts);

        let pipeline = RendererPipeline::builder(renderer.main_device.clone())
            .render_pass(&renderer.defaut_render_pass)
            .vert_shader(vert_shader)
            .frag_shader(frag_shader)
            .set_layouts(&vk_set_layouts)
            .extent(renderer.swapchain.extent)
            .build();

        vert_shader.cleanup(&renderer.main_device);
        frag_shader.cleanup(&renderer.main_device);

        let pipeline = pipeline?;

        Ok(Self {
            pipeline,
            material_sets_layouts,
            material_sets,
        })
    }

    pub fn get_writer_for(&self, set_layout_index: usize, index: usize) -> ScopDescriptorWriter {
        let mut writer =
            ScopDescriptorWriter::new(&self.pipeline.device, &self.material_sets_layouts[set_layout_index]);
        writer.descriptors(std::slice::from_ref(
            self.material_sets.get(index).unwrap(),
        ));
        writer
    }

    pub fn get_writer_for_all(&self, set_layout_index: usize) -> ScopDescriptorWriter {
        let mut writer =
            ScopDescriptorWriter::new(&self.pipeline.device, &self.material_sets_layouts[set_layout_index]);
        writer.descriptors(&self.material_sets);
        writer
    }
}
