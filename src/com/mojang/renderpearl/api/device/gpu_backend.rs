//! com/mojang/renderpearl/api/device/GpuBackend.java

use crate::com::mojang::renderpearl::api::device::backend_creation_exception::BackendCreationException;
use crate::com::mojang::renderpearl::api::device::gpu_debug_options::GpuDebugOptions;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::org::lwjgl::sdl::sdl_video::SdlWindow;

/// A graphics API the game can render with: it loads its library, opens windows the way the API
/// needs them, and makes the device.
pub trait GpuBackend {
    fn get_name(&self) -> &'static str;

    fn load_library(&mut self) -> Result<(), BackendCreationException>;

    fn unload_library(&mut self);

    /// An SDL window, or null when SDL can't make one (`SDL_GetError` says why).
    fn create_window(&self, title: Option<&str>, width: i32, height: i32, flags: u64) -> SdlWindow;

    fn create_device(
        &self,
        debug_options: &GpuDebugOptions,
    ) -> Result<Box<dyn GpuDevice>, BackendCreationException>;
}
