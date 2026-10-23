//! net/minecraft/util/valueproviders/ConstantInt.java
//!
//! `sample`, which a server's worldgen and mob spawning call, comes with its callers.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, INT};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};

static MAP_CODEC: StaticMapCodec<ConstantInt> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (INT.field_of("value").for_getter(|o: &ConstantInt| o.value),),
        ConstantInt::of,
    )
    .boxed()
});

/// An int that's always the same.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstantInt {
    value: i32,
}

impl ConstantInt {
    pub const ZERO: ConstantInt = ConstantInt { value: 0 };
    pub const MAP_CODEC: &'static StaticMapCodec<ConstantInt> = &MAP_CODEC;

    /// `of`: `ZERO` for 0, which Java shares.
    pub fn of(value: i32) -> ConstantInt {
        ConstantInt { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn min_inclusive(&self) -> i32 {
        self.value
    }

    pub fn max_inclusive(&self) -> i32 {
        self.value
    }
}

impl fmt::Display for ConstantInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
