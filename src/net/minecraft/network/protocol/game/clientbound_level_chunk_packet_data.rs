//! net/minecraft/network/protocol/game/ClientboundLevelChunkPacketData.java
//!
//! Not ported yet: the constructor making the data from a `LevelChunk`, `extractChunkData` and
//! `calculateChunkSize`, the server's. The heightmaps are Java's `EnumMap`, which iterates in the
//! types' order, so a `BTreeMap` by type: the server writes its own map's order (a `HashMap`'s),
//! which the client doesn't keep. A block entity's type is read from the built-in registry,
//! `ByteBufCodecs.registry(BLOCK_ENTITY_TYPE)`, which the client's registry access holds as it is.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::io::netty::buffer::unpooled::wrapped_buffer;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::levelgen::heightmap::Types;

static HEIGHTMAPS_STREAM_CODEC: StaticStreamCodec<ByteBuf, BTreeMap<Types, Vec<i64>>> =
    LazyLock::new(|| {
        Box::new(ByteBufCodecs::map(
            |_size| BTreeMap::new(),
            Types::STREAM_CODEC,
            ByteBufCodecs::LONG_ARRAY,
        ))
    });

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundLevelChunkPacketData> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite3(
            &HEIGHTMAPS_STREAM_CODEC,
            |data: &ClientboundLevelChunkPacketData| data.heightmaps.clone(),
            ByteBufCodecs::byte_array(2097152),
            |data: &ClientboundLevelChunkPacketData| data.buffer.clone(),
            (&BLOCK_ENTITY_INFO_STREAM_CODEC).apply(ByteBufCodecs::list()),
            |data: &ClientboundLevelChunkPacketData| data.block_entities_data.clone(),
            ClientboundLevelChunkPacketData::new,
        ))
    });

// `ByteBufCodecs.registry(Registries.BLOCK_ENTITY_TYPE)`: a type by its id in the built-in
// registry, in a VarInt.
pub(crate) static BLOCK_ENTITY_TYPE_STREAM_CODEC: StaticStreamCodec<ByteBuf, BlockEntityType> =
    LazyLock::new(|| {
        Box::new(ByteBufCodecs::id_mapper(
            BlockEntityType::by_id_or_throw,
            |r#type: &BlockEntityType| r#type.id(),
        ))
    });

static BLOCK_ENTITY_INFO_STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, BlockEntityInfo> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite4(
            ByteBufCodecs::BYTE,
            |info: &BlockEntityInfo| info.packed_xz,
            ByteBufCodecs::SHORT,
            |info: &BlockEntityInfo| info.y,
            &BLOCK_ENTITY_TYPE_STREAM_CODEC,
            |info: &BlockEntityInfo| info.r#type,
            ByteBufCodecs::OPTIONAL_COMPOUND_TAG,
            |info: &BlockEntityInfo| info.tag.clone(),
            BlockEntityInfo::new,
        ))
    });

/// A chunk's blocks as the server sends them: its heightmaps, its sections' bytes and its block
/// entities.
#[derive(Clone, Debug)]
pub struct ClientboundLevelChunkPacketData {
    heightmaps: BTreeMap<Types, Vec<i64>>,
    buffer: Vec<i8>,
    block_entities_data: Vec<BlockEntityInfo>,
}

impl ClientboundLevelChunkPacketData {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundLevelChunkPacketData,
    > = &STREAM_CODEC;

    fn new(
        heightmaps: BTreeMap<Types, Vec<i64>>,
        buffer: Vec<i8>,
        block_entities_data: Vec<BlockEntityInfo>,
    ) -> ClientboundLevelChunkPacketData {
        ClientboundLevelChunkPacketData {
            heightmaps,
            buffer,
            block_entities_data,
        }
    }

    /// Each block entity's position in the chunk at `chunk_x`, `chunk_z`, type and tag, if any.
    pub fn for_each_block_entity_tag(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        output: &mut dyn FnMut(BlockPos, BlockEntityType, Option<&CompoundTag>),
    ) {
        let base_x = 16i32.wrapping_mul(chunk_x);
        let base_z = 16i32.wrapping_mul(chunk_z);

        for data in &self.block_entities_data {
            let unpacked_x =
                base_x.wrapping_add(SectionPos::section_relative((data.packed_xz >> 4) as i32));
            let unpacked_z =
                base_z.wrapping_add(SectionPos::section_relative(data.packed_xz as i32));
            let pos = BlockPos::new(unpacked_x, data.y as i32, unpacked_z);
            output(pos, data.r#type, data.tag.as_ref());
        }
    }

    /// The sections' bytes, to read from.
    pub fn get_read_buffer(&self) -> FriendlyByteBuf {
        FriendlyByteBuf::new(wrapped_buffer(self.buffer.clone()))
    }

    pub fn get_heightmaps(&self) -> &BTreeMap<Types, Vec<i64>> {
        &self.heightmaps
    }
}

/// A block entity: its x and z in the chunk (4 bits each, in a byte), its y, type and tag.
#[derive(Clone, Debug)]
struct BlockEntityInfo {
    packed_xz: i8,
    y: i16,
    r#type: BlockEntityType,
    tag: Option<CompoundTag>,
}

impl BlockEntityInfo {
    fn new(
        packed_xz: i8,
        y: i16,
        r#type: BlockEntityType,
        tag: Option<CompoundTag>,
    ) -> BlockEntityInfo {
        BlockEntityInfo {
            packed_xz,
            y,
            r#type,
            tag,
        }
    }
}
