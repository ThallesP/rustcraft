//! com/mojang/renderpearl/api/device/DeviceLimits.java

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

impl DeviceLimits {
    /// The largest power-of-two side a square texture of the format can have: within the
    /// device's size, and within its largest allocation.
    pub fn max_texture_size_for_format(
        &self,
        format: crate::com::mojang::renderpearl::api::gpu_format::GpuFormat,
    ) -> i32 {
        let by_memory =
            (self.max_memory_allocation_size as f64 / f64::from(format.block_size())).sqrt() as i32;
        highest_one_bit(self.max_texture_size.min(by_memory))
    }
}

/// `Integer.highestOneBit`.
fn highest_one_bit(i: i32) -> i32 {
    i & (i32::MIN as u32).wrapping_shr(i.leading_zeros()) as i32
}
