//! net/minecraft/world/level/block/entity/BannerPatternLayers.java
//!
//! A banner's patterns, bottom first, each a pattern of the synchronized `banner_pattern`
//! registry and a dye colour. A pattern is read by its id (`BannerPattern.CODEC`, a
//! `RegistryFileCodec`); one written out inline (`asset_id` and `translation_key`, which Java
//! makes a direct holder of) isn't ported and fails to decode saying so. Not ported yet:
//! `STREAM_CODEC` (items' banner patterns), the builder and the tooltips.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::MapCodecExt;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::codec::registry_codecs::RegistryCodecs;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::world::item::dye_color::DyeColor;

static LAYER_CODEC: StaticCodec<Layer> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            RegistryCodecs::holder_by_file(Registries::BANNER_PATTERN, "BannerPattern")
                .field_of("pattern")
                .for_getter(|o: &Layer| o.pattern.clone()),
            DyeColor::CODEC
                .shared()
                .field_of("color")
                .for_getter(|o: &Layer| o.color),
        ),
        Layer::new,
    )
    .boxed()
});

static CODEC: StaticCodec<BannerPatternLayers> = LazyLock::new(|| {
    LAYER_CODEC
        .shared()
        .list_of()
        .xmap(BannerPatternLayers::new, |layers: &BannerPatternLayers| {
            layers.layers.clone()
        })
        .boxed()
});

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BannerPatternLayers {
    layers: Vec<Layer>,
}

impl BannerPatternLayers {
    pub const CODEC: &'static StaticCodec<BannerPatternLayers> = &CODEC;

    pub fn new(layers: Vec<Layer>) -> BannerPatternLayers {
        BannerPatternLayers { layers }
    }

    /// `EMPTY`.
    pub fn empty() -> BannerPatternLayers {
        BannerPatternLayers::default()
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }
}

/// `BannerPatternLayers.Layer`.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pattern: Holder,
    color: DyeColor,
}

impl Layer {
    pub const CODEC: &'static StaticCodec<Layer> = &LAYER_CODEC;

    pub fn new(pattern: Holder, color: DyeColor) -> Layer {
        Layer { pattern, color }
    }

    pub fn pattern(&self) -> &Holder {
        &self.pattern
    }

    pub fn color(&self) -> DyeColor {
        self.color
    }
}
