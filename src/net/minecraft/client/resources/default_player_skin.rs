//! net/minecraft/client/resources/DefaultPlayerSkin.java

use crate::java::lang::math::Math;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::core::client_asset::ResourceTexture;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::entity::player::player_model_type::PlayerModelType;
use crate::net::minecraft::world::entity::player::player_skin::PlayerSkin;

/// The skins a player without one of its own gets, by its profile's id.
pub struct DefaultPlayerSkin;

const DEFAULT_SKINS: [(&str, PlayerModelType); 18] = [
    ("entity/player/slim/alex", PlayerModelType::SLIM),
    ("entity/player/slim/ari", PlayerModelType::SLIM),
    ("entity/player/slim/efe", PlayerModelType::SLIM),
    ("entity/player/slim/kai", PlayerModelType::SLIM),
    ("entity/player/slim/makena", PlayerModelType::SLIM),
    ("entity/player/slim/noor", PlayerModelType::SLIM),
    ("entity/player/slim/steve", PlayerModelType::SLIM),
    ("entity/player/slim/sunny", PlayerModelType::SLIM),
    ("entity/player/slim/zuri", PlayerModelType::SLIM),
    ("entity/player/wide/alex", PlayerModelType::WIDE),
    ("entity/player/wide/ari", PlayerModelType::WIDE),
    ("entity/player/wide/efe", PlayerModelType::WIDE),
    ("entity/player/wide/kai", PlayerModelType::WIDE),
    ("entity/player/wide/makena", PlayerModelType::WIDE),
    ("entity/player/wide/noor", PlayerModelType::WIDE),
    ("entity/player/wide/steve", PlayerModelType::WIDE),
    ("entity/player/wide/sunny", PlayerModelType::WIDE),
    ("entity/player/wide/zuri", PlayerModelType::WIDE),
];

impl DefaultPlayerSkin {
    pub fn get_default_texture() -> Identifier {
        DefaultPlayerSkin::get_default_skin().body.texture_path
    }

    pub fn get_default_skin() -> PlayerSkin {
        DefaultPlayerSkin::create(6)
    }

    /// The skin for a profile's id: `DEFAULT_SKINS[floorMod(id.hashCode(), 18)]`.
    pub fn get(profile_id: &UUID) -> PlayerSkin {
        DefaultPlayerSkin::create(Math::floor_mod_i32(
            profile_id.hash_code(),
            DEFAULT_SKINS.len() as i32,
        ) as usize)
    }

    fn create(index: usize) -> PlayerSkin {
        let (body, model) = DEFAULT_SKINS[index];
        PlayerSkin {
            body: ResourceTexture::new(Identifier::with_default_namespace(body)),
            cape: None,
            elytra: None,
            model,
            secure: true,
        }
    }
}
