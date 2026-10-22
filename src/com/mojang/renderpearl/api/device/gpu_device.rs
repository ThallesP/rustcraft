//! com/mojang/renderpearl/api/device/GpuDevice.java
//!
//! Textures and their views, samplers, buffers, pipelines, the command encoder and a window's
//! surface. Java's `GpuDevice` is the frontend's `FrontendGpuDevice`, which checks each call
//! before the backend's device runs it; until it's ported, the backend's device is the
//! `GpuDevice`, and `compile_pipeline` is the backend's (`GpuDeviceBackend.compilePipeline` and
//! the pending pipeline's `finishCompile`), which `PipelineBuilder::compile_pipeline` calls like
//! the frontend's does.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::commands::command_encoder::CommandEncoder;
use crate::com::mojang::renderpearl::api::device::device_info::DeviceInfo;
use crate::com::mojang::renderpearl::api::device::gpu_surface::GpuSurface;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::{
    BackendRenderPipeline, CreateInfo,
};
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

    /// `createBuffer(Supplier, int, ByteBuffer)`: a buffer holding `data`.
    fn create_buffer_with_data(
        &self,
        label: Option<&str>,
        usage: i32,
        data: &[u8],
    ) -> Rc<dyn GpuBuffer>;

    /// `createSampler(...)`, with `maxLod` as Java's `OptionalDouble`.
    fn create_sampler(
        &self,
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        min_filter: FilterMode,
        mag_filter: FilterMode,
        max_anisotropy: i32,
        max_lod: Option<f64>,
    ) -> Rc<dyn GpuSampler>;

    /// The backend's pipeline for the create info, or `None` where Java logs why it couldn't
    /// make one and returns null.
    fn compile_pipeline(&self, create_info: &CreateInfo) -> Option<Rc<dyn BackendRenderPipeline>>;

    fn get_device_info(&self) -> &DeviceInfo;
}
