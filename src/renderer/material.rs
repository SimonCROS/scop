use std::{ops::Deref, rc::Rc};

use anyhow::{Ok, Result};
use ash::vk::{self};

use crate::renderer::{Renderer, RendererPipeline, ScopDescriptorSetLayout, Shader};

use super::ScopDescriptorWriter;

struct MaterialContent {
    pub(crate) pipeline: RendererPipeline,
    pub(crate) material_sets_layouts: Vec<ScopDescriptorSetLayout>,
    vk_material_sets_layouts: Vec<vk::DescriptorSetLayout>,
}

struct MaterialInstanceContent {
    material: Material,
    pub(crate) material_sets: Vec<vk::DescriptorSet>,
}

#[derive(Clone)]
pub struct Material(Rc<MaterialContent>);

#[derive(Clone)]
pub struct MaterialInstance(Rc<MaterialInstanceContent>);

impl Material {
    pub fn new(
        renderer: &Renderer,
        material_sets_layouts: Vec<ScopDescriptorSetLayout>,
        vert_shader: &Shader,
        frag_shader: &Shader,
    ) -> Result<Self> {
        let vk_material_sets_layouts = material_sets_layouts
            .iter()
            .map(|e| e.set_layout)
            .collect::<Vec<vk::DescriptorSetLayout>>();

        let mut vk_set_layouts = vec![renderer.global_descriptor_set_layout.set_layout];
        vk_set_layouts.extend_from_slice(&vk_material_sets_layouts);

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

        Ok(Self(Rc::new(MaterialContent {
            pipeline,
            material_sets_layouts,
            vk_material_sets_layouts,
        })))
    }

    pub fn instanciate(&self, renderer: &Renderer) -> Result<MaterialInstance> {
        let mut material_sets = Vec::with_capacity(renderer.swapchain.image_count);

        let allocate_info = *vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(renderer.global_descriptor_pool.descriptor_pool)
            .set_layouts(&self.vk_material_sets_layouts);

        for _ in 0..renderer.swapchain.image_count {
            material_sets.extend(unsafe {
                renderer
                    .main_device
                    .logical_device
                    .allocate_descriptor_sets(&allocate_info)?
            });
        }

        Ok(MaterialInstance(Rc::new(MaterialInstanceContent {
            material: self.clone(),
            material_sets,
        })))
    }
}

impl MaterialInstance {
    pub fn writer_for_index(&self, set_layout_index: usize, index: usize) -> ScopDescriptorWriter {
        let mut writer = ScopDescriptorWriter::new(
            &self.material.pipeline.device,
            &self.material.material_sets_layouts[set_layout_index],
        );
        writer.descriptors(std::slice::from_ref(self.material_sets.get(index).unwrap()));
        writer
    }

    pub fn writer(&self, set_layout_index: usize) -> ScopDescriptorWriter {
        let mut writer = ScopDescriptorWriter::new(
            &self.material.pipeline.device,
            &self.material.material_sets_layouts[set_layout_index],
        );
        writer.descriptors(&self.material_sets);
        writer
    }
}

impl Deref for MaterialInstanceContent {
    type Target = Material;

    fn deref(&self) -> &Self::Target {
        &self.material
    }
}

impl Deref for Material {
    type Target = MaterialContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for MaterialInstance {
    type Target = MaterialInstanceContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
