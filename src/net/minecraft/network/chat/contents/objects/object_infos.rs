//! net/minecraft/network/chat/contents/objects/ObjectInfos.java
//!
//! The kinds of objects, by the `object` field's name, matched like a component's contents
//! (`createLegacyComponentMatcher`).

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::map_codec::{
    BoxedMapCodec, MapCodecExt, SharedMapCodec, StaticMapCodec,
};
use crate::net::minecraft::network::chat::component_serialization::{
    ComponentSerialization, LegacyType,
};
use crate::net::minecraft::network::chat::contents::objects::atlas_sprite::AtlasSprite;
use crate::net::minecraft::network::chat::contents::objects::object_info::ObjectInfo;
use crate::net::minecraft::network::chat::contents::objects::player_sprite::PlayerSprite;
use crate::net::minecraft::util::extra_codecs::LateBoundIdMapper;

/// Each class's `MAP_CODEC`, the values of the `ID_MAPPER`, in the order its static block puts
/// them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectType {
    Atlas,
    Player,
}

impl LegacyType for ObjectType {
    type Value = ObjectInfo;

    fn codec(self) -> BoxedMapCodec<ObjectInfo> {
        match self {
            ObjectType::Atlas => AtlasSprite::MAP_CODEC.shared(),
            ObjectType::Player => PlayerSprite::MAP_CODEC.shared(),
        }
    }

    fn of(value: &ObjectInfo) -> ObjectType {
        value.codec()
    }
}

/// `MapCodec.toString`, which no message prints.
impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

static CODEC: StaticMapCodec<ObjectInfo> = LazyLock::new(|| {
    let mut id_mapper = LateBoundIdMapper::new();
    id_mapper.put("atlas".to_owned(), ObjectType::Atlas);
    id_mapper.put("player".to_owned(), ObjectType::Player);
    ComponentSerialization::create_legacy_component_matcher(&id_mapper, "object").boxed()
});

pub struct ObjectInfos;

impl ObjectInfos {
    pub const CODEC: &'static StaticMapCodec<ObjectInfo> = &CODEC;
}
