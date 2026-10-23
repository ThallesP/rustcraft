//! net/minecraft/network/chat/HoverEvent.java
//!
//! What hovering over a text shows. Java's interface of records is an enum of them, and `CODEC`
//! dispatches on the action under `"action"`. Not ported yet: `ShowItem`, which holds an
//! `ItemStackTemplate` (items and their data components); its action's codec fails to decode
//! (`map_codec::unported`). `EntityTooltipInfo.getTooltipLines` comes with tooltips, and its
//! `toString` is `Object`'s, an identity hash, which `ShowEntity`'s prints a stand-in for.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::SharedCodec;
use crate::com::mojang::serialization::codec::{CodecExt, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::com::mojang::serialization::map_codec::SharedMapCodec;
use crate::com::mojang::serialization::map_codec::{
    self, BoxedMapCodec, MapCodecExt, StaticMapCodec,
};
use crate::java::util::uuid::UUID;
use crate::net::minecraft::core::uuid_util::UUIDUtil;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};
use crate::net::minecraft::world::entity::entity_type::EntityType;
use crate::net::minecraft::world::entity::entity_types;

#[derive(Clone, Debug, PartialEq)]
pub enum HoverEvent {
    ShowText { value: Component },
    ShowEntity { entity: EntityTooltipInfo },
}

/// `HoverEvent.EntityTooltipInfo`: an entity's type, UUID and name.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityTooltipInfo {
    pub r#type: EntityType,
    pub uuid: UUID,
    pub name: Option<Component>,
}

// `BuiltInRegistries.ENTITY_TYPE.byNameCodec()`: a registered name, stable either way
// (`overrideLifecycle` with a built-in entry's lifecycle), then `flatComapMap` to the value.
static ENTITY_TYPE_BY_NAME: StaticCodec<EntityType> = LazyLock::new(|| {
    ExtraCodecs::override_lifecycle_stable(Identifier::CODEC.shared().comap_flat_map(
        |name: Identifier| match entity_types::REGISTRY.get_optional(&name) {
            Some(entity_type) => DataResult::success(entity_type),
            None => DataResult::error(|| {
                format!("Unknown registry key in ResourceKey[minecraft:root / minecraft:entity_type]: {name}")
            }),
        },
        |entity_type: &EntityType| entity_types::REGISTRY.get_key(*entity_type).clone(),
    ))
    .flat_comap_map(|entity_type| entity_type, |entity_type: &EntityType| {
        DataResult::success(*entity_type)
    })
    .boxed()
});

// `EntityTooltipInfo.CODEC`.
static ENTITY_TOOLTIP_INFO_CODEC: StaticMapCodec<EntityTooltipInfo> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            ENTITY_TYPE_BY_NAME
                .shared()
                .field_of("id")
                .for_getter(|o: &EntityTooltipInfo| o.r#type),
            UUIDUtil::LENIENT_CODEC
                .shared()
                .field_of("uuid")
                .for_getter(|o: &EntityTooltipInfo| o.uuid),
            ComponentSerialization::CODEC
                .shared()
                .optional_field_of("name")
                .for_getter(|o: &EntityTooltipInfo| o.name.clone()),
        ),
        |r#type, uuid, name| EntityTooltipInfo { r#type, uuid, name },
    )
    .boxed()
});

static SHOW_ENTITY_CODEC: StaticMapCodec<HoverEvent> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (ENTITY_TOOLTIP_INFO_CODEC
            .shared()
            .for_getter(|event: &HoverEvent| match event {
                HoverEvent::ShowEntity { entity } => entity.clone(),
                other => panic!("ClassCastException: {other}"),
            }),),
        |entity| HoverEvent::ShowEntity { entity },
    )
    .boxed()
});

/// `HoverEvent.Action`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    SHOW_TEXT,
    SHOW_ITEM,
    SHOW_ENTITY,
}

const NAMES: [&str; 3] = ["show_text", "show_item", "show_entity"];

// `UNSAFE_CODEC.validate(Action::filterForSerialization)`: every action may come from a server.
static ACTION_CODEC: StaticCodec<Action> = LazyLock::new(|| {
    string_representable::from_values(&Action::VALUES)
        .validate(|action| DataResult::success_lifecycle(action, Lifecycle::stable()))
        .boxed()
});

impl Action {
    pub const VALUES: [Action; 3] = [Action::SHOW_TEXT, Action::SHOW_ITEM, Action::SHOW_ENTITY];
    pub const CODEC: &'static StaticCodec<Action> = &ACTION_CODEC;

    pub fn is_allowed_from_server(self) -> bool {
        true
    }

    /// The map codec of the action's record.
    pub fn codec(self) -> BoxedMapCodec<HoverEvent> {
        match self {
            Action::SHOW_TEXT => SHOW_TEXT_CODEC.shared(),
            Action::SHOW_ITEM => map_codec::unported("HoverEvent.ShowItem").boxed(),
            Action::SHOW_ENTITY => SHOW_ENTITY_CODEC.clone(),
        }
    }
}

impl StringRepresentable for Action {
    fn get_serialized_name(&self) -> &'static str {
        NAMES[*self as usize]
    }
}

/// `<action show_text>`.
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<action {}>", self.get_serialized_name())
    }
}

static SHOW_TEXT_CODEC: StaticMapCodec<HoverEvent> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (ComponentSerialization::CODEC
            .shared()
            .field_of("value")
            .for_getter(|event: &HoverEvent| match event {
                HoverEvent::ShowText { value } => value.clone(),
                other => panic!("ClassCastException: {other}"),
            }),),
        |value| HoverEvent::ShowText { value },
    )
    .boxed()
});

// `Action.CODEC.dispatch("action", HoverEvent::action, action -> action.codec)`.
static CODEC: StaticCodec<HoverEvent> = LazyLock::new(|| {
    Action::CODEC
        .shared()
        .dispatch("action", HoverEvent::action, |action: &Action| {
            action.codec()
        })
        .boxed()
});

impl HoverEvent {
    pub const CODEC: &'static StaticCodec<HoverEvent> = &CODEC;

    pub fn action(&self) -> Action {
        match self {
            HoverEvent::ShowText { .. } => Action::SHOW_TEXT,
            HoverEvent::ShowEntity { .. } => Action::SHOW_ENTITY,
        }
    }
}

/// The records' `toString`.
impl fmt::Display for HoverEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HoverEvent::ShowText { value } => write!(f, "ShowText[value={value}]"),
            HoverEvent::ShowEntity { .. } => f.write_str(
                "ShowEntity[entity=net.minecraft.network.chat.HoverEvent$EntityTooltipInfo]",
            ),
        }
    }
}
