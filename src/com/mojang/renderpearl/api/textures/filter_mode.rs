//! com/mojang/renderpearl/api/textures/FilterMode.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilterMode {
    NEAREST,
    LINEAR,
}

impl FilterMode {
    /// `values()`.
    pub const VALUES: [FilterMode; 2] = [FilterMode::NEAREST, FilterMode::LINEAR];
}
