use anyhow::{Ok, Result};
use ash::vk;

use super::{
    RendererDevice, ScopBuffer, ScopDescriptorPool, ScopDescriptorSetLayout,
    ScopTexture2D,
};

pub struct ScopDescriptorWriter<'a> {
    device: &'a RendererDevice,
    descriptor_pool: &'a ScopDescriptorPool,
    descriptor_set_layout: &'a ScopDescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    write_descriptor_sets: Vec<vk::WriteDescriptorSetBuilder<'a>>,
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
            descriptor_pool,
            descriptor_set_layout,
            descriptor_set,
            write_descriptor_sets: vec![],
        })
    }

    pub fn add_buffer(&mut self, binding: u32, buffer: &ScopBuffer) {
        let descriptor_buffer_info = buffer.descriptor_info(buffer.instance_size, 0);

        let descriptor_type = self.descriptor_set_layout.bindings[&binding].descriptor_type;

        let write_descriptor_set = vk::WriteDescriptorSet::builder()
            .dst_binding(binding)
            .dst_set(self.descriptor_set)
            .buffer_info(&[descriptor_buffer_info])
            .descriptor_type(descriptor_type);

        self.write_descriptor_sets.push(write_descriptor_set);
    }

    pub fn add_texture2d(&mut self, binding: u32, texture2d: &ScopTexture2D) {
        let descriptor_image_info = texture2d.descriptor_info();

        let descriptor_type = self.descriptor_set_layout.bindings[&binding].descriptor_type;

        let write_descriptor_set = vk::WriteDescriptorSet::builder()
            .dst_binding(binding)
            .dst_set(self.descriptor_set)
            .image_info(&[descriptor_image_info])
            .descriptor_type(descriptor_type);

        self.write_descriptor_sets.push(write_descriptor_set);
    }

    pub fn write(self) -> vk::DescriptorSet {
        unsafe {
            self.device
                .logical_device
                .update_descriptor_sets(&self.write_descriptor_sets, &[])
        }

        self.descriptor_set
    }

    // pub fn write_buffers(
    //     &self,
    //     binding: u32,
    //     descriptor_set_layout: &ScopDescriptorSetLayout,
    //     buffers: &[ScopBuffer],
    // ) -> Result<Vec<vk::DescriptorSet>> {
    //     let mut descriptor_sets = Vec::<vk::DescriptorSet>::with_capacity(buffers.len());
    //     for buffer in buffers {
    //         descriptor_sets.push(self.write_buffer(binding, descriptor_set_layout, buffer)?);
    //     }
    //     Ok(descriptor_sets)
    // }
}
