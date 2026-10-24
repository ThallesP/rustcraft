//! net/minecraft/network/chat/contents/data/DataSource.java
//!
//! Where an NBT component's tags come from: an entity, a block entity, or the command storage.
//! Java's interface of three records is an enum here; reading the tags (`getData`) is the server's.

use std::fmt;

use crate::net::minecraft::network::chat::contents::data::block_data_source::BlockDataSource;
use crate::net::minecraft::network::chat::contents::data::data_sources::DataSourceType;
use crate::net::minecraft::network::chat::contents::data::entity_data_source::EntityDataSource;
use crate::net::minecraft::network::chat::contents::data::storage_data_source::StorageDataSource;

#[derive(Clone, Debug, PartialEq)]
pub enum DataSource {
    Entity(EntityDataSource),
    Block(BlockDataSource),
    Storage(StorageDataSource),
}

impl DataSource {
    /// `codec()`.
    pub fn codec(&self) -> DataSourceType {
        match self {
            DataSource::Entity(_) => DataSourceType::Entity,
            DataSource::Block(_) => DataSourceType::Block,
            DataSource::Storage(_) => DataSourceType::Storage,
        }
    }
}

impl fmt::Display for DataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataSource::Entity(source) => source.fmt(f),
            DataSource::Block(source) => source.fmt(f),
            DataSource::Storage(source) => source.fmt(f),
        }
    }
}
