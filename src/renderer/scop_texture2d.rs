use core::slice;
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    mem::{size_of, MaybeUninit},
    rc::Rc,
};

use anyhow::{bail, ensure, Context, Ok, Result};
use ash::vk;

use super::{scop_command_pool, RendererDevice, ScopBuffer, ScopCommandPool, ScopImage};

pub struct ScopTexture2D {
    pub image: ScopImage,
    pub image_view: vk::ImageView,
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
    ) -> Result<Self> {
        let size = (width as vk::DeviceSize) * (height as vk::DeviceSize);
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
            device,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            width,
            height,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        staging_buffer.copy_to_image(command_pool, &image)?;
        image.change_layout(command_pool, vk::ImageLayout::READ_ONLY_OPTIMAL)?;

        staging_buffer.cleanup();

        let image_view = image.create_image_view(vk::ImageAspectFlags::COLOR)?;

        Ok(Self { image, image_view })
    }

    pub fn from_bmp_r32g32b32_file(
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

        dbg!(&bmp_header);
        dbg!(&dib_header);

        ensure!(
            dib_header.bits_per_pixel == 24,
            "BMP bits per pixel should be 24"
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
        )
    }

    pub fn cleanup(&mut self) {
        self.image.cleanup_image_view(self.image_view);
        self.image.cleanup();
    }
}
