//! com/mojang/renderpearl/api/pipeline/PolygonMode.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PolygonMode {
    FILL,
    WIREFRAME,
}

impl PolygonMode {
    /// `values()`.
    pub const VALUES: [PolygonMode; 2] = [PolygonMode::FILL, PolygonMode::WIREFRAME];
}
