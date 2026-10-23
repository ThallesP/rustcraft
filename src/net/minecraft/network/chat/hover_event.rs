//! net/minecraft/network/chat/HoverEvent.java
//!
//! What hovering over a text shows. Java's interface of records is an enum of them, and `CODEC`
//! dispatches on the action under `"action"`. Not ported yet: `ShowItem`, which holds an
//! `ItemStackTemplate` (items and their data components), and `ShowEntity`'s
//! `EntityTooltipInfo`, which reads an entity type from `BuiltInRegistries.ENTITY_TYPE`; their
//! actions' codecs fail to decode (`map_codec::unported`).

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
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};

#[derive(Clone, Debug, PartialEq)]
pub enum HoverEvent {
    ShowText { value: Component },
}

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
            Action::SHOW_ENTITY => map_codec::unported("HoverEvent.ShowEntity").boxed(),
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
        }
    }
}

/// The records' `toString`.
impl fmt::Display for HoverEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HoverEvent::ShowText { value } => write!(f, "ShowText[value={value}]"),
        }
    }
}
