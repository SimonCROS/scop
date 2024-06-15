use std::ffi;

use ash::vk;

use crate::utils::Result;

use super::RendererDevice;

#[derive(Copy, Clone)]
pub struct Shader {
    pub shader_module: vk::ShaderModule,
    pub stage: vk::ShaderStageFlags,
}

impl Shader {
    pub fn from_code(
        device: &RendererDevice,
        code: &[u32],
        stage: vk::ShaderStageFlags,
    ) -> Result<Self> {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code);

        let shader_module = unsafe {
            device
                .logical_device
                .create_shader_module(&create_info, None)
        }?;

        Ok(Self {
            shader_module,
            stage,
        })
    }

    pub fn shader_stage(&self, entry_point: &ffi::CString) -> vk::PipelineShaderStageCreateInfo {
        let create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(self.stage)
            .module(self.shader_module)
            .name(entry_point);

        create_info.build()
    }

    pub fn cleanup(&self, device: &RendererDevice) {
        unsafe {
            device
                .logical_device
                .destroy_shader_module(self.shader_module, None)
        };
    }
}
