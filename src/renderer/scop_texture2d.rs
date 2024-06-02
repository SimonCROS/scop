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

#[derive(Default, Debug)]
#[repr(packed)]
struct BmpHeader {
    file_type: u16,
    file_size: u32,
    reserved1: u16,
    reserved2: u16,
    offset_data: u32,
}

#[derive(Default, Debug)]
#[repr(packed)]
struct DibHeader {
    header_size: u32,
    width: u32,
    height: u32,
    planes: u16,
    bits_per_pixel: u16,
    compression_method: u32,
    bitmap_size: u32,
    horizontal_resolution: u32,
    vertical_resolution: u32,
    color_palette_size: u32,
    important_colors_used: u32,
}

impl ScopTexture2D {
    pub fn new(
        device: Rc<RendererDevice>,
        command_pool: &ScopCommandPool,
        data: &[u8],
        width: u32,
        height: u32,
        bits_per_pixel: u16,
    ) -> Result<Self> {
        ensure!(bits_per_pixel % 8 == 0, "bits_per_pixel should be a multiple of 8");
        let size = width as vk::DeviceSize * height as vk::DeviceSize * (bits_per_pixel / 8) as vk::DeviceSize;
        let mut staging_buffer = ScopBuffer::new(
            device.clone(),
            1,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            1,
        )?;

        staging_buffer.map(vk::WHOLE_SIZE, 0)?;
        staging_buffer.write_to_buffer(data, 0);
        staging_buffer.unmap();

        let mut image = ScopImage::new(
            device.clone(),
            vk::Format::R8G8B8A8_SNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            width,
            height,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        image.change_layout(command_pool, vk::ImageLayout::TRANSFER_DST_OPTIMAL)?;
        staging_buffer.copy_to_image(command_pool, &image)?;
        image.change_layout(command_pool, vk::ImageLayout::READ_ONLY_OPTIMAL)?;

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

    pub fn from_bmp_r8g8b8a8_file(
        device: Rc<RendererDevice>,
        command_pool: &ScopCommandPool,
        path: &'static str,
    ) -> Result<ScopTexture2D> {
        let mut file = File::open(path)?;
        let mut bmp_header = BmpHeader::default();
        let mut dib_header = DibHeader::default();
        let bmp_header_size = size_of::<BmpHeader>();
        let dib_header_size = size_of::<DibHeader>();

        unsafe {
            let p: *mut BmpHeader = &mut bmp_header;
            let p: *mut u8 = p as *mut u8;
            file.read_exact(std::slice::from_raw_parts_mut(p, bmp_header_size))?;
        }
        unsafe {
            let p: *mut DibHeader = &mut dib_header;
            let p: *mut u8 = p as *mut u8;
            file.read_exact(std::slice::from_raw_parts_mut(p, dib_header_size))?;
        }

        ensure!(
            dib_header.bits_per_pixel == 32,
            "BMP bits per pixel should be 32"
        );
        ensure!(dib_header.width > 0, "BMP width be > 0");
        ensure!(dib_header.height > 0, "BMP height be > 0");

        file.seek(SeekFrom::Start(bmp_header.offset_data as u64))?;

        let content_len = dib_header.width as usize
            * dib_header.height as usize
            * dib_header.bits_per_pixel as usize;
        let mut bytes = Vec::<u8>::with_capacity(content_len);
        file.read_exact(&mut bytes)?;

        ScopTexture2D::new(
            device,
            command_pool,
            &bytes,
            dib_header.width as u32,
            dib_header.height as u32,
            dib_header.bits_per_pixel,
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
