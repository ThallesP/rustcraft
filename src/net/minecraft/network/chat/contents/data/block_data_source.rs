//! net/minecraft/network/chat/contents/data/BlockDataSource.java

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{BoxedCodec, CodecExt};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::commands::arguments::coordinates::block_pos_argument::BlockPosArgument;
use crate::net::minecraft::commands::arguments::coordinates::coordinates::Coordinates;
use crate::net::minecraft::network::chat::contents::data::data_source::DataSource;
use crate::net::minecraft::util::compilable_string::{CompilableString, parse_command};

static BLOCK_POS_CODEC: LazyLock<BoxedCodec<CompilableString<Coordinates>>> = LazyLock::new(|| {
    CompilableString::codec(|contents| {
        parse_command(
            contents,
            |reader| BlockPosArgument::block_pos().parse(reader),
            |original, exception| {
                format!(
                    "Invalid coordinates path: {original}: {}",
                    exception.get_message()
                )
            },
        )
    })
});

static MAP_CODEC: StaticMapCodec<DataSource> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        ((*BLOCK_POS_CODEC).clone().field_of("block").for_getter(
            |source: &DataSource| match source {
                DataSource::Block(block) => block.coordinates.clone(),
                other => panic!("ClassCastException: {other}"),
            },
        ),),
        |coordinates| DataSource::Block(BlockDataSource { coordinates }),
    )
    .boxed()
});

/// The block entity at a position.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockDataSource {
    pub coordinates: CompilableString<Coordinates>,
}

impl BlockDataSource {
    pub const MAP_CODEC: &'static StaticMapCodec<DataSource> = &MAP_CODEC;

    /// `BLOCK_POS_CODEC`.
    pub fn block_pos_codec() -> BoxedCodec<CompilableString<Coordinates>> {
        (*BLOCK_POS_CODEC).clone()
    }
}

/// The record's `toString`.
impl fmt::Display for BlockDataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockDataSource[coordinates={}]", self.coordinates)
    }
}
