use std::{ops::Deref, rc::Rc};

use anyhow::{Ok, Result};
use ash::vk::{self};

use crate::{
    engine::{Engine, GameObjectId, MaterialId, ResourcesAccessor},
    renderer::{RendererPipeline, ScopDescriptorSetLayout},
};

use super::ScopDescriptorWriter;

pub struct Material {
    pub(crate) pipeline: RendererPipeline,
    pub(crate) material_sets_layouts: Vec<ScopDescriptorSetLayout>,
    vk_material_sets_layouts: Vec<vk::DescriptorSetLayout>,
}

pub struct MaterialInstance {
    pub material: MaterialId,
    pub(crate) material_sets: Vec<vk::DescriptorSet>,
}

// impl Material {
//     pub fn new(
//         renderer: &Renderer,
//         material_sets_layouts: Vec<ScopDescriptorSetLayout>,
//         vert_shader: &Shader,
//         frag_shader: &Shader,
//     ) -> Result<Self> {
//         let vk_material_sets_layouts = material_sets_layouts
//             .iter()
//             .map(|e| e.set_layout)
//             .collect::<Vec<vk::DescriptorSetLayout>>();

//         let mut vk_set_layouts = vec![renderer.global_descriptor_set_layout.set_layout];
//         vk_set_layouts.extend_from_slice(&vk_material_sets_layouts);

//         let pipeline = RendererPipeline::builder(renderer.main_device.clone())
//             .render_pass(&renderer.defaut_render_pass)
//             .vert_shader(vert_shader)
//             .frag_shader(frag_shader)
//             .set_layouts(&vk_set_layouts)
//             .extent(renderer.swapchain.extent)
//             .build();

//         vert_shader.cleanup(&renderer.main_device);
//         frag_shader.cleanup(&renderer.main_device);

//         let pipeline = pipeline?;

//         Ok(Self(Rc::new(MaterialContent {
//             pipeline,
//             material_sets_layouts,
//             vk_material_sets_layouts,
//         })))
//     }

//     pub fn instanciate(&self, renderer: &Renderer) -> Result<MaterialInstance> {
//         let mut material_sets = Vec::with_capacity(renderer.swapchain.image_count);

//         let allocate_info = *vk::DescriptorSetAllocateInfo::builder()
//             .descriptor_pool(renderer.global_descriptor_pool.descriptor_pool)
//             .set_layouts(&self.vk_material_sets_layouts);

//         for _ in 0..renderer.swapchain.image_count {
//             material_sets.extend(unsafe {
//                 renderer
//                     .main_device
//                     .logical_device
//                     .allocate_descriptor_sets(&allocate_info)?
//             });
//         }

//         Ok(MaterialInstance(Rc::new(MaterialInstanceContent {
//             material: self.clone(),
//             material_sets,
//         })))
//     }
// }

impl MaterialInstance {
    pub fn get_material<'a>(&self, resource_accessor: &'a ResourcesAccessor) -> &'a Material {
        resource_accessor
            .get_material(self.material)
            .expect("Material does not exist")
    }

    pub fn writer_for_index<'a>(
        &'a self,
        resource_accessor: &'a ResourcesAccessor,
        set_layout_index: usize,
        index: usize,
    ) -> ScopDescriptorWriter<'a> {
        let material = self.get_material(resource_accessor);

        let mut writer = ScopDescriptorWriter::new(
            &material.pipeline.device,
            &material.material_sets_layouts[set_layout_index],
        );
        writer.descriptors(std::slice::from_ref(self.material_sets.get(index).unwrap()));
        writer
    }

    pub fn writer<'a>(
        &'a self,
        resource_accessor: &'a ResourcesAccessor,
        set_layout_index: usize,
    ) -> ScopDescriptorWriter {
        let material = self.get_material(resource_accessor);

        let mut writer = ScopDescriptorWriter::new(
            &material.pipeline.device,
            &material.material_sets_layouts[set_layout_index],
        );
        writer.descriptors(&self.material_sets);
        writer
    }
}
