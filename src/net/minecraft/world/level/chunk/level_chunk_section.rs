//! net/minecraft/world/level/chunk/LevelChunkSection.java
//!
//! Not ported yet: `fillBiomesFromNoise`, which world generation calls. `acquire` and `release`
//! guard the states from other threads, which `&mut self` rules out, and `setBlockState`'s
//! `checkThreading` with them. A section holds its biomes as a `PalettedContainer`, the one class
//! implementing Java's read-only `PalettedContainerRO`.

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::chunk::paletted_container::PalettedContainer;
use crate::net::minecraft::world::level::chunk::paletted_container_factory::PalettedContainerFactory;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;

/// 16³ blocks of a chunk and their 4³ biomes, with counts of the blocks that aren't air, the
/// fluids, and those that tick randomly.
pub struct LevelChunkSection {
    non_empty_block_count: i16,
    fluid_count: i16,
    ticking_block_count: i16,
    ticking_fluid_count: i16,
    states: PalettedContainer<BlockState>,
    biomes: PalettedContainer<Holder>,
}

impl LevelChunkSection {
    pub const BIOME_CONTAINER_BITS: i32 = 2;

    /// `LevelChunkSection(states, biomes)`, which counts the blocks.
    pub fn from_containers(
        states: PalettedContainer<BlockState>,
        biomes: PalettedContainer<Holder>,
    ) -> LevelChunkSection {
        let mut section = LevelChunkSection {
            non_empty_block_count: 0,
            fluid_count: 0,
            ticking_block_count: 0,
            ticking_fluid_count: 0,
            states,
            biomes,
        };
        section.recalc_block_counts();
        section
    }

    /// `LevelChunkSection(containerFactory)`: air, and the default biome.
    pub fn new(container_factory: &PalettedContainerFactory) -> LevelChunkSection {
        LevelChunkSection {
            non_empty_block_count: 0,
            fluid_count: 0,
            ticking_block_count: 0,
            ticking_fluid_count: 0,
            states: container_factory.create_for_block_states(),
            biomes: container_factory.create_for_biomes(),
        }
    }

    #[inline]
    pub fn get_block_state(&self, section_x: i32, section_y: i32, section_z: i32) -> BlockState {
        self.states.get(section_x, section_y, section_z)
    }

    pub fn get_fluid_state(&self, section_x: i32, section_y: i32, section_z: i32) -> FluidState {
        self.states
            .get(section_x, section_y, section_z)
            .get_fluid_state()
    }

    /// Sets the state and returns the one it replaced, keeping the counts.
    pub fn set_block_state(
        &mut self,
        section_x: i32,
        section_y: i32,
        section_z: i32,
        state: BlockState,
    ) -> BlockState {
        let previous = self
            .states
            .get_and_set(section_x, section_y, section_z, &state);
        if !previous.is_air() {
            self.non_empty_block_count = self.non_empty_block_count.wrapping_sub(1);
            if previous.is_randomly_ticking() {
                self.ticking_block_count = self.ticking_block_count.wrapping_sub(1);
            }

            let previous_fluid = previous.get_fluid_state();
            if !previous_fluid.is_empty() {
                self.fluid_count = self.fluid_count.wrapping_sub(1);
                if previous_fluid.is_randomly_ticking() {
                    self.ticking_fluid_count = self.ticking_fluid_count.wrapping_sub(1);
                }
            }
        }

        if !state.is_air() {
            self.non_empty_block_count = self.non_empty_block_count.wrapping_add(1);
            if state.is_randomly_ticking() {
                self.ticking_block_count = self.ticking_block_count.wrapping_add(1);
            }

            let fluid = state.get_fluid_state();
            if !fluid.is_empty() {
                self.fluid_count = self.fluid_count.wrapping_add(1);
                if fluid.is_randomly_ticking() {
                    self.ticking_fluid_count = self.ticking_fluid_count.wrapping_add(1);
                }
            }
        }

        previous
    }

    pub fn has_only_air(&self) -> bool {
        self.non_empty_block_count == 0
    }

    pub fn has_fluid(&self) -> bool {
        self.fluid_count > 0
    }

    pub fn is_randomly_ticking(&self) -> bool {
        self.is_randomly_ticking_blocks() || self.is_randomly_ticking_fluids()
    }

    pub fn is_randomly_ticking_blocks(&self) -> bool {
        self.ticking_block_count > 0
    }

    pub fn is_randomly_ticking_fluids(&self) -> bool {
        self.ticking_fluid_count > 0
    }

    /// Counts the blocks again from the states, each value once with how many there are.
    pub fn recalc_block_counts(&mut self) {
        let mut non_empty_block_count = 0i32;
        let mut fluid_count = 0i32;
        let mut ticking_block_count = 0i32;
        let mut ticking_fluid_count = 0i32;
        self.states.count(&mut |state, count| {
            if !state.is_air() {
                non_empty_block_count += count;
                if state.is_randomly_ticking() {
                    ticking_block_count += count;
                }

                let fluid = state.get_fluid_state();
                if !fluid.is_empty() {
                    fluid_count += count;
                    if fluid.is_randomly_ticking() {
                        ticking_fluid_count += count;
                    }
                }
            }
        });
        self.non_empty_block_count = non_empty_block_count as i16;
        self.fluid_count = fluid_count as i16;
        self.ticking_block_count = ticking_block_count as i16;
        self.ticking_fluid_count = ticking_fluid_count as i16;
    }

    pub fn get_states(&self) -> &PalettedContainer<BlockState> {
        &self.states
    }

    pub fn get_biomes(&self) -> &PalettedContainer<Holder> {
        &self.biomes
    }

    /// The counts the server sent (not the ticking ones, which stay as they were), the states and
    /// the biomes.
    pub fn read(&mut self, buffer: &mut FriendlyByteBuf) {
        self.non_empty_block_count = buffer.read_short();
        self.fluid_count = buffer.read_short();
        self.states.read(buffer);
        let mut biomes = self.biomes.recreate();
        biomes.read(buffer);
        self.biomes = biomes;
    }

    pub fn read_biomes(&mut self, buffer: &mut FriendlyByteBuf) {
        let mut biomes = self.biomes.recreate();
        biomes.read(buffer);
        self.biomes = biomes;
    }

    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_short(self.non_empty_block_count as i32);
        buffer.write_short(self.fluid_count as i32);
        self.states.write(buffer);
        self.biomes.write(buffer);
    }

    pub fn get_serialized_size(&self) -> i32 {
        4 + self.states.get_serialized_size() + self.biomes.get_serialized_size()
    }

    pub fn maybe_has(&self, predicate: &mut dyn FnMut(&BlockState) -> bool) -> bool {
        self.states.maybe_has(predicate)
    }

    pub fn get_noise_biome(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Holder {
        self.biomes.get(quart_x, quart_y, quart_z)
    }

    pub fn copy(&self) -> LevelChunkSection {
        LevelChunkSection {
            non_empty_block_count: self.non_empty_block_count,
            fluid_count: self.fluid_count,
            ticking_block_count: self.ticking_block_count,
            ticking_fluid_count: self.ticking_fluid_count,
            states: self.states.copy(),
            biomes: self.biomes.copy(),
        }
    }

    /// Its four counts, which Java keeps private: non-empty blocks, fluids, ticking blocks and
    /// ticking fluids.
    pub fn counts(&self) -> [i16; 4] {
        [
            self.non_empty_block_count,
            self.fluid_count,
            self.ticking_block_count,
            self.ticking_fluid_count,
        ]
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::{Value, json};

    use super::LevelChunkSection;
    use crate::golden::{Decode, Encode, Golden, catch, hex, readable, unhex};
    use crate::io::netty::buffer::unpooled::{self, wrapped_buffer};
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::id_map::IdMap;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::core::registry::Registry;
    use crate::net::minecraft::core::registry_access::RegistryAccess;
    use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::net::minecraft::world::level::chunk::paletted_container_factory::PalettedContainerFactory;

    /// The registries with the biomes a golden file names, in its order.
    pub(crate) fn registries(biomes: &Value) -> RegistryAccess {
        let keys = Vec::<String>::decode(biomes)
            .iter()
            .map(|name| ResourceKey::create(Registries::BIOME, Identifier::parse(name)))
            .collect();
        RegistryAccess::new(vec![Registry::new((**Registries::BIOME).clone(), keys)])
    }

    /// A section as the goldens encode it: `[counts, statesBits, states, biomesBits, biomes]`,
    /// its states and biomes by id in index order, as runs.
    pub(crate) fn encode_section(section: &LevelChunkSection, biomes: &Registry) -> Value {
        let states = runs(16, |x, y, z| {
            Value::from(section.get_block_state(x, y, z).id())
        });
        let biome_ids = runs(4, |x, y, z| {
            Value::from(biomes.get_id(&section.get_noise_biome(x, y, z)))
        });
        json!([
            section.counts(),
            section.get_states().bits_per_entry(),
            states,
            section.get_biomes().bits_per_entry(),
            biome_ids
        ])
    }

    fn runs(side: i32, lookup: impl Fn(i32, i32, i32) -> Value) -> Value {
        let mut runs: Vec<(Value, i32)> = Vec::new();
        for y in 0..side {
            for z in 0..side {
                for x in 0..side {
                    let value = lookup(x, y, z);
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

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.chunk.LevelChunkSection");
        golden.field(
            "BIOME_CONTAINER_BITS",
            LevelChunkSection::BIOME_CONTAINER_BITS,
        );
        let registries = registries(&golden.input("BIOMES"));
        let factory = PalettedContainerFactory::create(&registries);
        let biomes = registries.lookup_or_throw(Registries::BIOME);
        golden.check("programs", |ops: Value| run(&factory, biomes, &ops));
        golden.finish();
    }

    // Mirrors LevelChunkSectionSuite.run in oracle/golden/.
    fn run(factory: &PalettedContainerFactory, biomes: &Registry, ops: &Value) -> Vec<Value> {
        let mut section = LevelChunkSection::new(factory);
        let mut steps = Vec::new();
        for op in ops.as_array().expect("ops") {
            let int = |index: usize| op[index].as_i64().expect("an int") as i32;
            let state = |index: usize| BlockState::decode(&op[index]);
            let biome_id = |biome: Holder| Value::from(biomes.get_id(&biome));
            let step = catch(|| {
                let output = match op[0].as_str().expect("an op's name") {
                    "setBlockState" => Value::from(
                        section
                            .set_block_state(int(1), int(2), int(3), state(4))
                            .id(),
                    ),
                    "getBlockState" => {
                        Value::from(section.get_block_state(int(1), int(2), int(3)).id())
                    }
                    "getFluidState" => section.get_fluid_state(int(1), int(2), int(3)).encode(),
                    "getNoiseBiome" => biome_id(section.get_noise_biome(int(1), int(2), int(3))),
                    "recalcBlockCounts" => {
                        section.recalc_block_counts();
                        Value::Null
                    }
                    "write" => {
                        let mut buffer = FriendlyByteBuf::new(unpooled::buffer());
                        section.write(&mut buffer);
                        Value::from(hex(&readable(&buffer)))
                    }
                    "read" | "readBiomes" => {
                        let bytes = unhex(op[1].as_str().expect("hex"));
                        let mut buffer = FriendlyByteBuf::new(wrapped_buffer(bytes));
                        if op[0] == "read" {
                            section.read(&mut buffer);
                        } else {
                            section.read_biomes(&mut buffer);
                        }
                        Value::from(buffer.readable_bytes())
                    }
                    "getSerializedSize" => Value::from(section.get_serialized_size()),
                    "maybeHas" => {
                        let wanted = state(1);
                        Value::from(section.maybe_has(&mut |candidate| *candidate == wanted))
                    }
                    "copy" => {
                        section = section.copy();
                        Value::Null
                    }
                    name => panic!("unknown op {name}"),
                };
                json!([
                    output,
                    section.counts(),
                    section.has_only_air(),
                    section.has_fluid(),
                    section.is_randomly_ticking(),
                    section.is_randomly_ticking_blocks(),
                    section.is_randomly_ticking_fluids()
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
}
