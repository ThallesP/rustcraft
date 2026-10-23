//! net/minecraft/util/StringRepresentable.java
//!
//! `getSerializedName`, the name a block state property writes its values by, and the codecs of
//! values by it: `fromEnum` and `fromValues` (`EnumCodec` and `StringRepresentableCodec`), which
//! are `orCompressed` of `Codec.stringResolver` and an id codec, and so the former for every ops
//! the game's codecs reach (the id codec isn't ported). `EnumCodec.byName` and `keys` come with
//! their callers.

use crate::com::mojang::serialization::codec::{BoxedCodec, CodecExt, STRING};
use crate::com::mojang::serialization::data_result::DataResult;

/// A value with a name of its own for saving and sending, `north` for `Direction.NORTH`.
pub trait StringRepresentable {
    fn get_serialized_name(&self) -> &'static str;
}

/// `fromEnum` and `fromValues`: a value by its serialized name, looked up among `values` in order
/// (`createNameLookup`, a map of them past 16, which finds the same).
pub fn from_values<E>(values: &'static [E]) -> BoxedCodec<E>
where
    E: StringRepresentable + Copy + Send + Sync + 'static,
{
    STRING
        .flat_xmap(
            move |name: String| match values.iter().find(|e| e.get_serialized_name() == name) {
                Some(value) => DataResult::success(*value),
                None => DataResult::error(|| format!("Unknown element name:{name}")),
            },
            |e: &E| DataResult::success(e.get_serialized_name().to_owned()),
        )
        .boxed()
}
