//! com/mojang/renderpearl/api/device/GpuSurface.java
//!
//! Only presenting so far: a texture blitted to the window, then shown. Configuring the swap
//! chain (present mode, size) comes with the window loop.

use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;

pub trait GpuSurface {
    fn blit_from_texture(&self, texture_view: &GpuTextureView);

    fn present(&self);
}
