//! com/mojang/renderpearl/api/pipeline/ColorTargetState.java

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::blend_function::BlendFunction;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ColorTargetState {
    pub blend_function: Option<BlendFunction>,
    pub format: GpuFormat,
    pub write_mask: i32,
}

impl ColorTargetState {
    pub const WRITE_RED: i32 = 1;
    pub const WRITE_GREEN: i32 = 2;
    pub const WRITE_BLUE: i32 = 4;
    pub const WRITE_ALPHA: i32 = 8;
    pub const WRITE_COLOR: i32 = 7;
    pub const WRITE_ALL: i32 = 15;
    pub const WRITE_NONE: i32 = 0;
    pub const DEFAULT: ColorTargetState = ColorTargetState::new(None, GpuFormat::RGBA8_UNORM, 15);
    pub const MAX_COLOR_TARGETS: usize = 8;

    pub const fn new(
        blend_function: Option<BlendFunction>,
        format: GpuFormat,
        write_mask: i32,
    ) -> ColorTargetState {
        ColorTargetState {
            blend_function,
            format,
            write_mask,
        }
    }

    /// `ColorTargetState(BlendFunction)`: blending into RGBA8, writing every channel.
    pub const fn blending(blend_function: BlendFunction) -> ColorTargetState {
        ColorTargetState::new(Some(blend_function), GpuFormat::RGBA8_UNORM, 15)
    }

    pub fn write_red(&self) -> bool {
        self.write_mask & 1 != 0
    }

    pub fn write_green(&self) -> bool {
        self.write_mask & 2 != 0
    }

    pub fn write_blue(&self) -> bool {
        self.write_mask & 4 != 0
    }

    pub fn write_alpha(&self) -> bool {
        self.write_mask & 8 != 0
    }
}
