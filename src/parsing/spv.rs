use std::{fs::{self, File}, io::Read, mem::size_of};

use anyhow::{ensure, Result};



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
