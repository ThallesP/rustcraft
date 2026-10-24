//! net/minecraft/network/chat/contents/objects/PlayerSprite.java

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{BOOL, CodecExt, SharedCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::network::chat::contents::objects::object_info::ObjectInfo;
use crate::net::minecraft::network::chat::font_description::FontDescription;
use crate::net::minecraft::world::item::component::resolvable_profile::ResolvableProfile;

static MAP_CODEC: StaticMapCodec<ObjectInfo> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            ResolvableProfile::CODEC
                .shared()
                .field_of("player")
                .for_getter(|info: &ObjectInfo| match info {
                    ObjectInfo::Player(sprite) => sprite.player.clone(),
                    other => panic!("ClassCastException: {other}"),
                }),
            BOOL.optional_field_of_default("hat", true).for_getter(
                |info: &ObjectInfo| match info {
                    ObjectInfo::Player(sprite) => sprite.hat,
                    other => panic!("ClassCastException: {other}"),
                },
            ),
        ),
        |player, hat| ObjectInfo::Player(PlayerSprite::new(player, hat)),
    )
    .boxed()
});

/// A player's head, drawn as a glyph, with its hat or without.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlayerSprite {
    player: ResolvableProfile,
    hat: bool,
}

impl PlayerSprite {
    pub const MAP_CODEC: &'static StaticMapCodec<ObjectInfo> = &MAP_CODEC;

    pub fn new(player: ResolvableProfile, hat: bool) -> PlayerSprite {
        PlayerSprite { player, hat }
    }

    pub fn player(&self) -> &ResolvableProfile {
        &self.player
    }

    pub fn hat(&self) -> bool {
        self.hat
    }

    pub fn font_description(&self) -> FontDescription {
        FontDescription::PlayerSprite {
            profile: Box::new(self.player.clone()),
            hat: self.hat,
        }
    }

    /// `[Steve head]`, or `[unknown player head]` without a name.
    pub fn default_fallback(&self) -> String {
        match self.player.name() {
            Some(name) => format!("[{name} head]"),
            None => "[unknown player head]".to_owned(),
        }
    }
}

/// The record's `toString`, the profile by its class (Java prints it by identity).
impl fmt::Display for PlayerSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlayerSprite[player={}, hat={}]", self.player, self.hat)
    }
}
