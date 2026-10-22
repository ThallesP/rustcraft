//! net/minecraft/world/level/block/state/StateDefinition.java
//!
//! Not ported yet: `propertiesCodec`, which waits for DataFixerUpper's codecs. Java's
//! `StateDefinition<O, S>` is a block's here (`O` a `Block`, `S` a `BlockState`); fluids' comes
//! with fluids.
//!
//! The states are the cartesian product of the properties' values, the properties sorted by name
//! and the last varying fastest (Guava's `Lists.cartesianProduct`), so a state is its index in
//! that list: a mixed-radix number whose digits are its values' indexes. A state's neighbour
//! with another value (Java's `neighbors` table, which `setValue` reads) is the index with that
//! digit changed. A definition is made once for each block and kept for the game's lifetime
//! (leaked), so a state can be a reference to its definition and an index (`BlockState`).

use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::property::Property;

/// A block's properties and the states they make.
pub struct StateDefinition {
    // The block and its first state's id, once it's registered.
    registration: OnceLock<(Block, u32)>,
    properties_by_name: Vec<&'static dyn Property>,
    // How far apart two states one value of each property apart are.
    strides: Vec<u32>,
    state_count: u32,
}

impl StateDefinition {
    fn new(mut properties: Vec<&'static dyn Property>) -> StateDefinition {
        // `ImmutableSortedMap`'s order, by name: names are lower case ASCII, so their UTF-16
        // order is their bytes'.
        properties.sort_by_key(|property| property.get_name());
        let mut strides = vec![1u32; properties.len()];
        for i in (0..properties.len().saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * properties[i + 1].possible_value_count() as u32;
        }
        let state_count = properties
            .first()
            .map_or(1, |first| strides[0] * first.possible_value_count() as u32);
        StateDefinition {
            registration: OnceLock::new(),
            properties_by_name: properties,
            strides,
            state_count,
        }
    }

    /// `getPossibleStates()`, in id order.
    pub fn get_possible_states(&'static self) -> impl ExactSizeIterator<Item = BlockState> {
        (0..self.state_count).map(move |index| BlockState::new(self, index))
    }

    /// The first of the states, every property at its first value.
    pub fn any(&'static self) -> BlockState {
        BlockState::new(self, 0)
    }

    /// Panics before the block is registered.
    pub fn get_owner(&self) -> Block {
        self.registration().0
    }

    /// `getProperties()`, sorted by name.
    pub fn get_properties(&self) -> &[&'static dyn Property] {
        &self.properties_by_name
    }

    pub fn get_property(&self, name: &str) -> Option<&'static dyn Property> {
        self.properties_by_name
            .iter()
            .find(|property| property.get_name() == name)
            .copied()
    }

    pub fn is_singleton_state(&self) -> bool {
        self.properties_by_name.is_empty()
    }

    pub fn state_count(&self) -> u32 {
        self.state_count
    }

    /// Where the property is among the definition's, compared by reference.
    pub(crate) fn property_index(&self, property: &dyn Property) -> Option<usize> {
        self.properties_by_name.iter().position(|candidate| {
            crate::net::minecraft::world::level::block::state::properties::property::same(
                *candidate, property,
            )
        })
    }

    /// The index among its property's values of the state's value for the property at
    /// `property_index`.
    pub(crate) fn value_index(&self, state: u32, property_index: usize) -> usize {
        let count = self.properties_by_name[property_index].possible_value_count() as u32;
        ((state / self.strides[property_index]) % count) as usize
    }

    /// The state with the property at `property_index` set to its value at `value_index`.
    pub(crate) fn with_value_index(
        &self,
        state: u32,
        property_index: usize,
        value_index: usize,
    ) -> u32 {
        let stride = self.strides[property_index];
        let old = self.value_index(state, property_index) as u32;
        state - old * stride + value_index as u32 * stride
    }

    /// Ties the definition to its block, whose states take ids from `first_id` on.
    pub(crate) fn register(&self, owner: Block, first_id: u32) {
        assert!(
            self.registration.set((owner, first_id)).is_ok(),
            "IllegalStateException: a state definition is registered once"
        );
    }

    pub(crate) fn first_id(&self) -> u32 {
        self.registration().1
    }

    fn registration(&self) -> (Block, u32) {
        *self
            .registration
            .get()
            .expect("a state definition's block isn't registered yet")
    }
}

/// `MoreObjects.toStringHelper`: the block and the properties' names.
impl fmt::Display for StateDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self
            .properties_by_name
            .iter()
            .map(|property| property.get_name())
            .collect();
        write!(
            f,
            "StateDefinition{{block={}, properties=[{}]}}",
            self.get_owner(),
            names.join(", ")
        )
    }
}

/// `StateDefinition.Builder`, which a block's `createBlockStateDefinition` adds its properties
/// to. Its owner only names the block in Java's error messages, so it's left out.
#[derive(Default)]
pub struct Builder {
    properties: HashMap<&'static str, &'static dyn Property>,
}

/// `NAME_PATTERN`, `^[a-z0-9_]+$`.
fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

impl Builder {
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Panics on a property with a bad name, one value or fewer, a value with a bad name, or a
    /// name already added, where Java throws `IllegalArgumentException`.
    pub fn add(&mut self, properties: &[&'static dyn Property]) -> &mut Builder {
        for &property in properties {
            let name = property.get_name();
            assert!(
                is_valid_name(name),
                "IllegalArgumentException: invalidly named property {name}"
            );
            assert!(
                property.possible_value_count() > 1,
                "IllegalArgumentException: property {name} with <= 1 possible values"
            );
            for value in property.get_possible_values() {
                let value_name = property.get_name_of(value);
                assert!(
                    is_valid_name(&value_name),
                    "IllegalArgumentException: property {name} with invalidly named value {value_name}"
                );
            }
            assert!(
                !self.properties.contains_key(name),
                "IllegalArgumentException: duplicate property {name}"
            );
            self.properties.insert(name, property);
        }
        self
    }

    /// `create(defaultState, factory)`: the definition, kept for the game's lifetime.
    pub fn create(self) -> &'static StateDefinition {
        Box::leak(Box::new(StateDefinition::new(
            self.properties.into_values().collect(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::Builder;
    use crate::golden::Golden;
    use crate::net::minecraft::world::level::block::state::properties::block_state_properties::tests::{
        field_name, property,
    };

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.block.state.StateDefinition");
        golden.check(
            "Builder.add([Lnet/minecraft/world/level/block/state/properties/Property;).create",
            |names: Vec<String>| {
                let mut builder = Builder::new();
                for name in &names {
                    builder.add(&[property(name)]);
                }
                let definition = builder.create();
                let fields: Vec<String> = definition
                    .get_properties()
                    .iter()
                    .map(|property| field_name(*property).to_owned())
                    .collect();
                (fields, definition.state_count() as i32)
            },
        );
        golden.finish();
    }
}
