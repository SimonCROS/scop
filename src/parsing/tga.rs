use std::{
    fs::File,
    io::{Read, Seek},
    mem::size_of,
    rc::Rc,
};

use anyhow::{ensure, Result};
use ash::vk;

use crate::renderer::{RendererDevice, ScopCommandPool, ScopTexture2D};

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

pub fn read_tga_r8g8b8a8_file(
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

    file.seek_relative(tga_header.id_length as i64)?; // Skip id field

    let bytes_per_pixel = (tga_header.image.bits_per_pixel / 8) as usize;

    let content_len =
        tga_header.image.width as usize * tga_header.image.height as usize * bytes_per_pixel;

    let mut bytes = vec![0u8; content_len];
    file.read_exact(&mut bytes)?;

    // if tga_header.image.height > 1 {
    //     let half_len = content_len / 2;
    //     let (left, right) = bytes.split_at_mut(half_len);
    //     let width = tga_header.image.width as usize * bytes_per_pixel;

    //     for i in (0..half_len).step_by(width as usize) {
    //         if i <= half_len - width {
    //             // Greater when height is odd
    //             left[i..i + width].swap_with_slice(&mut right[half_len - i - width..half_len - i])
    //         }
    //     }
    // }

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
