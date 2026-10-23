//! net/minecraft/world/level/lighting/LayerLightEventListener.java
//!
//! What the renderer and the game read of one layer's light. Java's also extends
//! `LightEventListener`, whose methods the engines implement on their own; reading the light needs
//! none of them, so a listener lent out for reading (`LevelLightEngine::get_layer_listener`) is
//! only this.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::chunk::data_layer::DataLayer;

/// One layer's light, as the engines last swapped it in.
pub trait LayerLightEventListener {
    /// The section's layer, queued or visible, or none. A copy, where Java hands out the layer.
    fn get_data_layer_data(&self, pos: &SectionPos) -> Option<DataLayer>;

    fn get_light_value(&self, pos: &BlockPos) -> i32;
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
}
