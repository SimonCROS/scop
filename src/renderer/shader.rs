use std::{ffi, fs::{self, File}, io::Read, mem::size_of};

use anyhow::{ensure, Result};
use ash::vk;

pub struct Shader {
    pub shader_module: vk::ShaderModule,
    pub stage: vk::ShaderStageFlags,
}

impl Shader {
    pub fn from_code(
        device: &ash::Device,
        code: &[u32],
        stage: vk::ShaderStageFlags,
    ) -> Result<Self> {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code);

        let shader_module = unsafe { device.create_shader_module(&create_info, None) }?;

        Ok(Self {
            shader_module,
            stage,
        })
    }

    pub fn from_code_vert(device: &ash::Device, code: &[u32]) -> Result<Self> {
        Self::from_code(device, code, vk::ShaderStageFlags::VERTEX)
    }

    pub fn from_code_frag(device: &ash::Device, code: &[u32]) -> Result<Self> {
        Self::from_code(device, code, vk::ShaderStageFlags::FRAGMENT)
    }

    pub fn read_spv_file(path: &str) -> Result<Vec<u32>> {
        let mut f = File::open(&path)?;
        let metadata = fs::metadata(&path)?;

        ensure!(
            metadata.len() % 4 == 0,
            "Spir-V shader code len should be a multpile of 4"
        );

        let len = metadata.len() as usize;

        unsafe {
            let mut buffer = vec![0u32; len / size_of::<u32>()];
            let bytes: &mut [u8] = std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, len);
            f.read_exact(bytes)?;
        
            Ok(buffer)
        }
    }

    pub fn shader_stage(&self, entry_point: &ffi::CString) -> vk::PipelineShaderStageCreateInfo {
        let create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(self.stage)
            .module(self.shader_module)
            .name(entry_point);

        create_info.build()
    }

    pub unsafe fn cleanup(&self, device: &ash::Device) {
        device.destroy_shader_module(self.shader_module, None);
    }
}
