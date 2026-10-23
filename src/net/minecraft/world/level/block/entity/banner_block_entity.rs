//! net/minecraft/world/level/block/entity/BannerBlockEntity.java
//!
//! A banner's base colour (its block's), custom name and patterns, as `loadAdditional` reads
//! them. Not ported yet: the components (`applyImplicitComponents`, which come with items),
//! `getName`'s default and the item the block drops (`getItem`).

use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::world::item::dye_color::DyeColor;
use crate::net::minecraft::world::level::block::entity::banner_pattern_layers::BannerPatternLayers;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

#[derive(Clone, Debug, PartialEq)]
pub struct BannerBlockEntity {
    name: Option<Component>,
    base_color: DyeColor,
    patterns: BannerPatternLayers,
}

impl BannerBlockEntity {
    /// `BannerBlockEntity(worldPosition, blockState, color)`.
    pub fn new(base_color: DyeColor) -> BannerBlockEntity {
        BannerBlockEntity {
            name: None,
            base_color,
            patterns: BannerPatternLayers::empty(),
        }
    }

    pub fn load_additional(&mut self, input: &TagValueInput<'_>) {
        self.name = input.read("CustomName", ComponentSerialization::CODEC);
        self.patterns = input
            .read("patterns", BannerPatternLayers::CODEC)
            .unwrap_or_else(BannerPatternLayers::empty);
    }

    pub fn get_custom_name(&self) -> Option<&Component> {
        self.name.as_ref()
    }

    pub fn get_patterns(&self) -> &BannerPatternLayers {
        &self.patterns
    }

    pub fn get_base_color(&self) -> DyeColor {
        self.base_color
    }
}
