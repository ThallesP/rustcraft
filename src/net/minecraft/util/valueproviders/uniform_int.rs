//! net/minecraft/util/valueproviders/UniformInt.java
//!
//! `sample` is `Mth.randomBetweenInclusive`, which the lightning rod's sparks draw with.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, INT};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::util::random_source::RandomSource;

static MAP_CODEC: StaticMapCodec<UniformInt> = LazyLock::new(|| {
    let check = |u: UniformInt| {
        if u.max_inclusive < u.min_inclusive {
            DataResult::error(move || {
                format!(
                    "Max must be at least min, min_inclusive: {}, max_inclusive: {}",
                    u.min_inclusive, u.max_inclusive
                )
            })
        } else {
            DataResult::success(u)
        }
    };
    record_codec_builder::map_codec(
        (
            INT.field_of("min_inclusive")
                .for_getter(|o: &UniformInt| o.min_inclusive),
            INT.field_of("max_inclusive")
                .for_getter(|o: &UniformInt| o.max_inclusive),
        ),
        UniformInt::of,
    )
    // MapCodec.validate: flatXmap with the checker both ways.
    .flat_xmap(check, move |u: &UniformInt| check(*u))
    .boxed()
});

/// An int drawn evenly from a range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UniformInt {
    min_inclusive: i32,
    max_inclusive: i32,
}

impl UniformInt {
    pub const MAP_CODEC: &'static StaticMapCodec<UniformInt> = &MAP_CODEC;

    pub fn of(min_inclusive: i32, max_inclusive: i32) -> UniformInt {
        UniformInt {
            min_inclusive,
            max_inclusive,
        }
    }

    pub fn min_inclusive(&self) -> i32 {
        self.min_inclusive
    }

    pub fn max_inclusive(&self) -> i32 {
        self.max_inclusive
    }

    /// `sample(random)`: `Mth.randomBetweenInclusive`.
    pub fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        random
            .next_int_bounded(
                self.max_inclusive
                    .wrapping_sub(self.min_inclusive)
                    .wrapping_add(1),
            )
            .wrapping_add(self.min_inclusive)
    }
}

impl fmt::Display for UniformInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}-{}]", self.min_inclusive, self.max_inclusive)
    }
}
