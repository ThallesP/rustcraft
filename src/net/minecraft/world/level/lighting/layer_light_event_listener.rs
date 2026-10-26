//! net/minecraft/world/level/lighting/LayerLightEventListener.java
//!
//! What the renderer and the game read of one layer's light. Java's also extends
//! `LightEventListener`, whose methods the engines implement on their own; reading the light needs
//! none of them, so a listener lent out for reading (`LevelLightEngine::get_layer_listener`) is
//! only this.
//!
//! `copy_section_light` is the port's: a section's light as `getLightValue` reads it, copied for a
//! section compile on another thread (`RenderSectionRegion`). Java's compile reads the engine's
//! visible layers there, which the engine swaps whole and never writes to.

use std::sync::Arc;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::chunk::data_layer::DataLayer;

/// One layer's light, as the engines last swapped it in.
pub trait LayerLightEventListener {
    /// The section's layer, queued or visible, or none. A copy, where Java hands out the layer.
    fn get_data_layer_data(&self, pos: &SectionPos) -> Option<DataLayer>;

    fn get_light_value(&self, pos: &BlockPos) -> i32;

    /// What `get_light_value` reads in the section, copied.
    fn copy_section_light(&self, section_node: i64) -> SectionLight;
}

/// A section's light as `getLightValue` reads it, copied: each block's value is `get`'s at the
/// block's place in the section.
#[derive(Clone, Debug)]
pub enum SectionLight {
    /// The same everywhere: none stored, or the sky's above a column's top.
    Uniform(i32),
    /// The section's layer.
    Layer(Arc<DataLayer>),
    /// The bottom of the next layer up, at every height: the sky's light in a section below a
    /// column's top that stores none.
    BottomOf(Arc<DataLayer>),
}

impl SectionLight {
    /// The light at a block of the section, by its coordinates in the section.
    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> i32 {
        match self {
            SectionLight::Uniform(value) => *value,
            SectionLight::Layer(layer) => layer.get(x, y, z),
            SectionLight::BottomOf(layer) => layer.get(x, 0, z),
        }
    }
}

/// `DummyLightLayerEventListener.INSTANCE`: no light anywhere, for a level without the layer.
pub struct DummyLightLayerEventListener;

impl LayerLightEventListener for DummyLightLayerEventListener {
    fn get_data_layer_data(&self, _pos: &SectionPos) -> Option<DataLayer> {
        None
    }

    fn get_light_value(&self, _pos: &BlockPos) -> i32 {
        0
    }

    fn copy_section_light(&self, _section_node: i64) -> SectionLight {
        SectionLight::Uniform(0)
    }
}
