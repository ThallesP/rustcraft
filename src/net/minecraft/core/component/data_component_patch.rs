//! net/minecraft/core/component/DataComponentPatch.java
//!
//! The components an item stack sets or removes over its item's defaults, by type (`!` before a
//! type's name removes it). The component types' value codecs aren't ported, so a value is kept as
//! the data it was read from, as NBT (`ExtraCodecs.NBT`), which reads whatever a value is: a value
//! Java's codec would reject reads here too, so the port fails less than Java, never more. Not
//! ported yet: `STREAM_CODEC` and `DELIMITED_STREAM_CODEC` (which need each type's stream codec),
//! the patch's queries and builder, and `split`.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{
    BoxedCodec, CodecExt, STRING, SharedCodec, StaticCodec,
};
use crate::com::mojang::serialization::codecs::dispatched_map_codec::DispatchedMapCodec;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::net::minecraft::core::component::data_component_type::DataComponentType;
use crate::net::minecraft::core::component::removed::Removed;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;

/// `PatchKey`: a type, set or removed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PatchKey {
    r#type: DataComponentType,
    removed: bool,
}

impl PatchKey {
    // `valueCodec`: the mark for a removed type, else the type's value as data.
    fn value_codec(&self) -> BoxedCodec<Option<Tag>> {
        if self.removed {
            Removed::CODEC
                .shared()
                .xmap(|_| None, |_: &Option<Tag>| Removed::INSTANCE)
                .boxed()
        } else {
            ExtraCodecs::NBT
                .shared()
                .xmap(Some, |value: &Option<Tag>| {
                    value.clone().expect("a set component has its value")
                })
                .boxed()
        }
    }
}

/// `toString`: the record's.
impl fmt::Display for PatchKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PatchKey[type={}, removed={}]",
            self.r#type, self.removed
        )
    }
}

static PATCH_KEY_CODEC: StaticCodec<PatchKey> = LazyLock::new(|| {
    STRING
        .flat_xmap(
            |string: String| {
                let (removed, string) = match string.strip_prefix('!') {
                    Some(rest) => (true, rest.to_owned()),
                    None => (false, string),
                };
                let id = Identifier::try_parse(&string);
                match id.as_ref().and_then(DataComponentType::by_name) {
                    None => {
                        let shown = id.map_or_else(|| "null".to_owned(), |id| id.to_string());
                        DataResult::error(|| format!("No component with type: '{shown}'"))
                    }
                    Some(r#type) if r#type.is_transient() => {
                        let shown = r#type.key();
                        DataResult::error(|| format!("'{shown}' is not a persistent component"))
                    }
                    Some(r#type) => DataResult::success(PatchKey { r#type, removed }),
                }
            },
            |key: &PatchKey| {
                let id = key.r#type.key();
                DataResult::success(if key.removed {
                    format!("!{id}")
                } else {
                    id.to_string()
                })
            },
        )
        .boxed()
});

// `Codec.dispatchedMap(PatchKey.CODEC, PatchKey::valueCodec)`, made into a patch (a type set and
// then removed, or the other way, keeps its first place and the last value).
static CODEC: StaticCodec<DataComponentPatch> = LazyLock::new(|| {
    DispatchedMapCodec::new(PATCH_KEY_CODEC.shared(), |key: &PatchKey| key.value_codec())
        .xmap(
            |data: Vec<(PatchKey, Option<Tag>)>| {
                let mut map: Vec<(DataComponentType, Option<Tag>)> = Vec::new();
                for (key, value) in data {
                    match map.iter_mut().find(|(r#type, _)| *r#type == key.r#type) {
                        Some(entry) => entry.1 = value,
                        None => map.push((key.r#type, value)),
                    }
                }
                DataComponentPatch { map }
            },
            |patch: &DataComponentPatch| {
                patch
                    .map
                    .iter()
                    .filter(|(r#type, _)| !r#type.is_transient())
                    .map(|(r#type, value)| {
                        (
                            PatchKey {
                                r#type: *r#type,
                                removed: value.is_none(),
                            },
                            value.clone(),
                        )
                    })
                    .collect()
            },
        )
        .boxed()
});

/// Components set (their values as data) or removed (`None`), by type, in the order read.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DataComponentPatch {
    map: Vec<(DataComponentType, Option<Tag>)>,
}

impl DataComponentPatch {
    pub const EMPTY: DataComponentPatch = DataComponentPatch { map: Vec::new() };
    pub const CODEC: &'static StaticCodec<DataComponentPatch> = &CODEC;

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn size(&self) -> i32 {
        self.map.len() as i32
    }

    /// The set components' types and values as data, and the removed ones' types, in order.
    pub fn entries(&self) -> &[(DataComponentType, Option<Tag>)] {
        &self.map
    }
}

/// `{minecraft:damage=>5, !minecraft:lore}`: a value as the data it holds, where Java prints its
/// class's own `toString`.
impl fmt::Display for DataComponentPatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{")?;
        for (index, (r#type, value)) in self.map.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            match value {
                Some(value) => write!(f, "{type}=>{value}")?,
                None => write!(f, "!{type}")?,
            }
        }
        f.write_str("}")
    }
}
