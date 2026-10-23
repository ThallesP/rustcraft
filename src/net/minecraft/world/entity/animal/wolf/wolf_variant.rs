//! net/minecraft/world/entity/animal/wolf/WolfVariant.java
//!
//! A wolf's textures, an element of the synchronized `wolf_variant` registry, as `NETWORK_CODEC`
//! reads it (`RegistryDataLoader`), which leaves out the spawn conditions (`DIRECT_CODEC`'s, the
//! server's). `CODEC` and `STREAM_CODEC`, holders, come with wolves.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::MapCodecExt;
use crate::net::minecraft::core::client_asset::ResourceTexture;

static NETWORK_CODEC: StaticCodec<WolfVariant> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            AssetInfo::CODEC
                .shared()
                .field_of("assets")
                .for_getter(|o: &WolfVariant| o.adult_info.clone()),
            AssetInfo::CODEC
                .shared()
                .field_of("baby_assets")
                .for_getter(|o: &WolfVariant| o.baby_info.clone()),
        ),
        WolfVariant::new,
    )
    .boxed()
});

#[derive(Clone, Debug, PartialEq)]
pub struct WolfVariant {
    adult_info: AssetInfo,
    baby_info: AssetInfo,
}

impl WolfVariant {
    pub const NETWORK_CODEC: &'static StaticCodec<WolfVariant> = &NETWORK_CODEC;

    /// The private constructor `NETWORK_CODEC` makes it with, without spawn conditions.
    fn new(adult_info: AssetInfo, baby_info: AssetInfo) -> WolfVariant {
        WolfVariant {
            adult_info,
            baby_info,
        }
    }

    pub fn adult_info(&self) -> &AssetInfo {
        &self.adult_info
    }

    pub fn baby_info(&self) -> &AssetInfo {
        &self.baby_info
    }
}

static ASSET_INFO_CODEC: StaticCodec<AssetInfo> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            ResourceTexture::CODEC
                .shared()
                .field_of("wild")
                .for_getter(|o: &AssetInfo| o.wild.clone()),
            ResourceTexture::CODEC
                .shared()
                .field_of("tame")
                .for_getter(|o: &AssetInfo| o.tame.clone()),
            ResourceTexture::CODEC
                .shared()
                .field_of("angry")
                .for_getter(|o: &AssetInfo| o.angry.clone()),
        ),
        AssetInfo::new,
    )
    .boxed()
});

/// `WolfVariant.AssetInfo`: the wild, tame and angry textures.
#[derive(Clone, Debug, PartialEq)]
pub struct AssetInfo {
    wild: ResourceTexture,
    tame: ResourceTexture,
    angry: ResourceTexture,
}

impl AssetInfo {
    pub const CODEC: &'static StaticCodec<AssetInfo> = &ASSET_INFO_CODEC;

    pub fn new(wild: ResourceTexture, tame: ResourceTexture, angry: ResourceTexture) -> AssetInfo {
        AssetInfo { wild, tame, angry }
    }

    pub fn wild(&self) -> &ResourceTexture {
        &self.wild
    }

    pub fn tame(&self) -> &ResourceTexture {
        &self.tame
    }

    pub fn angry(&self) -> &ResourceTexture {
        &self.angry
    }
}
