//! net/minecraft/network/chat/contents/data/StorageDataSource.java

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::network::chat::contents::data::data_source::DataSource;
use crate::net::minecraft::resources::identifier::Identifier;

static MAP_CODEC: StaticMapCodec<DataSource> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (Identifier::CODEC
            .shared()
            .field_of("storage")
            .for_getter(|source: &DataSource| match source {
                DataSource::Storage(storage) => storage.id.clone(),
                other => panic!("ClassCastException: {other}"),
            }),),
        |id| DataSource::Storage(StorageDataSource { id }),
    )
    .boxed()
});

/// The command storage under an id.
#[derive(Clone, Debug, PartialEq)]
pub struct StorageDataSource {
    pub id: Identifier,
}

impl StorageDataSource {
    pub const MAP_CODEC: &'static StaticMapCodec<DataSource> = &MAP_CODEC;
}

/// `storage=minecraft:id`.
impl fmt::Display for StorageDataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "storage={}", self.id)
    }
}
