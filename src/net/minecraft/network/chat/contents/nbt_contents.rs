//! net/minecraft/network/chat/contents/NbtContents.java
//!
//! Tags found by a path in a data source, which the server resolves (`resolve`, not ported)
//! before it sends a component; one sent unresolved shows nothing. The path is read like a
//! command's (`NBT_PATH_CODEC`), and fails to decode where it doesn't parse.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{BOOL, BoxedCodec, CodecExt, SharedCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, SharedMapCodec, StaticMapCodec};
use crate::net::minecraft::commands::arguments::nbt_path_argument::{NbtPath, NbtPathArgument};
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_contents::ComponentContents;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::chat::contents::data::data_source::DataSource;
use crate::net::minecraft::network::chat::contents::data::data_sources::DataSources;
use crate::net::minecraft::util::compilable_string::{CompilableString, parse_command};

static NBT_PATH_CODEC: LazyLock<BoxedCodec<CompilableString<NbtPath>>> = LazyLock::new(|| {
    CompilableString::codec(|contents| {
        parse_command(
            contents,
            |reader| NbtPathArgument.parse(reader),
            |original, exception| {
                format!("Invalid NBT path: {original}: {}", exception.get_message())
            },
        )
    })
});

// The record, checked for both flags on.
static INNER_CODEC: StaticMapCodec<NbtContents> = LazyLock::new(|| {
    let check = |contents: NbtContents| {
        if contents.interpreting && contents.plain {
            DataResult::error(|| "'interpret' and 'plain' flags can't be both on".to_owned())
        } else {
            DataResult::success(contents)
        }
    };
    record_codec_builder::map_codec(
        (
            (*NBT_PATH_CODEC)
                .clone()
                .field_of("nbt")
                .for_getter(|contents: &NbtContents| contents.nbt_path.clone()),
            BOOL.lenient_optional_field_of_default("interpret", false)
                .for_getter(|contents: &NbtContents| contents.interpreting),
            BOOL.lenient_optional_field_of_default("plain", false)
                .for_getter(|contents: &NbtContents| contents.plain),
            ComponentSerialization::CODEC
                .shared()
                .lenient_optional_field_of("separator")
                .for_getter(|contents: &NbtContents| contents.separator.clone()),
            DataSources::CODEC
                .shared()
                .for_getter(|contents: &NbtContents| contents.data_source.clone()),
        ),
        NbtContents::new,
    )
    .flat_xmap(check, move |contents: &NbtContents| check(contents.clone()))
    .boxed()
});

static MAP_CODEC: StaticMapCodec<ComponentContents> = LazyLock::new(|| {
    INNER_CODEC
        .shared()
        .xmap(
            ComponentContents::Nbt,
            |contents: &ComponentContents| match contents {
                ComponentContents::Nbt(nbt) => nbt.clone(),
                other => panic!("ClassCastException: {other}"),
            },
        )
        .boxed()
});

#[derive(Clone, Debug, PartialEq)]
pub struct NbtContents {
    nbt_path: CompilableString<NbtPath>,
    interpreting: bool,
    plain: bool,
    separator: Option<Component>,
    data_source: DataSource,
}

impl NbtContents {
    pub const MAP_CODEC: &'static StaticMapCodec<ComponentContents> = &MAP_CODEC;

    /// `NBT_PATH_CODEC`.
    pub fn nbt_path_codec() -> BoxedCodec<CompilableString<NbtPath>> {
        (*NBT_PATH_CODEC).clone()
    }

    pub fn new(
        nbt_path: CompilableString<NbtPath>,
        interpreting: bool,
        plain: bool,
        separator: Option<Component>,
        data_source: DataSource,
    ) -> NbtContents {
        NbtContents {
            nbt_path,
            interpreting,
            plain,
            separator,
            data_source,
        }
    }

    pub fn nbt_path(&self) -> &CompilableString<NbtPath> {
        &self.nbt_path
    }

    pub fn interpreting(&self) -> bool {
        self.interpreting
    }

    pub fn plain(&self) -> bool {
        self.plain
    }

    pub fn separator(&self) -> Option<&Component> {
        self.separator.as_ref()
    }

    pub fn data_source(&self) -> &DataSource {
        &self.data_source
    }
}

/// The record's `toString`.
impl fmt::Display for NbtContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NbtContents[nbtPath={}, interpreting={}, plain={}, separator=",
            self.nbt_path, self.interpreting, self.plain
        )?;
        match &self.separator {
            Some(separator) => write!(f, "Optional[{separator}]")?,
            None => f.write_str("Optional.empty")?,
        }
        write!(f, ", dataSource={}]", self.data_source)
    }
}
