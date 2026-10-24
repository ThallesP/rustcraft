//! net/minecraft/network/chat/contents/ScoreContents.java
//!
//! A score on the server's scoreboard, which the server resolves (`resolve`, not ported) before it
//! sends a component; one sent unresolved shows nothing. The name is a selector where it parses
//! as one, else the name itself, as `Codec.either` decodes it.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::datafixers::util::either::Either;
use crate::com::mojang::serialization::codec::{CodecExt, STRING};
use crate::com::mojang::serialization::codecs::either_codec::EitherCodec;
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::commands::arguments::selector::entity_selector::EntitySelector;
use crate::net::minecraft::network::chat::component_contents::ComponentContents;
use crate::net::minecraft::util::compilable_string::CompilableString;

#[derive(Clone, Debug, PartialEq)]
pub struct ScoreContents {
    name: Either<CompilableString<EntitySelector>, String>,
    objective: String,
}

static INNER_CODEC: StaticMapCodec<ScoreContents> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            EitherCodec::new(EntitySelector::compilable_codec(), STRING)
                .field_of("name")
                .for_getter(|contents: &ScoreContents| contents.name.clone()),
            STRING
                .field_of("objective")
                .for_getter(|contents: &ScoreContents| contents.objective.clone()),
        ),
        ScoreContents::new,
    )
    .boxed()
});

// `INNER_CODEC.fieldOf("score")`.
static MAP_CODEC: StaticMapCodec<ComponentContents> = LazyLock::new(|| {
    (*INNER_CODEC)
        .clone()
        .field_of("score")
        .xmap(
            ComponentContents::Score,
            |contents: &ComponentContents| match contents {
                ComponentContents::Score(score) => score.clone(),
                other => panic!("ClassCastException: {other}"),
            },
        )
        .boxed()
});

impl ScoreContents {
    pub const INNER_CODEC: &'static StaticMapCodec<ScoreContents> = &INNER_CODEC;
    pub const MAP_CODEC: &'static StaticMapCodec<ComponentContents> = &MAP_CODEC;

    pub fn new(
        name: Either<CompilableString<EntitySelector>, String>,
        objective: String,
    ) -> ScoreContents {
        ScoreContents { name, objective }
    }

    pub fn name(&self) -> &Either<CompilableString<EntitySelector>, String> {
        &self.name
    }

    pub fn objective(&self) -> &str {
        &self.objective
    }
}

/// `score{name='Left[@p]', objective='kills'}`.
impl fmt::Display for ScoreContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "score{{name='{}', objective='{}'}}",
            self.name, self.objective
        )
    }
}
