//! net/minecraft/world/level/chunk/Strategy.java
//!
//! Java's two anonymous subclasses, the block states' and the biomes', differ only in
//! `getConfigurationForBitCount`, so a strategy says which it is (`Kind`) and the method matches
//! on it. A strategy is shared by the containers it makes, like Java's reference to it, so they
//! hold it in an `Arc`.

use std::sync::Arc;

use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::chunk::configuration::Configuration;
use crate::net::minecraft::world::level::chunk::global_palette::GlobalPalette;
use crate::net::minecraft::world::level::chunk::palette::Factory;

const SINGLE_VALUE_PALETTE_FACTORY: Factory = Factory::SingleValue;
const LINEAR_PALETTE_FACTORY: Factory = Factory::Linear;
const HASHMAP_PALETTE_FACTORY: Factory = Factory::HashMap;
const ZERO_BITS: Configuration = simple(SINGLE_VALUE_PALETTE_FACTORY, 0);
const ONE_BIT_LINEAR: Configuration = simple(LINEAR_PALETTE_FACTORY, 1);
const TWO_BITS_LINEAR: Configuration = simple(LINEAR_PALETTE_FACTORY, 2);
const THREE_BITS_LINEAR: Configuration = simple(LINEAR_PALETTE_FACTORY, 3);
const FOUR_BITS_LINEAR: Configuration = simple(LINEAR_PALETTE_FACTORY, 4);
const FIVE_BITS_HASHMAP: Configuration = simple(HASHMAP_PALETTE_FACTORY, 5);
const SIX_BITS_HASHMAP: Configuration = simple(HASHMAP_PALETTE_FACTORY, 6);
const SEVEN_BITS_HASHMAP: Configuration = simple(HASHMAP_PALETTE_FACTORY, 7);
const EIGHT_BITS_HASHMAP: Configuration = simple(HASHMAP_PALETTE_FACTORY, 8);

const fn simple(factory: Factory, bits: i32) -> Configuration {
    Configuration::Simple { factory, bits }
}

/// Which of Java's two subclasses a strategy is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    BlockStates,
    Biomes,
}

/// How a container of a section's values (16³ block states, or 4³ biomes) numbers them, and
/// which palette it takes for how many bits.
pub struct Strategy<T> {
    global_map: Arc<dyn IdMap<T> + Send + Sync>,
    global_palette: GlobalPalette<T>,
    global_palette_bits_in_memory: i32,
    bits_per_axis: i32,
    entry_count: i32,
    kind: Kind,
}

impl<T> Strategy<T> {
    fn new(
        global_map: Arc<dyn IdMap<T> + Send + Sync>,
        bits_per_axis: i32,
        kind: Kind,
    ) -> Strategy<T> {
        let global_palette_bits_in_memory =
            Strategy::<T>::minimum_bits_required_for_distinct_values(global_map.size());
        Strategy {
            global_palette: GlobalPalette::new(Arc::clone(&global_map)),
            global_map,
            global_palette_bits_in_memory,
            bits_per_axis,
            entry_count: 1 << (bits_per_axis * 3),
            kind,
        }
    }

    /// 16 by 16 by 16 values: a single value, a linear palette of up to 4 bits, a hash map
    /// palette of 5 to 8, and the registry past that.
    pub fn create_for_block_states(registry: Arc<dyn IdMap<T> + Send + Sync>) -> Strategy<T> {
        Strategy::new(registry, 4, Kind::BlockStates)
    }

    /// 4 by 4 by 4 values: a single value, a linear palette of up to 3 bits, and the registry
    /// past that.
    pub fn create_for_biomes(registry: Arc<dyn IdMap<T> + Send + Sync>) -> Strategy<T> {
        Strategy::new(registry, 2, Kind::Biomes)
    }

    pub fn entry_count(&self) -> i32 {
        self.entry_count
    }

    /// Where a position's value is: y, then z, then x.
    #[inline]
    pub fn get_index(&self, x: i32, y: i32, z: i32) -> i32 {
        (y << self.bits_per_axis | z) << self.bits_per_axis | x
    }

    pub fn global_map(&self) -> &(dyn IdMap<T> + Send + Sync) {
        &*self.global_map
    }

    pub fn global_palette(&self) -> &GlobalPalette<T> {
        &self.global_palette
    }

    pub fn get_configuration_for_bit_count(&self, entry_bits: i32) -> Configuration {
        match (self.kind, entry_bits) {
            (_, 0) => ZERO_BITS,
            (Kind::BlockStates, 1..=4) => FOUR_BITS_LINEAR,
            (Kind::BlockStates, 5) => FIVE_BITS_HASHMAP,
            (Kind::BlockStates, 6) => SIX_BITS_HASHMAP,
            (Kind::BlockStates, 7) => SEVEN_BITS_HASHMAP,
            (Kind::BlockStates, 8) => EIGHT_BITS_HASHMAP,
            (Kind::Biomes, 1) => ONE_BIT_LINEAR,
            (Kind::Biomes, 2) => TWO_BITS_LINEAR,
            (Kind::Biomes, 3) => THREE_BITS_LINEAR,
            _ => Configuration::Global {
                bits_in_memory: self.global_palette_bits_in_memory,
                bits_in_storage: entry_bits,
            },
        }
    }

    pub fn get_configuration_for_palette_size(&self, palette_size: i32) -> Configuration {
        let bits = Strategy::<T>::minimum_bits_required_for_distinct_values(palette_size);
        self.get_configuration_for_bit_count(bits)
    }

    fn minimum_bits_required_for_distinct_values(count: i32) -> i32 {
        Mth::ceillog2(count)
    }
}
