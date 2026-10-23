//! net/minecraft/util/valueproviders/IntProviders.java
//!
//! An int provider's codec: a plain int is a constant, anything else is dispatched by its type's
//! name in `BuiltInRegistries.INT_PROVIDER_TYPE` (`byNameCodec`), which is `IntProviderType`
//! here, in `bootstrap`'s order. The types whose providers aren't ported have
//! `map_codec::unported`.

use std::sync::LazyLock;

use crate::com::mojang::datafixers::util::either::Either;
use crate::com::mojang::serialization::codec::{
    BoxedCodec, CodecExt, INT, SharedCodec, StaticCodec,
};
use crate::com::mojang::serialization::codecs::either_codec::EitherCodec;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::map_codec::{
    self, BoxedMapCodec, MapCodecExt, SharedMapCodec,
};
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::valueproviders::constant_int::ConstantInt;
use crate::net::minecraft::util::valueproviders::int_provider::IntProvider;
use crate::net::minecraft::util::valueproviders::uniform_int::UniformInt;

/// The types `IntProviders.bootstrap` registers, in its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum IntProviderType {
    CONSTANT,
    UNIFORM,
    BIASED_TO_BOTTOM,
    VERY_BIASED_TO_BOTTOM,
    CLAMPED,
    WEIGHTED_LIST,
    CLAMPED_NORMAL,
    TRAPEZOID,
}

impl IntProviderType {
    pub const VALUES: [IntProviderType; 8] = [
        IntProviderType::CONSTANT,
        IntProviderType::UNIFORM,
        IntProviderType::BIASED_TO_BOTTOM,
        IntProviderType::VERY_BIASED_TO_BOTTOM,
        IntProviderType::CLAMPED,
        IntProviderType::WEIGHTED_LIST,
        IntProviderType::CLAMPED_NORMAL,
        IntProviderType::TRAPEZOID,
    ];

    fn name(&self) -> &'static str {
        match self {
            IntProviderType::CONSTANT => "constant",
            IntProviderType::UNIFORM => "uniform",
            IntProviderType::BIASED_TO_BOTTOM => "biased_to_bottom",
            IntProviderType::VERY_BIASED_TO_BOTTOM => "very_biased_to_bottom",
            IntProviderType::CLAMPED => "clamped",
            IntProviderType::WEIGHTED_LIST => "weighted_list",
            IntProviderType::CLAMPED_NORMAL => "clamped_normal",
            IntProviderType::TRAPEZOID => "trapezoid",
        }
    }

    /// The registered map codec, as one of `IntProvider`.
    fn codec(&self) -> BoxedMapCodec<IntProvider> {
        match self {
            IntProviderType::CONSTANT => ConstantInt::MAP_CODEC
                .shared()
                .xmap(
                    IntProvider::Constant,
                    |provider: &IntProvider| match provider {
                        IntProvider::Constant(constant) => *constant,
                        other => panic!("ClassCastException: {other} isn't a ConstantInt"),
                    },
                )
                .boxed(),
            IntProviderType::UNIFORM => UniformInt::MAP_CODEC
                .shared()
                .xmap(
                    IntProvider::Uniform,
                    |provider: &IntProvider| match provider {
                        IntProvider::Uniform(uniform) => *uniform,
                        other => panic!("ClassCastException: {other} isn't a UniformInt"),
                    },
                )
                .boxed(),
            IntProviderType::BIASED_TO_BOTTOM => map_codec::unported("BiasedToBottomInt").boxed(),
            IntProviderType::VERY_BIASED_TO_BOTTOM => {
                map_codec::unported("VeryBiasedToBottomInt").boxed()
            }
            IntProviderType::CLAMPED => map_codec::unported("ClampedInt").boxed(),
            IntProviderType::WEIGHTED_LIST => map_codec::unported("WeightedListInt").boxed(),
            IntProviderType::CLAMPED_NORMAL => map_codec::unported("ClampedNormalInt").boxed(),
            IntProviderType::TRAPEZOID => map_codec::unported("TrapezoidInt").boxed(),
        }
    }

    fn of(provider: &IntProvider) -> IntProviderType {
        match provider {
            IntProvider::Constant(_) => IntProviderType::CONSTANT,
            IntProvider::Uniform(_) => IntProviderType::UNIFORM,
        }
    }
}

// `BuiltInRegistries.INT_PROVIDER_TYPE.byNameCodec()`.
static TYPE_CODEC: StaticCodec<IntProviderType> = LazyLock::new(|| {
    Identifier::CODEC
        .shared()
        .comap_flat_map(
            |name: Identifier| {
                let found = (name.get_namespace() == Identifier::DEFAULT_NAMESPACE)
                    .then(|| {
                        IntProviderType::VALUES
                            .into_iter()
                            .find(|t| t.name() == name.get_path())
                    })
                    .flatten();
                match found {
                    Some(found) => DataResult::success(found),
                    None => DataResult::error(move || {
                        format!(
                            "Unknown registry key in {}: {name}",
                            **Registries::INT_PROVIDER_TYPE
                        )
                    }),
                }
            },
            |t: &IntProviderType| Identifier::with_default_namespace(t.name()),
        )
        .boxed()
});

static CODEC: StaticCodec<IntProvider> = LazyLock::new(|| {
    let dispatch =
        TYPE_CODEC
            .shared()
            .dispatch("type", IntProviderType::of, |t: &IntProviderType| t.codec());
    EitherCodec::new(INT, dispatch)
        .xmap(
            |either: Either<i32, IntProvider>| {
                either.map(|value| IntProvider::Constant(ConstantInt::of(value)), |f| f)
            },
            |f: &IntProvider| match f {
                IntProvider::Constant(constant) => Either::Left(constant.value()),
                other => Either::Right(*other),
            },
        )
        .boxed()
});

pub struct IntProviders;

impl IntProviders {
    pub const CODEC: &'static StaticCodec<IntProvider> = &CODEC;

    /// `codec(min, max)`: `CODEC`, the provider's range inside the bounds.
    pub fn codec(min_value: i32, max_value: i32) -> BoxedCodec<IntProvider> {
        CODEC
            .shared()
            .validate(move |value| IntProviders::validate(min_value, max_value, value))
            .boxed()
    }

    fn validate(min_value: i32, max_value: i32, value: IntProvider) -> DataResult<IntProvider> {
        if value.min_inclusive() < min_value {
            DataResult::error(move || {
                format!(
                    "Value provider too low: {min_value} [{}-{}]",
                    value.min_inclusive(),
                    value.max_inclusive()
                )
            })
        } else if value.max_inclusive() > max_value {
            DataResult::error(move || {
                format!(
                    "Value provider too high: {max_value} [{}-{}]",
                    value.min_inclusive(),
                    value.max_inclusive()
                )
            })
        } else {
            DataResult::success(value)
        }
    }
}
