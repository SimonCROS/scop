use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    mem::size_of,
    rc::Rc,
};

use anyhow::{ensure, Ok, Result};
use ash::vk;

use super::{RendererDevice, ScopBuffer, ScopCommandPool, ScopImage};

pub struct ScopTexture2D {
    device: Rc<RendererDevice>,
    pub image: ScopImage,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
}

#[derive(Default, Debug, Copy, Clone)]
#[repr(packed)]
struct TgaColorMapSpecifications {
    first_entry_index: u16,
    length: u16,
    entry_size: u8,
}

#[derive(Default, Debug, Copy, Clone)]
#[repr(packed)]
struct TgaImageSpecifications {
    x_origin: u16,
    y_origin: u16,
    width: u16,
    height: u16,
    bits_per_pixel: u8,
    image_descriptor: u8,
}

#[derive(Default, Debug)]
#[repr(packed)]
struct TgaHeader {
    id_length: u8,
    color_map_type: u8,
    image_type: u8,
    color_map: TgaColorMapSpecifications,
    image: TgaImageSpecifications,
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

    pub fn from_tga_r8g8b8a8_file(
        device: Rc<RendererDevice>,
        command_pool: &ScopCommandPool,
        path: &'static str,
    ) -> Result<ScopTexture2D> {
        let mut file = File::open(path)?;
        let mut tga_header = TgaHeader::default();
        let tga_header_size = size_of::<TgaHeader>();

        unsafe {
            let p: *mut TgaHeader = &mut tga_header;
            let p: *mut u8 = p as *mut u8;
            file.read_exact(std::slice::from_raw_parts_mut(p, tga_header_size))?;
        }

        ensure!(
            tga_header.color_map_type == 0,
            "The TGA file must not contain a color map"
        );
        ensure!(
            tga_header.image_type == 2,
            "The TGA file must not contain an uncompressed true-color image"
        );
        ensure!(
            tga_header.color_map.first_entry_index
                | tga_header.color_map.length
                | tga_header.color_map.entry_size as u16
                == 0,
            "Invalid TGA file"
        );
        ensure!(
            tga_header.image.x_origin | tga_header.image.y_origin == 0,
            "The TGA file image origin should be at [0,0] from the bottom left"
        );
        ensure!(
            tga_header.image.width > 0 && tga_header.image.height > 0,
            "Invalid TGA file"
        );
        ensure!(
            tga_header.image.bits_per_pixel == 32,
            "The TGA file must contain 32 bits per pixel"
        );
        ensure!(
            tga_header.image.image_descriptor == 0b00001000,
            "The TGA file must contain 8 bits for alpha, and be in bottom-to-top, left-to-right order"
        );

        let bytes_per_pixel = (tga_header.image.bits_per_pixel / 8) as usize;

        let content_len =
            tga_header.image.width as usize * tga_header.image.height as usize * bytes_per_pixel;

        let mut bytes = vec![0u8; content_len];
        file.read_exact(&mut bytes)?;

        ScopTexture2D::new(
            device,
            command_pool,
            &bytes,
            tga_header.image.width as u32,
            tga_header.image.height as u32,
            vk::Format::B8G8R8A8_UNORM,
            tga_header.image.bits_per_pixel as u16,
        )
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
