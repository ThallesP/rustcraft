//! net/minecraft/network/chat/contents/objects/ObjectInfo.java
//!
//! What an object component draws. Java's interface is an enum of its two classes here, and
//! `codec()` is the variant's kind (`ObjectInfos`).

use std::fmt;

use crate::net::minecraft::network::chat::contents::objects::atlas_sprite::AtlasSprite;
use crate::net::minecraft::network::chat::contents::objects::object_infos::ObjectType;
use crate::net::minecraft::network::chat::contents::objects::player_sprite::PlayerSprite;
use crate::net::minecraft::network::chat::font_description::FontDescription;

// A player's profile is larger than an atlas's two ids, and objects are rare.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum ObjectInfo {
    Atlas(AtlasSprite),
    Player(PlayerSprite),
}

impl ObjectInfo {
    pub fn font_description(&self) -> FontDescription {
        match self {
            ObjectInfo::Atlas(sprite) => sprite.font_description(),
            ObjectInfo::Player(sprite) => sprite.font_description(),
        }
    }

    pub fn default_fallback(&self) -> String {
        match self {
            ObjectInfo::Atlas(sprite) => sprite.default_fallback(),
            ObjectInfo::Player(sprite) => sprite.default_fallback(),
        }
    }

    /// `codec()`.
    pub fn codec(&self) -> ObjectType {
        match self {
            ObjectInfo::Atlas(_) => ObjectType::Atlas,
            ObjectInfo::Player(_) => ObjectType::Player,
        }
    }
}

impl fmt::Display for ObjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectInfo::Atlas(sprite) => sprite.fmt(f),
            ObjectInfo::Player(sprite) => sprite.fmt(f),
        }
    }
}
