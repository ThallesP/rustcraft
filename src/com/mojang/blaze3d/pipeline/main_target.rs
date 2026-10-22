//! com/mojang/blaze3d/pipeline/MainTarget.java
//!
//! The game's main render target: RGBA8 color and 32-bit float depth at the window's size. Java
//! falls back to smaller sizes when the driver can't allocate it (`Dimension.listWithFallback`);
//! the port's allocation panics instead, until a driver needs the fallback.

use crate::com::mojang::blaze3d::pipeline::render_target::RenderTarget;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use std::ops::{Deref, DerefMut};

pub struct MainTarget {
    target: RenderTarget,
}

impl MainTarget {
    pub fn new(device: &dyn GpuDevice, desired_width: i32, desired_height: i32) -> MainTarget {
        let mut target = RenderTarget::new(
            Some("Main"),
            Some(GpuFormat::RGBA8_UNORM),
            Some(GpuFormat::D32_FLOAT),
        );
        target.create_buffers(device, desired_width, desired_height);
        MainTarget { target }
    }
}

impl Deref for MainTarget {
    type Target = RenderTarget;

    fn deref(&self) -> &RenderTarget {
        &self.target
    }
}

impl DerefMut for MainTarget {
    fn deref_mut(&mut self) -> &mut RenderTarget {
        &mut self.target
    }
}
