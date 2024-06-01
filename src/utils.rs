use std::{
    fs::{self, File},
    io::Read, mem::size_of,
};

use anyhow::{ensure, Result};

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

pub fn read_shader(filename: &str) -> Result<Vec<u32>> {
    let mut f = File::open(&filename)?;
    let metadata = fs::metadata(&filename)?;

    ensure!(
        metadata.len() % 4 == 0,
        "Spir-V shader code len should be a multpile of 4"
    );

    let len = metadata.len() as usize;

    unsafe {
        let mut buffer = vec![0u32; len / size_of::<u32>()];
        let bytes: &mut [u8] = std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, len);
        f.read(bytes)?;
    
        Ok(buffer)
    }
}
