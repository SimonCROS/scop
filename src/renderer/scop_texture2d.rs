use std::rc::Rc;

use ash::vk;

use crate::{ensure, utils::Result};

use super::{RendererDevice, ScopBuffer, ScopCommandPool, ScopImage};

pub struct ScopTexture2D {
    device: Rc<RendererDevice>,
    pub image: ScopImage,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl ScopTexture2D {
    pub fn new(
        device: Rc<RendererDevice>,
        command_pool: &ScopCommandPool,
        data: &[u8],
        width: u32,
        height: u32,
        image_format: vk::Format,
        bits_per_pixel: u16,
    ) -> Result<Self> {
        ensure!(
            bits_per_pixel % 8 == 0,
            "bits_per_pixel should be a multiple of 8"
        );

        let size = width as usize * height as usize * (bits_per_pixel / 8) as usize;

        ensure!(data.len() == size as usize, "data is not the write size");

        let mut staging_buffer = ScopBuffer::new(
            device.clone(),
            size,
            1,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            1,
        )?;

        staging_buffer.map(vk::WHOLE_SIZE, 0)?;
        staging_buffer.write_to_buffer(data, 0);
        staging_buffer.unmap();

        let mut image = ScopImage::new(
            device.clone(),
            image_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            width,
            height,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        image.change_layout(command_pool, vk::ImageLayout::TRANSFER_DST_OPTIMAL)?;
        staging_buffer.copy_to_image(command_pool, &image)?;
        image.change_layout(command_pool, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)?;

        staging_buffer.cleanup();

        let image_view = image.create_image_view(vk::ImageAspectFlags::COLOR)?;

        let sampler_create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR);

        let sampler = unsafe {
            device
                .logical_device
                .create_sampler(&sampler_create_info, None)?
        };

        Ok(Self {
            device,
            image,
            image_view,
            sampler,
        })
    }

    pub fn descriptor_info(&self) -> vk::DescriptorImageInfo {
        vk::DescriptorImageInfo::builder()
            .image_layout(self.image.layout)
            .image_view(self.image_view)
            .sampler(self.sampler)
            .build()
    }

    pub fn cleanup(&mut self) {
        unsafe {
            self.device
                .logical_device
                .destroy_sampler(self.sampler, None)
        };
        self.image.cleanup_image_view(self.image_view);
        self.image.cleanup();
    }
}
