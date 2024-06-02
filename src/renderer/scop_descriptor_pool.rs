use std::rc::Rc;

use anyhow::{Ok, Result};
use ash::vk;

use super::{RendererDevice, ScopBuffer, ScopDescriptorSetLayout};

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

    pub fn write_buffer(
        &self,
        binding: u32,
        descriptor_layout: &ScopDescriptorSetLayout,
        buffer: &ScopBuffer,
    ) -> Result<vk::DescriptorSet> {
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.descriptor_pool)
            .set_layouts(&[descriptor_layout.set_layout])
            .build();

        let descriptor_set = unsafe {
            self.device
                .logical_device
                .allocate_descriptor_sets(&allocate_info)?[0]
        };

        let descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer.buffer)
            .offset(0)
            .range(buffer.instance_size)
            .build();

        let descriptor_type = descriptor_layout.bindings[&binding].descriptor_type;

        let write_descriptor_set = vk::WriteDescriptorSet::builder()
            .dst_binding(binding)
            .dst_set(descriptor_set)
            .buffer_info(&[descriptor_buffer_info])
            .descriptor_type(descriptor_type)
            .build();

        unsafe {
            self.device
                .logical_device
                .update_descriptor_sets(&[write_descriptor_set], &[])
        }

        Ok(descriptor_set)
    }

    pub fn write_buffers(
        &self,
        binding: u32,
        descriptor_layout: &ScopDescriptorSetLayout,
        buffers: &[ScopBuffer],
    ) -> Result<Vec<vk::DescriptorSet>> {
        let mut descriptor_sets = Vec::<vk::DescriptorSet>::with_capacity(buffers.len());
        for buffer in buffers {
            descriptor_sets.push(self.write_buffer(binding, descriptor_layout, buffer)?);
        }
        Ok(descriptor_sets)
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
