//! net/minecraft/network/chat/ClickEvent.java
//!
//! What clicking a text does. Java's interface of records is an enum of them, each record's map
//! codec reading its variant, and `CODEC` dispatches on the action under `"action"`. Not ported
//! yet: `ShowDialog`, which holds a dialog's holder; its action's codec fails to decode
//! (`map_codec::unported`). `OpenFile`'s `File` and
//! `Path` constructors aren't ported: the client never makes one from a server's text, and its
//! action isn't allowed from one (`filterForSerialization`).

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::SharedCodec;
use crate::com::mojang::serialization::codec::{CodecExt, STRING, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::com::mojang::serialization::map_codec::SharedMapCodec;
use crate::com::mojang::serialization::map_codec::{
    self, BoxedMapCodec, MapCodecExt, StaticMapCodec,
};
use crate::java::net::uri::URI;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};

#[derive(Clone, Debug, PartialEq)]
pub enum ClickEvent {
    OpenUrl {
        uri: URI,
    },
    OpenFile {
        path: String,
    },
    RunCommand {
        command: String,
    },
    SuggestCommand {
        command: String,
    },
    ChangePage {
        page: i32,
    },
    CopyToClipboard {
        value: String,
    },
    Custom {
        id: Identifier,
        payload: Option<Tag>,
    },
}

/// `ClickEvent.Action`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    OPEN_URL,
    OPEN_FILE,
    RUN_COMMAND,
    SUGGEST_COMMAND,
    SHOW_DIALOG,
    CHANGE_PAGE,
    COPY_TO_CLIPBOARD,
    CUSTOM,
}

// Each action's name and whether a server may send it.
const ACTIONS: [(&str, bool); 8] = [
    ("open_url", true),
    ("open_file", false),
    ("run_command", true),
    ("suggest_command", true),
    ("show_dialog", true),
    ("change_page", true),
    ("copy_to_clipboard", true),
    ("custom", true),
];

// `UNSAFE_CODEC.validate(Action::filterForSerialization)`.
static ACTION_CODEC: StaticCodec<Action> = LazyLock::new(|| {
    string_representable::from_values(&Action::VALUES)
        .validate(Action::filter_for_serialization)
        .boxed()
});

impl Action {
    pub const VALUES: [Action; 8] = [
        Action::OPEN_URL,
        Action::OPEN_FILE,
        Action::RUN_COMMAND,
        Action::SUGGEST_COMMAND,
        Action::SHOW_DIALOG,
        Action::CHANGE_PAGE,
        Action::COPY_TO_CLIPBOARD,
        Action::CUSTOM,
    ];
    pub const CODEC: &'static StaticCodec<Action> = &ACTION_CODEC;

    pub fn is_allowed_from_server(self) -> bool {
        ACTIONS[self as usize].1
    }

    /// `valueCodec()`: the map codec of the action's record.
    pub fn value_codec(self) -> BoxedMapCodec<ClickEvent> {
        match self {
            Action::OPEN_URL => OPEN_URL_CODEC.clone(),
            Action::OPEN_FILE => OPEN_FILE_CODEC.shared(),
            Action::RUN_COMMAND => RUN_COMMAND_CODEC.shared(),
            Action::SUGGEST_COMMAND => SUGGEST_COMMAND_CODEC.shared(),
            Action::SHOW_DIALOG => map_codec::unported("ClickEvent.ShowDialog").boxed(),
            Action::CHANGE_PAGE => CHANGE_PAGE_CODEC.shared(),
            Action::COPY_TO_CLIPBOARD => COPY_TO_CLIPBOARD_CODEC.shared(),
            Action::CUSTOM => CUSTOM_CODEC.shared(),
        }
    }

    /// The action, stable, if a server may send it.
    pub fn filter_for_serialization(action: Action) -> DataResult<Action> {
        if !action.is_allowed_from_server() {
            DataResult::error(|| format!("Click event type not allowed: {action}"))
        } else {
            DataResult::success_lifecycle(action, Lifecycle::stable())
        }
    }
}

impl StringRepresentable for Action {
    fn get_serialized_name(&self) -> &'static str {
        ACTIONS[*self as usize].0
    }
}

/// The constant's name, `Enum.toString`.
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// A record's getter, which Java reaches through a cast to the record's class (its
// `ClassCastException` a panic here).
macro_rules! field {
    ($variant:ident, $field:ident) => {
        |event: &ClickEvent| match event {
            ClickEvent::$variant { $field, .. } => $field.clone(),
            other => panic!("ClassCastException: {other}"),
        }
    };
}

static OPEN_URL_CODEC: StaticMapCodec<ClickEvent> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (ExtraCodecs::UNTRUSTED_URI
            .shared()
            .field_of("url")
            .for_getter(field!(OpenUrl, uri)),),
        |uri| ClickEvent::OpenUrl { uri },
    )
    .boxed()
});

static OPEN_FILE_CODEC: StaticMapCodec<ClickEvent> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (STRING.field_of("path").for_getter(field!(OpenFile, path)),),
        |path| ClickEvent::OpenFile { path },
    )
    .boxed()
});

static RUN_COMMAND_CODEC: StaticMapCodec<ClickEvent> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (ExtraCodecs::CHAT_STRING
            .shared()
            .field_of("command")
            .for_getter(field!(RunCommand, command)),),
        |command| ClickEvent::RunCommand { command },
    )
    .boxed()
});

static SUGGEST_COMMAND_CODEC: StaticMapCodec<ClickEvent> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (ExtraCodecs::CHAT_STRING
            .shared()
            .field_of("command")
            .for_getter(field!(SuggestCommand, command)),),
        |command| ClickEvent::SuggestCommand { command },
    )
    .boxed()
});

static CHANGE_PAGE_CODEC: StaticMapCodec<ClickEvent> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (ExtraCodecs::POSITIVE_INT
            .shared()
            .field_of("page")
            .for_getter(field!(ChangePage, page)),),
        |page| ClickEvent::ChangePage { page },
    )
    .boxed()
});

static COPY_TO_CLIPBOARD_CODEC: StaticMapCodec<ClickEvent> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (STRING
            .field_of("value")
            .for_getter(field!(CopyToClipboard, value)),),
        |value| ClickEvent::CopyToClipboard { value },
    )
    .boxed()
});

static CUSTOM_CODEC: StaticMapCodec<ClickEvent> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            Identifier::CODEC
                .shared()
                .field_of("id")
                .for_getter(field!(Custom, id)),
            ExtraCodecs::NBT
                .shared()
                .optional_field_of("payload")
                .for_getter(field!(Custom, payload)),
        ),
        |id, payload| ClickEvent::Custom { id, payload },
    )
    .boxed()
});

// `Action.CODEC.dispatch("action", ClickEvent::action, action -> action.codec)`.
static CODEC: StaticCodec<ClickEvent> = LazyLock::new(|| {
    Action::CODEC
        .shared()
        .dispatch("action", ClickEvent::action, |action: &Action| {
            action.value_codec()
        })
        .boxed()
});

impl ClickEvent {
    pub const CODEC: &'static StaticCodec<ClickEvent> = &CODEC;

    pub fn action(&self) -> Action {
        match self {
            ClickEvent::OpenUrl { .. } => Action::OPEN_URL,
            ClickEvent::OpenFile { .. } => Action::OPEN_FILE,
            ClickEvent::RunCommand { .. } => Action::RUN_COMMAND,
            ClickEvent::SuggestCommand { .. } => Action::SUGGEST_COMMAND,
            ClickEvent::ChangePage { .. } => Action::CHANGE_PAGE,
            ClickEvent::CopyToClipboard { .. } => Action::COPY_TO_CLIPBOARD,
            ClickEvent::Custom { .. } => Action::CUSTOM,
        }
    }
}

/// The records' `toString`; an `Optional` prints `Optional[…]` or `Optional.empty`.
impl fmt::Display for ClickEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClickEvent::OpenUrl { uri } => write!(f, "OpenUrl[uri={uri}]"),
            ClickEvent::OpenFile { path } => write!(f, "OpenFile[path={path}]"),
            ClickEvent::RunCommand { command } => write!(f, "RunCommand[command={command}]"),
            ClickEvent::SuggestCommand { command } => {
                write!(f, "SuggestCommand[command={command}]")
            }
            ClickEvent::ChangePage { page } => write!(f, "ChangePage[page={page}]"),
            ClickEvent::CopyToClipboard { value } => write!(f, "CopyToClipboard[value={value}]"),
            ClickEvent::Custom { id, payload } => match payload {
                Some(payload) => write!(f, "Custom[id={id}, payload=Optional[{payload}]]"),
                None => write!(f, "Custom[id={id}, payload=Optional.empty]"),
            },
        }
    }
}
