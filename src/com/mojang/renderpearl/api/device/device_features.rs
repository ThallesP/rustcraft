//! com/mojang/renderpearl/api/device/DeviceFeatures.java

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceFeatures {
    pub wireframe_fill_mode: bool,
    pub shader_draw_parameters: bool,
    pub multi_draw_direct_interleaved: bool,
    pub multi_draw_direct_separate: bool,
    pub multi_draw_indirect: bool,
    pub draw_indirect: bool,
    pub non_zero_first_instance: bool,
    pub persistent_mapping: bool,
}
