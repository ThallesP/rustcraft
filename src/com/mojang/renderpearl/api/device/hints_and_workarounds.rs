//! com/mojang/renderpearl/api/device/HintsAndWorkarounds.java

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HintsAndWorkarounds {
    pub write_to_buffer_is_slow: bool,
    pub anisotropy_has_known_issues: bool,
    pub is_explicit_depth_required: bool,
    pub multi_draw_indirect_has_known_issues: bool,
}
