//! net/minecraft/world/level/chunk/PalettedContainer.java
//!
//! Not ported yet: `codecRW`, `codecRO`, `pack` and `unpack`, the format chunks are saved in on
//! disk (DataFixerUpper's codecs), which the client doesn't read.
//!
//! `PalettedContainerRO`, the read-only view a section holds its biomes as, is only implemented
//! by this class, so a section holds a `PalettedContainer` and its methods are this one's. The
//! `ThreadingDetector` behind `acquire` and `release` is `&mut self`.
//!
//! A palette that fills up calls the container back (`onResize`), which replaces the palette it
//! was called on. Rust can't lend the container to its own palette, so the container hands the
//! palette a handler that only notes the bits asked for, and makes the new palette once the
//! palette's `idFor` has returned: each palette returns `onResize`'s answer as soon as it has it,
//! so that's the same. The data holds its storage and palette as enums of the kinds there are,
//! which dispatch with a `match`: `get` is what every block lookup calls.
//!
//! `count` hands each value out in the order it first comes in the storage, where Java's goes by
//! an `Int2IntOpenHashMap`'s table; its only caller (`LevelChunkSection.recalcBlockCounts`) adds
//! them up. `getAll` goes in that order too, like Java's `IntArraySet`.
//!
//! `own_palette` and `id_at` are the port's: `get` in two steps, the palette's values once and the
//! id at a position each time, for a caller that works something out of each value once
//! (`ChunkSkyLightSources.fillFrom`).

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::util::bit_storage::BitStorage;
use crate::net::minecraft::util::simple_bit_storage::SimpleBitStorage;
use crate::net::minecraft::util::zero_bit_storage::ZeroBitStorage;
use crate::net::minecraft::world::level::chunk::configuration::Configuration;
use crate::net::minecraft::world::level::chunk::palette::{AnyPalette, Palette};
use crate::net::minecraft::world::level::chunk::palette_resize::{
    PaletteResize, no_resize_expected,
};
use crate::net::minecraft::world::level::chunk::strategy::Strategy;

/// A section's values, 16³ block states or 4³ biomes: a palette of the values there are, and a
/// storage of each position's id in it.
pub struct PalettedContainer<T> {
    data: Data<T>,
    strategy: Arc<Strategy<T>>,
}

impl<T: Clone + PartialEq + Hash> PalettedContainer<T> {
    /// `PalettedContainer(initialValue, strategy)`: every position holds the value.
    pub fn new(initial_value: T, strategy: Arc<Strategy<T>>) -> PalettedContainer<T> {
        let data = PalettedContainer::create_or_reuse_data(&strategy, None, 0)
            .expect("there's no data to reuse");
        let mut container = PalettedContainer { data, strategy };
        container.id_for(&initial_value);
        container
    }

    pub fn strategy(&self) -> &Arc<Strategy<T>> {
        &self.strategy
    }

    /// The data for that many bits: `None` when the old data's configuration is the one, which
    /// Java hands back to reuse.
    fn create_or_reuse_data(
        strategy: &Strategy<T>,
        old_data: Option<&Data<T>>,
        target_bits: i32,
    ) -> Option<Data<T>> {
        let data_configuration = strategy.get_configuration_for_bit_count(target_bits);
        if old_data.is_some_and(|old_data| data_configuration == old_data.configuration) {
            return None;
        }

        let storage = if data_configuration.bits_in_memory() == 0 {
            Storage::Zero(ZeroBitStorage::new(strategy.entry_count()))
        } else {
            Storage::Simple(SimpleBitStorage::new(
                data_configuration.bits_in_memory(),
                strategy.entry_count(),
            ))
        };
        let palette = data_configuration.create_palette(strategy, &[]);
        Some(Data {
            configuration: data_configuration,
            storage,
            palette,
        })
    }

    /// `onResize`: the palette for that many bits, the values copied over to it, and the id of
    /// the value that didn't fit.
    fn on_resize(&mut self, bits: i32, last_added_value: &T) -> i32 {
        match PalettedContainer::create_or_reuse_data(&self.strategy, Some(&self.data), bits) {
            Some(mut new_data) => {
                new_data.copy_from(&self.data.palette, &self.data.storage);
                self.data = new_data;
            }
            // The data copies from itself: from a copy of it, which reads the same.
            None => {
                let old_palette = self.data.palette.clone();
                let old_storage = self.data.storage.clone();
                self.data.copy_from(&old_palette, &old_storage);
            }
        }
        self.data
            .palette
            .id_for(last_added_value, &mut no_resize_expected())
    }

    /// `palette.idFor(value, this)`.
    fn id_for(&mut self, value: &T) -> i32 {
        let mut request = ResizeRequest(None);
        let id = self.data.palette.id_for(value, &mut request);
        match request.0 {
            Some(bits) => self.on_resize(bits, value),
            None => id,
        }
    }

    /// Sets the value and returns the one it replaced. The value goes in the palette before the
    /// position is checked, so it's there even when the position isn't in the container (which
    /// panics, where Java throws `IllegalArgumentException`).
    pub fn get_and_set(&mut self, x: i32, y: i32, z: i32, value: &T) -> T {
        let index = self.strategy.get_index(x, y, z);
        self.get_and_set_by_index(index, value)
    }

    pub fn get_and_set_unchecked(&mut self, x: i32, y: i32, z: i32, value: &T) -> T {
        let index = self.strategy.get_index(x, y, z);
        self.get_and_set_by_index(index, value)
    }

    fn get_and_set_by_index(&mut self, index: i32, value: &T) -> T {
        let id = self.id_for(value);
        let old_id = self.data.storage.get_and_set(index, id);
        self.data.palette.value_for(old_id)
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, value: &T) {
        let index = self.strategy.get_index(x, y, z);
        self.set_by_index(index, value);
    }

    fn set_by_index(&mut self, index: i32, value: &T) {
        let id = self.id_for(value);
        self.data.storage.set(index, id);
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> T {
        self.get_by_index(self.strategy.get_index(x, y, z))
    }

    /// Java's protected `get(int)`.
    #[inline]
    pub fn get_by_index(&self, index: i32) -> T {
        self.data.palette.value_for(self.data.storage.get(index))
    }

    /// The palette's values by id, where the container has a palette of its own: none for the
    /// global palette, whose ids are the registry's.
    pub fn own_palette(&self) -> Option<Vec<T>> {
        if matches!(self.data.palette, AnyPalette::Global(_)) {
            return None;
        }
        Some(
            (0..self.data.palette.get_size())
                .map(|id| self.data.palette.value_for(id))
                .collect(),
        )
    }

    /// The palette id at a position: `get`'s value is the palette's value for it.
    #[inline]
    pub fn id_at(&self, x: i32, y: i32, z: i32) -> i32 {
        self.data.storage.get(self.strategy.get_index(x, y, z))
    }

    /// Each value there is, once.
    pub fn get_all(&self, consumer: &mut dyn FnMut(T)) {
        let palette = &self.data.palette;
        for id in self.distinct_ids() {
            consumer(palette.value_for(id));
        }
    }

    // The ids in the storage, each once, in the order they first come.
    fn distinct_ids(&self) -> Vec<i32> {
        let mut seen = std::collections::HashSet::new();
        let mut ids = Vec::new();
        self.data.storage.get_all(&mut |id| {
            if seen.insert(id) {
                ids.push(id);
            }
        });
        ids
    }

    /// Reads the bits per entry, the palette and the longs, into the data in place when the
    /// configuration is the one it has, so a read that panics halfway leaves that data as Java
    /// does (and new data unused).
    pub fn read(&mut self, buffer: &mut FriendlyByteBuf) {
        let new_bits = buffer.read_byte() as i32;
        match PalettedContainer::create_or_reuse_data(&self.strategy, Some(&self.data), new_bits) {
            None => {
                self.data.palette.read(buffer, self.strategy.global_map());
                buffer.read_fixed_size_long_array(self.data.storage.get_raw_mut());
            }
            Some(mut new_data) => {
                new_data.palette.read(buffer, self.strategy.global_map());
                buffer.read_fixed_size_long_array(new_data.storage.get_raw_mut());
                self.data = new_data;
            }
        }
    }

    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        self.data.write(buffer, &self.strategy);
    }

    pub fn get_serialized_size(&self) -> i32 {
        1 + self
            .data
            .palette
            .get_serialized_size(self.strategy.global_map())
            + self.data.storage.get_raw().len() as i32 * 8
    }

    pub fn bits_per_entry(&self) -> i32 {
        self.data.storage.get_bits()
    }

    pub fn maybe_has(&self, predicate: &mut dyn FnMut(&T) -> bool) -> bool {
        self.data.palette.maybe_has(predicate)
    }

    pub fn for_each_in_palette(&self, consumer: &mut dyn FnMut(T)) {
        for i in 0..self.data.palette.get_size() {
            consumer(self.data.palette.value_for(i));
        }
    }

    pub fn copy(&self) -> PalettedContainer<T> {
        PalettedContainer {
            data: self.data.copy(),
            strategy: Arc::clone(&self.strategy),
        }
    }

    /// A container of the palette's first value everywhere.
    pub fn recreate(&self) -> PalettedContainer<T> {
        PalettedContainer::new(self.data.palette.value_for(0), Arc::clone(&self.strategy))
    }

    /// Each value there is with how many positions hold it (`CountConsumer`).
    pub fn count(&self, output: &mut dyn FnMut(&T, i32)) {
        if self.data.palette.get_size() == 1 {
            output(
                &self.data.palette.value_for(0),
                self.data.storage.get_size(),
            );
        } else {
            let mut ids = Vec::new();
            let mut counts: HashMap<i32, i32> = HashMap::new();
            self.data.storage.get_all(&mut |id| {
                *counts.entry(id).or_insert_with(|| {
                    ids.push(id);
                    0
                }) += 1;
            });
            for id in ids {
                output(&self.data.palette.value_for(id), counts[&id]);
            }
        }
    }
}

/// What a palette's resize asks, noted for the container to do once the palette returns.
struct ResizeRequest(Option<i32>);

impl<T> PaletteResize<T> for ResizeRequest {
    fn on_resize(&mut self, bits: i32, _last_added_value: &T) -> i32 {
        self.0 = Some(bits);
        // Never used: the container answers in its place.
        -1
    }
}

/// `BitStorage`, as one of the two kinds a container holds.
#[derive(Clone)]
enum Storage {
    Zero(ZeroBitStorage),
    Simple(SimpleBitStorage),
}

macro_rules! dispatch {
    ($self:expr, $storage:ident => $call:expr) => {
        match $self {
            Storage::Zero($storage) => $call,
            Storage::Simple($storage) => $call,
        }
    };
}

impl BitStorage for Storage {
    fn get_and_set(&mut self, index: i32, value: i32) -> i32 {
        dispatch!(self, storage => storage.get_and_set(index, value))
    }

    fn set(&mut self, index: i32, value: i32) {
        dispatch!(self, storage => storage.set(index, value))
    }

    fn fill(&mut self, value: i32) {
        dispatch!(self, storage => storage.fill(value))
    }

    #[inline]
    fn get(&self, index: i32) -> i32 {
        dispatch!(self, storage => storage.get(index))
    }

    fn get_raw(&self) -> &[i64] {
        dispatch!(self, storage => storage.get_raw())
    }

    fn get_raw_mut(&mut self) -> &mut [i64] {
        dispatch!(self, storage => storage.get_raw_mut())
    }

    fn get_size(&self) -> i32 {
        dispatch!(self, storage => storage.get_size())
    }

    fn get_bits(&self) -> i32 {
        dispatch!(self, storage => storage.get_bits())
    }

    fn get_all(&self, output: &mut dyn FnMut(i32)) {
        dispatch!(self, storage => storage.get_all(output))
    }

    fn unpack(&self, output: &mut [i32]) {
        dispatch!(self, storage => storage.unpack(output))
    }
}

/// `PalettedContainer.Data`: a configuration, and the storage and palette it made.
struct Data<T> {
    configuration: Configuration,
    storage: Storage,
    palette: AnyPalette<T>,
}

impl<T: Clone + PartialEq + Hash> Data<T> {
    fn copy_from(&mut self, old_palette: &AnyPalette<T>, old_storage: &Storage) {
        let mut dummy_resizer = no_resize_expected();
        if old_palette.get_size() == 1 {
            let value = old_palette.value_for(0);
            let id = self.palette.id_for(&value, &mut dummy_resizer);
            self.storage.fill(id);
        } else {
            let mut last_old_id = -1;
            let mut new_id = -1;

            for i in 0..old_storage.get_size() {
                let old_id = old_storage.get(i);
                if old_id != last_old_id {
                    let value = old_palette.value_for(old_id);
                    new_id = self.palette.id_for(&value, &mut dummy_resizer);
                    last_old_id = old_id;
                }

                self.storage.set(i, new_id);
            }
        }
    }

    fn write(&self, buffer: &mut FriendlyByteBuf, strategy: &Strategy<T>) {
        buffer.write_byte(self.storage.get_bits());
        self.palette.write(buffer, strategy.global_map());
        buffer.write_fixed_size_long_array(self.storage.get_raw());
    }

    fn copy(&self) -> Data<T> {
        Data {
            configuration: self.configuration,
            storage: self.storage.clone(),
            palette: self.palette.copy(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeMap;
    use std::hash::Hash;
    use std::sync::Arc;

    use serde_json::{Value, json};

    use super::PalettedContainer;
    use crate::golden::{Golden, hex, readable, unhex};
    use crate::io::netty::buffer::{unpooled, unpooled::wrapped_buffer};
    use crate::net::minecraft::core::id_map::IdMap;
    use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::net::minecraft::world::level::chunk::strategy::Strategy;

    /// The golden suite's `BiomeStandIn`: a biome of a registry of stand-ins, by its id.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct BiomeStandIn(i32);

    /// A registry of that many stand-ins.
    pub(crate) struct StandIns(pub(crate) i32);

    impl IdMap<BiomeStandIn> for StandIns {
        fn get_id(&self, thing: &BiomeStandIn) -> i32 {
            if (0..self.0).contains(&thing.0) {
                thing.0
            } else {
                -1
            }
        }

        fn by_id(&self, id: i32) -> Option<BiomeStandIn> {
            (0..self.0).contains(&id).then_some(BiomeStandIn(id))
        }

        fn size(&self) -> i32 {
            self.0
        }
    }

    pub(crate) fn block_strategy() -> Arc<Strategy<BlockState>> {
        let registry: Arc<dyn IdMap<BlockState> + Send + Sync> =
            Arc::new(Block::block_state_registry());
        Arc::new(Strategy::create_for_block_states(registry))
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.chunk.PalettedContainer");
        let blocks = block_strategy();
        golden.check("programs", |registry: String, initial: i32, ops: Value| {
            if registry == "blocks" {
                run(&blocks, initial, &ops)
            } else {
                let size = registry
                    .strip_prefix("biomes:")
                    .and_then(|size| size.parse().ok())
                    .expect("a registry is blocks or biomes:<size>");
                let biomes = Arc::new(Strategy::create_for_biomes(Arc::new(StandIns(size))));
                run(&biomes, initial, &ops)
            }
        });
        golden.finish();
    }

    // Mirrors PalettedContainerSuite.run in oracle/golden/: `[output, bitsPerEntry,
    // getSerializedSize]` after each op, a panic ending them. Values are their ids.
    fn run<T: Clone + PartialEq + Hash>(
        strategy: &Arc<Strategy<T>>,
        initial: i32,
        ops: &Value,
    ) -> Vec<Value> {
        let map = strategy.global_map();
        let id = |value: &T| Value::from(map.get_id(value));
        let mut container =
            PalettedContainer::new(map.by_id_or_throw(initial), Arc::clone(strategy));
        let mut steps = Vec::new();
        for op in ops.as_array().expect("a program is an array of ops") {
            let op = op.as_array().expect("an op is an array");
            let int = |index: usize| op[index].as_i64().expect("an int") as i32;
            let value = |index: usize| map.by_id_or_throw(int(index));
            let step = crate::golden::catch(|| {
                let output = match op[0].as_str().expect("ops start with their name") {
                    "set" => {
                        container.set(int(1), int(2), int(3), &value(4));
                        Value::Null
                    }
                    "getAndSet" => id(&container.get_and_set(int(1), int(2), int(3), &value(4))),
                    "getAndSetUnchecked" => {
                        id(&container.get_and_set_unchecked(int(1), int(2), int(3), &value(4)))
                    }
                    "get" => id(&container.get(int(1), int(2), int(3))),
                    "maybeHas" => {
                        let value = value(1);
                        Value::from(container.maybe_has(&mut |candidate| *candidate == value))
                    }
                    "count" => {
                        let mut counts = BTreeMap::new();
                        container.count(&mut |value, count| {
                            *counts.entry(map.get_id(value)).or_insert(0) += count;
                        });
                        Value::from(
                            counts
                                .into_iter()
                                .map(|(id, count)| json!([id, count]))
                                .collect::<Vec<_>>(),
                        )
                    }
                    "getAll" => {
                        let mut values = Vec::new();
                        container.get_all(&mut |value| values.push(id(&value)));
                        Value::from(values)
                    }
                    "forEachInPalette" => {
                        let mut values = Vec::new();
                        container.for_each_in_palette(&mut |value| {
                            if values.len() < 512 {
                                values.push(id(&value));
                            }
                        });
                        Value::from(values)
                    }
                    "values" => values(&container, |value| id(value)),
                    "write" => {
                        let mut buffer = FriendlyByteBuf::new(unpooled::buffer());
                        container.write(&mut buffer);
                        Value::from(hex(&readable(&buffer)))
                    }
                    "copy" => {
                        container = container.copy();
                        Value::Null
                    }
                    "recreate" => {
                        container = container.recreate();
                        Value::Null
                    }
                    "read" => {
                        let bytes = unhex(op[1].as_str().expect("a read's bytes are hex"));
                        let mut buffer = FriendlyByteBuf::new(wrapped_buffer(bytes));
                        container.read(&mut buffer);
                        Value::from(buffer.readable_bytes())
                    }
                    "bitsPerEntry" => Value::from(container.bits_per_entry()),
                    "getSerializedSize" => Value::from(container.get_serialized_size()),
                    name => panic!("unknown op {name}"),
                };
                json!([
                    output,
                    container.bits_per_entry(),
                    container.get_serialized_size()
                ])
            });
            match step {
                Ok(step) => steps.push(step),
                Err(_) => {
                    steps.push(json!({ "throws": "panic" }));
                    break;
                }
            }
        }
        steps
    }

    /// Every value in index order (y, then z, then x), as runs: `[[id, count], …]`.
    pub(crate) fn values<T: Clone + PartialEq + Hash>(
        container: &PalettedContainer<T>,
        id: impl Fn(&T) -> Value,
    ) -> Value {
        let side = if container.strategy().entry_count() == 4096 {
            16
        } else {
            4
        };
        let mut runs: Vec<(Value, i32)> = Vec::new();
        for y in 0..side {
            for z in 0..side {
                for x in 0..side {
                    let value = id(&container.get(x, y, z));
                    match runs.last_mut() {
                        Some((last, count)) if *last == value => *count += 1,
                        _ => runs.push((value, 1)),
                    }
                }
            }
        }
        Value::from(
            runs.into_iter()
                .map(|(value, count)| json!([value, count]))
                .collect::<Vec<_>>(),
        )
    }
}
