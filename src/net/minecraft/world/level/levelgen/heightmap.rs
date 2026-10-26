//! net/minecraft/world/level/levelgen/Heightmap.java
//!
//! Each column's height above its highest block that counts: `SimpleBitStorage` of
//! `ceillog2(height + 1)` bits for each of a chunk's 256 columns, which the client takes from the
//! server's chunk packets (`setRawData`) and keeps up as it sets blocks (`update`). The types'
//! predicates (`isOpaque`) read block tags, which answer from the vanilla data pack like every
//! block tag until `update_tags` is ported. Not ported yet: `primeHeightmaps`, which works out a
//! chunk's heights from its blocks (the server's, and the client's for data that didn't fit), and
//! `Types.CODEC`.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::bit_storage::BitStorage;
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::simple_bit_storage::SimpleBitStorage;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

/// A chunk's heights for one of the types.
#[derive(Clone, Debug)]
pub struct Heightmap {
    data: SimpleBitStorage,
    // The type's `isOpaque`.
    r#type: Types,
    // `chunk.getMinY()`, which Java asks its chunk for.
    min_y: i32,
}

impl Heightmap {
    /// `Heightmap(chunk, type)` for a chunk `height` blocks high from `min_y`.
    pub fn new(min_y: i32, height: i32, r#type: Types) -> Heightmap {
        let height_bits = Mth::ceillog2(height + 1);
        Heightmap {
            data: SimpleBitStorage::new(height_bits, 256),
            r#type,
            min_y,
        }
    }

    /// `update(localX, localY, localZ, state)`: the column's height after the block at `local_y`
    /// became `state`, the chunk's blocks below read through `chunk` where the column's top block
    /// stopped counting; whether it changed.
    pub fn update(
        &mut self,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        state: BlockState,
        chunk: &dyn BlockGetter,
    ) -> bool {
        let first_available = self.get_first_available(local_x, local_z);
        if local_y <= first_available - 2 {
            return false;
        }

        if self.r#type.is_opaque(state) {
            if local_y >= first_available {
                self.set_height(local_x, local_z, local_y + 1);
                return true;
            }
        } else if first_available - 1 == local_y {
            for y in (self.min_y..local_y).rev() {
                let pos = BlockPos::new(local_x, y, local_z);
                if self.r#type.is_opaque(chunk.get_block_state(&pos)) {
                    self.set_height(local_x, local_z, y + 1);
                    return true;
                }
            }

            self.set_height(local_x, local_z, self.min_y);
            return true;
        }

        false
    }

    fn set_height(&mut self, x: i32, z: i32, height: i32) {
        self.data.set(get_index(x, z), height - self.min_y);
    }

    fn get_first_available_by_index(&self, index: i32) -> i32 {
        self.data.get(index) + self.min_y
    }

    /// The height above the column's highest block that counts, `min_y` when none does.
    pub fn get_first_available(&self, x: i32, z: i32) -> i32 {
        self.get_first_available_by_index(get_index(x, z))
    }

    pub fn get_highest_taken(&self, x: i32, z: i32) -> i32 {
        self.get_first_available_by_index(get_index(x, z)) - 1
    }

    /// `setRawData(chunk, type, data)`: the server's longs, which replace these when there are as
    /// many as the storage holds, and whether they did. Otherwise Java logs a warning and primes
    /// the heightmap from the chunk's blocks, which isn't ported, so the heights stay as they were.
    pub fn set_raw_data(&mut self, data: &[i64]) -> bool {
        let raw = self.data.get_raw_mut();
        if raw.len() == data.len() {
            raw.copy_from_slice(data);
            true
        } else {
            false
        }
    }

    pub fn get_raw_data(&self) -> &[i64] {
        self.data.get_raw()
    }
}

fn get_index(x: i32, z: i32) -> i32 {
    x + z * 16
}

static TYPES_BY_ID: LazyLock<Box<dyn Fn(i32) -> Types + Send + Sync>> = LazyLock::new(|| {
    Box::new(ByIdMap::continuous(
        |t: &Types| t.id(),
        &Types::VALUES,
        OutOfBoundsStrategy::ZERO,
    ))
});

static TYPES_STREAM_CODEC: StaticStreamCodec<ByteBuf, Types> =
    LazyLock::new(|| Box::new(ByteBufCodecs::id_mapper(&**TYPES_BY_ID, |t: &Types| t.id())));

/// Which blocks a heightmap counts, and who keeps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Types {
    WORLD_SURFACE_WG,
    WORLD_SURFACE,
    OCEAN_FLOOR_WG,
    OCEAN_FLOOR,
    MOTION_BLOCKING,
    MOTION_BLOCKING_NO_LEAVES,
}

/// Who keeps a heightmap: world generation, the server's level, or the client too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usage {
    WORLDGEN,
    LIVE_WORLD,
    CLIENT,
}

impl Types {
    pub const VALUES: [Types; 6] = [
        Types::WORLD_SURFACE_WG,
        Types::WORLD_SURFACE,
        Types::OCEAN_FLOOR_WG,
        Types::OCEAN_FLOOR,
        Types::MOTION_BLOCKING,
        Types::MOTION_BLOCKING_NO_LEAVES,
    ];

    /// A type by its id, the first type for any other.
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Types> = &TYPES_STREAM_CODEC;

    pub fn id(&self) -> i32 {
        *self as i32
    }

    pub fn get_serialization_key(&self) -> &'static str {
        match self {
            Types::WORLD_SURFACE_WG => "WORLD_SURFACE_WG",
            Types::WORLD_SURFACE => "WORLD_SURFACE",
            Types::OCEAN_FLOOR_WG => "OCEAN_FLOOR_WG",
            Types::OCEAN_FLOOR => "OCEAN_FLOOR",
            Types::MOTION_BLOCKING => "MOTION_BLOCKING",
            Types::MOTION_BLOCKING_NO_LEAVES => "MOTION_BLOCKING_NO_LEAVES",
        }
    }

    pub fn usage(&self) -> Usage {
        match self {
            Types::WORLD_SURFACE_WG | Types::OCEAN_FLOOR_WG => Usage::WORLDGEN,
            Types::OCEAN_FLOOR => Usage::LIVE_WORLD,
            Types::WORLD_SURFACE | Types::MOTION_BLOCKING | Types::MOTION_BLOCKING_NO_LEAVES => {
                Usage::CLIENT
            }
        }
    }

    pub fn send_to_client(&self) -> bool {
        self.usage() == Usage::CLIENT
    }

    pub fn keep_after_worldgen(&self) -> bool {
        self.usage() != Usage::WORLDGEN
    }

    /// `isOpaque()`: whether the type counts a block. `NOT_AIR` for the surfaces,
    /// `MATERIAL_MOTION_BLOCKING` (the tag) for the ocean floors, and the motion blocking ones
    /// count fluids too.
    pub fn is_opaque(&self, state: BlockState) -> bool {
        match self {
            Types::WORLD_SURFACE_WG | Types::WORLD_SURFACE => !state.is_air(),
            Types::OCEAN_FLOOR_WG | Types::OCEAN_FLOOR => {
                state.is_tag(BlockTags::BLOCKS_MOTION_IN_HEIGHTMAP)
            }
            Types::MOTION_BLOCKING => {
                state.is_tag(BlockTags::BLOCKS_MOTION_IN_HEIGHTMAP)
                    || !state.get_fluid_state().is_empty()
            }
            Types::MOTION_BLOCKING_NO_LEAVES => {
                state.is_tag(BlockTags::BLOCKS_MOTION_IN_HEIGHTMAP_NO_LEAVES)
                    || !state.get_fluid_state().is_empty()
            }
        }
    }
}

impl Enum for Types {
    const ENUM_CONSTANTS: &'static [Types] = &Types::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::Types;
    use crate::golden::{Golden, unhex};
    use crate::net::minecraft::client::multiplayer::client_chunk_cache::tests::overworld;
    use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::registry_reader;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::chunk::level_chunk_section::tests::registries;

    const TYPES: [Types; 4] = [
        Types::WORLD_SURFACE,
        Types::OCEAN_FLOOR,
        Types::MOTION_BLOCKING,
        Types::MOTION_BLOCKING_NO_LEAVES,
    ];

    /// The walk's chunks around the spawn, blocks set in a few columns through the client's
    /// level: each column's heights after each block, and the heightmaps' data at the end, as
    /// vanilla's `LevelChunk.setBlockState` updates them.
    #[test]
    fn heights_follow_blocks_like_vanillas() {
        let chunks = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket",
        );
        let registries = registries(&chunks.input("BIOMES"));
        let golden = Golden::load("net.minecraft.world.level.levelgen.Heightmap");
        let programs = golden.cases("programs");
        assert_eq!(programs.len(), 9);
        for (args, expected) in programs {
            let mut level = overworld(&registries, 8);
            let mut input = registry_reader(unhex(args[0].as_str().expect("hex")));
            let packet = ClientboundLevelChunkWithLightPacket::STREAM_CODEC.decode(&mut input);
            ClientPacketListener::handle_level_chunk_with_light_in(&mut level, &packet);
            let (chunk_x, chunk_z) = (packet.x(), packet.z());
            let heights =
                |level: &crate::net::minecraft::client::multiplayer::client_level::ClientLevel,
                 x: i32,
                 z: i32| {
                    let chunk = level
                        .get_chunk_source()
                        .get_chunk(chunk_x, chunk_z)
                        .expect("the chunk");
                    TYPES
                        .map(|r#type| {
                            chunk
                                .get_heightmap(r#type)
                                .expect("a full chunk's heightmap")
                                .get_first_available(x & 15, z & 15)
                        })
                        .to_vec()
                };
            let expected = expected.as_array().expect("outputs");
            let ops = args[1].as_array().expect("ops");
            assert_eq!(expected.len(), ops.len() + 1);
            for (index, op) in ops.iter().enumerate() {
                let [x, y, z, state] = [0, 1, 2, 3].map(|i| op[i].as_i64().expect("an int") as i32);
                let state = Block::block_state_registry()
                    .by_id(state)
                    .expect("a state the registry has");
                let pos = BlockPos::new(x, y, z);
                // The golden sets the chunk's state alone (`LevelChunk.setBlockState`): its shape
                // known, so no neighbour's changes with it.
                let changed = level.set_block(&pos, state, 19, 512);
                let actual = if changed {
                    json!(heights(&level, x, z))
                } else {
                    Value::Null
                };
                assert_eq!(actual, expected[index], "op {index}: {state} at {pos:?}");
            }
            let chunk = level
                .get_chunk_source()
                .get_chunk(chunk_x, chunk_z)
                .expect("the chunk");
            let raw: Vec<Value> = TYPES
                .map(|r#type| {
                    json!(
                        chunk
                            .get_heightmap(r#type)
                            .expect("a full chunk's heightmap")
                            .get_raw_data()
                    )
                })
                .to_vec();
            assert_eq!(json!(raw), expected[ops.len()]);
        }
        golden.finish();
    }
}
