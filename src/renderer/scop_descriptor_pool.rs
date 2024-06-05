use std::rc::Rc;

use anyhow::{Ok, Result};
use ash::vk;

use super::RendererDevice;

pub struct ScopDescriptorPool {
    device: Rc<RendererDevice>,
    pub descriptor_pool: vk::DescriptorPool,
}

pub struct ScopDescriptorPoolBuilder<'a> {
    device: &'a Rc<RendererDevice>,
    pub max_sets: u32,
    pub sizes: Vec<vk::DescriptorPoolSize>,
}

impl ScopDescriptorPool {
    pub fn builder<'a>(device: &'a Rc<RendererDevice>) -> ScopDescriptorPoolBuilder {
        ScopDescriptorPoolBuilder {
            device,
            sizes: vec![],
            max_sets: 0,
        }
    }

    pub fn cleanup(&mut self) {
        unsafe {
            self.device
                .logical_device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

impl<'a> ScopDescriptorPoolBuilder<'a> {
    pub fn add_size(mut self, descriptor_type: vk::DescriptorType, descriptor_count: u32) -> Self {
        self.sizes.push(
            *vk::DescriptorPoolSize::builder()
                .ty(descriptor_type)
                .descriptor_count(descriptor_count),
        );
        self
    }

    pub fn max_sets(mut self, max_sets: u32) -> Self {
        self.max_sets = max_sets;
        self
    }

    pub fn build(self) -> Result<ScopDescriptorPool> {
        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&self.sizes)
            .max_sets(self.max_sets);

        let descriptor_pool = unsafe {
            self.device
                .logical_device
                .create_descriptor_pool(&create_info, None)?
        };

        Ok(ScopDescriptorPool {
            device: self.device.clone(),
            descriptor_pool,
        })
    }
}
