use anyhow::{bail, Context, Ok, Result};
use ash::vk;

use super::ScopImage;

pub struct ScopTexture2D {
    pub image: ScopImage,
    pub image_view: vk::ImageView,
}

impl ScopTexture2D {
    pub fn cleanup(&mut self) {
        self.image.cleanup_image_view(self.image_view);
        self.image.cleanup();
    }
}
