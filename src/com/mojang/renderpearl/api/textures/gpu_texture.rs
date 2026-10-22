//! com/mojang/renderpearl/api/textures/GpuTexture.java
//!
//! An interface's constants are its module's, since a trait with constants can't be a `dyn`.

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use std::any::Any;

pub const USAGE_COPY_DST: i32 = 1;
pub const USAGE_COPY_SRC: i32 = 2;
pub const USAGE_TEXTURE_BINDING: i32 = 4;
pub const USAGE_RENDER_ATTACHMENT: i32 = 8;
pub const USAGE_CUBEMAP_COMPATIBLE: i32 = 16;

/// A texture a backend made. Each backend casts it back to its own (`(GlTexture) texture`)
/// through `as_any`.
pub trait GpuTexture {
    fn get_usage(&self) -> i32;

    fn get_label(&self) -> &str;

    fn get_format(&self) -> GpuFormat;

    fn get_width(&self, mip_level: i32) -> i32;

    fn get_height(&self, mip_level: i32) -> i32;

    fn get_depth_or_layers(&self) -> i32;

    fn get_mip_levels(&self) -> i32;

    fn as_any(&self) -> &dyn Any;
}
