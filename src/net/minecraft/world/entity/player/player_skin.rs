//! net/minecraft/world/entity/player/PlayerSkin.java
//!
//! Not ported yet: `Patch` and `with`, which come with the profiles' skin patches.

use crate::net::minecraft::core::client_asset::ResourceTexture;
use crate::net::minecraft::world::entity::player::player_model_type::PlayerModelType;

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
