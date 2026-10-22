//! com/mojang/renderpearl/api/device/GpuDevice.java
//!
//! Only what one frame needs so far: textures and their views, buffers, the command encoder and a
//! window's surface. Java's `GpuDevice` is the frontend's `FrontendGpuDevice`, which checks each
//! call before the backend's device runs it; until it's ported, the backend's device is the
//! `GpuDevice`.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::commands::command_encoder::CommandEncoder;
use crate::com::mojang::renderpearl::api::device::device_info::DeviceInfo;
use crate::com::mojang::renderpearl::api::device::gpu_surface::GpuSurface;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::org::lwjgl::sdl::sdl_video::SdlWindow;
use std::rc::Rc;

pub trait GpuDevice {
    fn create_command_encoder(&self) -> &dyn CommandEncoder;

    fn create_surface(&self, window: SdlWindow) -> Box<dyn GpuSurface + '_>;

    #[allow(clippy::too_many_arguments)]
    fn create_texture(
        &self,
        label: Option<&str>,
        usage: i32,
        format: GpuFormat,
        width: i32,
        height: i32,
        depth_or_layers: i32,
        mip_levels: i32,
    ) -> Rc<dyn GpuTexture>;

    /// `createTextureView(GpuTexture)`: every mip level.
    fn create_texture_view(&self, texture: &Rc<dyn GpuTexture>) -> GpuTextureView {
        self.create_texture_view_mips(texture, 0, texture.get_mip_levels())
    }

    /// `createTextureView(GpuTexture, int, int)`.
    fn create_texture_view_mips(
        &self,
        texture: &Rc<dyn GpuTexture>,
        base_mip_level: i32,
        mip_levels: i32,
    ) -> GpuTextureView;

    fn create_buffer(&self, label: Option<&str>, usage: i32, size: i64) -> Rc<dyn GpuBuffer>;

    fn get_device_info(&self) -> &DeviceInfo;
}
