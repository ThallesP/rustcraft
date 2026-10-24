//! net/minecraft/world/entity/player/PlayerSkin.java
//!
//! Not ported yet: `with` and `Patch.STREAM_CODEC`, which come with their callers.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::core::client_asset::ResourceTexture;
use crate::net::minecraft::world::entity::player::player_model_type::PlayerModelType;

static PATCH_MAP_CODEC: StaticMapCodec<Patch> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            ResourceTexture::CODEC
                .shared()
                .optional_field_of("texture")
                .for_getter(|patch: &Patch| patch.body.clone()),
            ResourceTexture::CODEC
                .shared()
                .optional_field_of("cape")
                .for_getter(|patch: &Patch| patch.cape.clone()),
            ResourceTexture::CODEC
                .shared()
                .optional_field_of("elytra")
                .for_getter(|patch: &Patch| patch.elytra.clone()),
            PlayerModelType::CODEC
                .shared()
                .optional_field_of("model")
                .for_getter(|patch: &Patch| patch.model),
        ),
        Patch::create,
    )
    .boxed()
});

/// A player's skin: its body's texture, its cape's and elytra's if it has them, its arms'
/// model, and whether it came signed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerSkin {
    pub body: ResourceTexture,
    pub cape: Option<ResourceTexture>,
    pub elytra: Option<ResourceTexture>,
    pub model: PlayerModelType,
    pub secure: bool,
}

impl PlayerSkin {
    pub fn insecure(
        body: ResourceTexture,
        cape: Option<ResourceTexture>,
        elytra: Option<ResourceTexture>,
        model: PlayerModelType,
    ) -> PlayerSkin {
        PlayerSkin {
            body,
            cape,
            elytra,
            model,
            secure: false,
        }
    }
}

/// `PlayerSkin.Patch`: the parts of a skin a profile sets in place of its own.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Patch {
    pub body: Option<ResourceTexture>,
    pub cape: Option<ResourceTexture>,
    pub elytra: Option<ResourceTexture>,
    pub model: Option<PlayerModelType>,
}

impl Patch {
    pub const EMPTY: Patch = Patch {
        body: None,
        cape: None,
        elytra: None,
        model: None,
    };
    pub const MAP_CODEC: &'static StaticMapCodec<Patch> = &PATCH_MAP_CODEC;

    pub fn create(
        texture: Option<ResourceTexture>,
        cape_texture: Option<ResourceTexture>,
        elytra_texture: Option<ResourceTexture>,
        model: Option<PlayerModelType>,
    ) -> Patch {
        Patch {
            body: texture,
            cape: cape_texture,
            elytra: elytra_texture,
            model,
        }
    }
}
