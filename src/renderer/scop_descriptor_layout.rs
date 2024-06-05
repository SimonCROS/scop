use std::collections::HashMap;

use anyhow::{Ok, Result};
use ash::vk;

use super::RendererDevice;

#[derive(Clone)]
pub struct ScopDescriptorSetLayout {
    pub set_layout: vk::DescriptorSetLayout,
    pub bindings: HashMap<u32, vk::DescriptorSetLayoutBinding>,
}

pub struct ScopDescriptorSetLayoutBuilder<'a> {
    device: &'a RendererDevice,
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
}

impl ScopDescriptorSetLayout {
    pub fn builder<'a>(device: &'a RendererDevice) -> ScopDescriptorSetLayoutBuilder {
        ScopDescriptorSetLayoutBuilder {
            device,
            bindings: vec![],
        }
    }

    pub fn cleanup(&mut self, device: &RendererDevice) {
        unsafe {
            device
                .logical_device
                .destroy_descriptor_set_layout(self.set_layout, None);
        }
    }
}

impl<'a> ScopDescriptorSetLayoutBuilder<'a> {
    pub fn add_binding(
        mut self,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        stage_flags: vk::ShaderStageFlags,
    ) -> Self {
        self.bindings.push(
            *vk::DescriptorSetLayoutBinding::builder()
                .binding(binding)
                .descriptor_type(descriptor_type)
                .stage_flags(stage_flags)
                .descriptor_count(1),
        );
        self
    }

    pub fn build(self) -> Result<ScopDescriptorSetLayout> {
        let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&self.bindings);

        let set_layout = unsafe {
            self.device
                .logical_device
                .create_descriptor_set_layout(&create_info, None)?
        };

        let bindings = self.bindings.into_iter().map(|b| (b.binding, b)).collect();

        Ok(ScopDescriptorSetLayout {
            set_layout,
            bindings,
        })
    }
}
