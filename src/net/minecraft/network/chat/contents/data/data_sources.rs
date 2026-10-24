//! net/minecraft/network/chat/contents/data/DataSources.java
//!
//! The kinds of data sources, by their field's name, matched like a component's contents
//! (`createLegacyComponentMatcher`, with the type under `source`).

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::map_codec::{
    BoxedMapCodec, MapCodecExt, SharedMapCodec, StaticMapCodec,
};
use crate::net::minecraft::network::chat::component_serialization::{
    ComponentSerialization, LegacyType,
};
use crate::net::minecraft::network::chat::contents::data::block_data_source::BlockDataSource;
use crate::net::minecraft::network::chat::contents::data::data_source::DataSource;
use crate::net::minecraft::network::chat::contents::data::entity_data_source::EntityDataSource;
use crate::net::minecraft::network::chat::contents::data::storage_data_source::StorageDataSource;
use crate::net::minecraft::util::extra_codecs::LateBoundIdMapper;

/// Each class's `MAP_CODEC`, in the order the `ID_MAPPER` has them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataSourceType {
    Entity,
    Block,
    Storage,
}

impl LegacyType for DataSourceType {
    type Value = DataSource;

    fn codec(self) -> BoxedMapCodec<DataSource> {
        match self {
            DataSourceType::Entity => EntityDataSource::MAP_CODEC.shared(),
            DataSourceType::Block => BlockDataSource::MAP_CODEC.shared(),
            DataSourceType::Storage => StorageDataSource::MAP_CODEC.shared(),
        }
    }

    fn of(value: &DataSource) -> DataSourceType {
        value.codec()
    }
}

/// `MapCodec.toString`, which no message prints.
impl fmt::Display for DataSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

static CODEC: StaticMapCodec<DataSource> = LazyLock::new(|| {
    let mut id_mapper = LateBoundIdMapper::new();
    id_mapper.put("entity".to_owned(), DataSourceType::Entity);
    id_mapper.put("block".to_owned(), DataSourceType::Block);
    id_mapper.put("storage".to_owned(), DataSourceType::Storage);
    ComponentSerialization::create_legacy_component_matcher(&id_mapper, "source").boxed()
});

pub struct DataSources;

impl DataSources {
    pub const CODEC: &'static StaticMapCodec<DataSource> = &CODEC;
}
