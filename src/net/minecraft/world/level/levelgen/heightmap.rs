//! net/minecraft/world/level/levelgen/Heightmap.java
//!
//! Each column's height above its highest block that counts: `SimpleBitStorage` of
//! `ceillog2(height + 1)` bits for each of a chunk's 256 columns, which the client takes from the
//! server's chunk packets (`setRawData`). Not ported yet: `primeHeightmaps`, `update` and
//! `setHeight`, which work out heights from blocks (the client only does when it sets a block),
//! and the types' predicates (`isOpaque`), which need block tags; `Types.CODEC` waits for
//! DataFixerUpper.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::util::bit_storage::BitStorage;
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::simple_bit_storage::SimpleBitStorage;

/// A chunk's heights for one of the types.
#[derive(Clone, Debug)]
pub struct Heightmap {
    data: SimpleBitStorage,
    // `chunk.getMinY()`, which Java asks its chunk for.
    min_y: i32,
}

impl Heightmap {
    /// `Heightmap(chunk, type)` for a chunk `height` blocks high from `min_y`.
    pub fn new(min_y: i32, height: i32) -> Heightmap {
        let height_bits = Mth::ceillog2(height + 1);
        Heightmap {
            data: SimpleBitStorage::new(height_bits, 256),
            min_y,
        }
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
}

impl Enum for Types {
    const ENUM_CONSTANTS: &'static [Types] = &Types::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}
