//! com/mojang/renderpearl/api/device/DeviceLimits.java
//!
//! `maxTextureSizeForFormat` comes with its first caller.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceLimits {
    pub max_anisotropy: i32,
    pub min_uniform_offset_alignment: i32,
    pub max_texture_size: i32,
    pub max_memory_allocation_size: i64,
    pub max_multi_draw_direct_interleaved_draw_count: i32,
    pub max_color_attachments: i32,
    pub max_draw_indirect_draw_count: i32,
}
