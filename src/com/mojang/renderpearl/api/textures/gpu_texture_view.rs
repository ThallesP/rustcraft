//! com/mojang/renderpearl/api/textures/GpuTextureView.java
//!
//! Java's is an interface each backend implements (`GlTextureView` on `BaseGpuTextureView`);
//! neither backend's adds anything a frame needs yet, so it's this struct.

use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use std::rc::Rc;

/// A range of a texture's mip levels.
#[derive(Clone)]
pub struct GpuTextureView {
    texture: Rc<dyn GpuTexture>,
    base_mip_level: i32,
    mip_levels: i32,
}

impl GpuTextureView {
    pub fn new(
        texture: Rc<dyn GpuTexture>,
        base_mip_level: i32,
        mip_levels: i32,
    ) -> GpuTextureView {
        GpuTextureView {
            texture,
            base_mip_level,
            mip_levels,
        }
    }

    pub fn texture(&self) -> &Rc<dyn GpuTexture> {
        &self.texture
    }

    pub fn base_mip_level(&self) -> i32 {
        self.base_mip_level
    }

    pub fn mip_levels(&self) -> i32 {
        self.mip_levels
    }

    pub fn get_width(&self, mip_level: i32) -> i32 {
        self.texture.get_width(mip_level + self.base_mip_level)
    }

    pub fn get_height(&self, mip_level: i32) -> i32 {
        self.texture.get_height(mip_level + self.base_mip_level)
    }
}
