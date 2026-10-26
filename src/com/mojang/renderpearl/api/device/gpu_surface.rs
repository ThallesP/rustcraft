//! com/mojang/renderpearl/api/device/GpuSurface.java
//!
//! Presenting: a texture blitted to the window, then shown, and the present mode, which vsync
//! picks. The swap chain's size comes with the window loop; the GL backend's is the window's.

use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;

pub trait GpuSurface {
    /// `configure`'s present mode (the size is the window's).
    fn configure(&self, present_mode: PresentMode);

    fn supported_present_modes(&self) -> &[PresentMode];

    fn blit_from_texture(&self, texture_view: &GpuTextureView);

    fn present(&self);
}

/// `GpuSurface.PresentMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentMode {
    IMMEDIATE,
    MAILBOX,
    FIFO,
    FIFO_RELAXED,
}

impl PresentMode {
    /// `values()`.
    pub const VALUES: [PresentMode; 4] = [
        PresentMode::IMMEDIATE,
        PresentMode::MAILBOX,
        PresentMode::FIFO,
        PresentMode::FIFO_RELAXED,
    ];
}

impl PresentMode {
    const PRESENT_MODES_VSYNC: [PresentMode; 1] = [PresentMode::FIFO];
    const PRESENT_MODES_NO_VSYNC: [PresentMode; 3] = [
        PresentMode::IMMEDIATE,
        PresentMode::MAILBOX,
        PresentMode::FIFO,
    ];

    /// `getSupportedVsyncMode`: the first mode of vsync's preference the surface supports.
    /// Panics where Java throws `IllegalStateException`.
    pub fn get_supported_vsync_mode(supported_modes: &[PresentMode], vsync: bool) -> PresentMode {
        let preferred: &[PresentMode] = if vsync {
            &PresentMode::PRESENT_MODES_VSYNC
        } else {
            &PresentMode::PRESENT_MODES_NO_VSYNC
        };
        preferred
            .iter()
            .copied()
            .find(|mode| supported_modes.contains(mode))
            .expect("No supported presentation mode was found")
    }
}
