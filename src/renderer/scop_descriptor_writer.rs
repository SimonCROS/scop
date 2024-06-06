use std::{collections::HashMap, hash::Hash};

use anyhow::{Ok, Result};
use ash::vk;

use super::{RendererDevice, ScopBuffer, ScopDescriptorSetLayout, ScopTexture2D};

pub struct ScopDescriptorWriter<'a> {
    device: &'a RendererDevice,
    descriptor_sets: Option<&'a [vk::DescriptorSet]>,
    set_layout: &'a ScopDescriptorSetLayout,
    buffer_infos: HashMap<u32, vk::DescriptorBufferInfo>,
    image_infos: HashMap<u32, vk::DescriptorImageInfo>,
}

impl<'a> ScopDescriptorWriter<'a> {
    pub fn new(
        device: &'a RendererDevice,
        set_layout: &'a ScopDescriptorSetLayout,
    ) -> ScopDescriptorWriter<'a> {
        Self {
            device,
            set_layout,
            descriptor_sets: None,
            buffer_infos: HashMap::new(),
            image_infos: HashMap::new(),
        }
    }

    pub fn descriptors(&mut self, descriptor_sets: &'a [vk::DescriptorSet]) -> &mut Self {
        self.descriptor_sets = Some(descriptor_sets);
        self
    }

    pub fn set_buffer(&mut self, binding: u32, buffer: &ScopBuffer) -> &mut Self {
        self.buffer_infos
            .insert(binding, buffer.descriptor_info(buffer.instance_size, 0));
        self
    }

    pub fn set_texture2d(&mut self, binding: u32, texture2d: &ScopTexture2D) -> &mut Self {
        self.image_infos
            .insert(binding, texture2d.descriptor_info());
        self
    }

    pub fn write(&self) {
        if self.descriptor_sets.is_none() {
            return;
        }

        let descriptor_sets = self.descriptor_sets.unwrap();

        let mut write_descriptor_sets =
            Vec::with_capacity(self.set_layout.bindings.len() * descriptor_sets.len());

        for set in descriptor_sets {
            for (binding, buffer) in &self.buffer_infos {
                assert!((*binding as usize) < self.set_layout.bindings.len(), "This binding does not exist !");
                write_descriptor_sets.push(
                    *vk::WriteDescriptorSet::builder()
                        .dst_binding(*binding)
                        .dst_set(*set)
                        .descriptor_type(self.set_layout.bindings[binding].descriptor_type)
                        .buffer_info(std::slice::from_ref(buffer)),
                );
            }

            for (binding, image) in &self.image_infos {
                assert!((*binding as usize) < self.set_layout.bindings.len(), "This binding does not exist !");
                write_descriptor_sets.push(
                    *vk::WriteDescriptorSet::builder()
                        .dst_binding(*binding)
                        .dst_set(*set)
                        .descriptor_type(self.set_layout.bindings[binding].descriptor_type)
                        .image_info(std::slice::from_ref(image)),
                );
            }
        }

        unsafe {
            self.device
                .logical_device
                .update_descriptor_sets(write_descriptor_sets.as_slice(), &[])
        }
    }
}
