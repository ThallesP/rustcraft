//! net/minecraft/core/component/Removed.java
//!
//! What a patch holds for a component it removes: its codec reads any map (`MapCodec.unitCodec`)
//! and writes an empty one.

use std::sync::LazyLock;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::{BoxedCodec, StaticCodec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};

static CODEC: StaticCodec<Removed> = LazyLock::new(|| BoxedCodec::new(UnitCodec));

/// The removed component's mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Removed;

impl Removed {
    pub const INSTANCE: Removed = Removed;
    pub const CODEC: &'static StaticCodec<Removed> = &CODEC;
}

// `MapCodec.unitCodec(INSTANCE)`: a map (a list for an ops that compresses maps, which none of the
// game's codecs reach) read as the mark, which writes as an empty map.
#[derive(Clone, Copy)]
struct UnitCodec;

impl Valued for UnitCodec {
    type Value = Removed;
}

impl<T: OpsValue> Decoder<T> for UnitCodec {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Removed, T>> {
        ops.get_map(input).map(|_| Pair::of(Removed, input.clone()))
    }
}

impl<T: OpsValue> Encoder<T> for UnitCodec {
    fn encode(&self, _input: &Removed, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        ops.map_builder().build(prefix)
    }
}
