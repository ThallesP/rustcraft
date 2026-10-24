//! net/minecraft/network/chat/contents/data/EntityDataSource.java

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::CodecExt;
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::commands::arguments::selector::entity_selector::EntitySelector;
use crate::net::minecraft::network::chat::contents::data::data_source::DataSource;
use crate::net::minecraft::util::compilable_string::CompilableString;

static MAP_CODEC: StaticMapCodec<DataSource> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (EntitySelector::compilable_codec()
            .field_of("entity")
            .for_getter(|source: &DataSource| match source {
                DataSource::Entity(entity) => entity.selector.clone(),
                other => panic!("ClassCastException: {other}"),
            }),),
        |selector| DataSource::Entity(EntityDataSource { selector }),
    )
    .boxed()
});

/// The entities a selector finds.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityDataSource {
    pub selector: CompilableString<EntitySelector>,
}

impl EntityDataSource {
    pub const MAP_CODEC: &'static StaticMapCodec<DataSource> = &MAP_CODEC;
}

/// The record's `toString`.
impl fmt::Display for EntityDataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EntityDataSource[selector={}]", self.selector)
    }
}
