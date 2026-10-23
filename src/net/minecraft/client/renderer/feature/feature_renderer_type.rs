//! net/minecraft/client/renderer/feature/FeatureRendererType.java
//!
//! Each feature renderer's type, numbered in the order Java's classes make them (`create`, as each
//! class loads), which only orders a `SimpleFeatureRenderPhase`'s features among themselves. Only
//! the entity models' and the shape outlines' types are ported, numbered in that order.

/// A feature renderer's type: its number and name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeatureRendererType {
    id: i32,
    name: &'static str,
}

impl FeatureRendererType {
    /// `ModelFeatureRenderer.TYPE`.
    pub const MODEL: FeatureRendererType = FeatureRendererType {
        id: 0,
        name: "Entity Model",
    };

    /// `ShapeOutlineFeatureRenderer.TYPE`.
    pub const SHAPE_OUTLINE: FeatureRendererType = FeatureRendererType {
        id: 1,
        name: "Shape Outline",
    };

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}
