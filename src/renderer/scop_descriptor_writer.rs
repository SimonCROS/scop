use std::{collections::HashMap, hash::Hash};

use anyhow::{Ok, Result};
use ash::vk;

use super::{
    RendererDevice, ScopBuffer, ScopDescriptorPool, ScopDescriptorSetLayout, ScopTexture2D,
};

pub struct ScopDescriptorWriter<'a> {
    device: &'a RendererDevice,
    descriptor_set_layout: &'a ScopDescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    buffer_infos: HashMap<u32, vk::DescriptorBufferInfo>,
    image_infos: HashMap<u32, vk::DescriptorImageInfo>,
}

impl<'a> ScopDescriptorWriter<'a> {
    pub fn new(
        device: &'a RendererDevice,
        descriptor_pool: &'a ScopDescriptorPool,
        descriptor_set_layout: &'a ScopDescriptorSetLayout,
    ) -> Result<ScopDescriptorWriter<'a>> {
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool.descriptor_pool)
            .set_layouts(&[descriptor_set_layout.set_layout])
            .build();

        let descriptor_set = unsafe {
            device
                .logical_device
                .allocate_descriptor_sets(&allocate_info)?[0]
        };

        Ok(Self {
            device,
            descriptor_set_layout,
            descriptor_set,
            buffer_infos: HashMap::new(),
            image_infos: HashMap::new(),
        })
    }

    pub fn add_buffer(&mut self, binding: u32, buffer: &ScopBuffer) {
        self.buffer_infos
            .insert(binding, buffer.descriptor_info(buffer.instance_size, 0));
    }

    pub fn add_texture2d(&mut self, binding: u32, texture2d: &ScopTexture2D) {
        self.image_infos
            .insert(binding, texture2d.descriptor_info());
    }

    pub fn write(self) -> vk::DescriptorSet {
        let write_descriptor_sets = self
            .descriptor_set_layout
            .bindings
            .iter()
            .map(|(binding, layout_info)| {
                let mut writer_builder = vk::WriteDescriptorSet::builder()
                    .dst_binding(*binding)
                    .dst_set(self.descriptor_set)
                    .descriptor_type(layout_info.descriptor_type);
                if let Some(buffer) = self.buffer_infos.get(binding) {
                    writer_builder = writer_builder.buffer_info(std::slice::from_ref(buffer))
                } else if let Some(image) = self.image_infos.get(binding) {
                    writer_builder = writer_builder.image_info(std::slice::from_ref(image))
                } else {
                    eprintln!("Binding {} is empty !", binding)
                }

                *writer_builder
            })
            .collect::<Vec<vk::WriteDescriptorSet>>();

        unsafe {
            self.device
                .logical_device
                .update_descriptor_sets(&write_descriptor_sets, &[])
        }

        self.descriptor_set
    }
}
