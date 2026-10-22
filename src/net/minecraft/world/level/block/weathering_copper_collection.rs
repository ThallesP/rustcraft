//! net/minecraft/world/level/block/WeatheringCopperCollection.java
//!
//! Not ported yet: `registerItems`, which comes with items, `createFamily` (`BlockFamily`, for data
//! generation) and `zipApply`/`zipUnwaxedWaxed`. `registerBlocks` takes the registry `Blocks`
//! registers into, and always registers with `Blocks.register`, the one function vanilla passes
//! it.

use crate::net::minecraft::world::level::block::block::{Block, BlockData};
use crate::net::minecraft::world::level::block::blocks::{self, BlockKey, Registrar};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::weathering_copper::WeatherState;

/// A thing for each state of weathering, unwaxed and waxed, like the eight copper blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WeatheringCopperCollection<T> {
    weathering: ByState<T>,
    waxed: ByState<T>,
}

/// `WeatheringCopperCollection.ByState`: a thing for each state of weathering, in its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ByState<T>([T; 4]);

impl<T> ByState<T> {
    pub fn new(unaffected: T, exposed: T, weathered: T, oxidized: T) -> ByState<T> {
        ByState([unaffected, exposed, weathered, oxidized])
    }

    pub fn create(value: T) -> ByState<T>
    where
        T: Clone,
    {
        ByState(std::array::from_fn(|_| value.clone()))
    }

    pub fn map<U>(&self, mapper: impl FnMut(&T) -> U) -> ByState<U> {
        ByState(self.0.each_ref().map(mapper))
    }

    pub fn pick(&self, state: WeatherState) -> &T {
        &self.0[state as usize]
    }

    /// `forEach`'s order: unaffected, exposed, weathered, oxidized.
    pub fn as_list(&self) -> &[T; 4] {
        &self.0
    }

    /// Each state and the next: unaffected to exposed, exposed to weathered, weathered to
    /// oxidized.
    pub fn progress_mapping(&self, mut consumer: impl FnMut(&T, &T)) {
        for pair in self.0.windows(2) {
            consumer(&pair[0], &pair[1]);
        }
    }

    pub fn zip_map<U, R>(
        first: &ByState<T>,
        second: &ByState<U>,
        mut operation: impl FnMut(&T, &U) -> R,
    ) -> ByState<R> {
        let mut index = 0;
        ByState(std::array::from_fn(|_| {
            let result = operation(&first.0[index], &second.0[index]);
            index += 1;
            result
        }))
    }
}

impl ByState<WeatherState> {
    /// `STATES`: each state itself.
    pub const STATES: ByState<WeatherState> = ByState(WeatherState::VALUES);
}

impl<T> WeatheringCopperCollection<T> {
    pub fn new(weathering: ByState<T>, waxed: ByState<T>) -> WeatheringCopperCollection<T> {
        WeatheringCopperCollection { weathering, waxed }
    }

    pub fn weathering(&self) -> &ByState<T> {
        &self.weathering
    }

    pub fn waxed(&self) -> &ByState<T> {
        &self.waxed
    }

    /// The same for unwaxed and waxed.
    pub fn same(by_state: ByState<T>) -> WeatheringCopperCollection<T>
    where
        T: Clone,
    {
        WeatheringCopperCollection::new(by_state.clone(), by_state)
    }

    pub fn map<U>(&self, mut mapper: impl FnMut(&T) -> U) -> WeatheringCopperCollection<U> {
        WeatheringCopperCollection::new(
            self.weathering.map(&mut mapper),
            self.waxed.map(&mut mapper),
        )
    }

    /// The unwaxed things first, then the waxed ones.
    pub fn as_list(&self) -> Vec<&T> {
        self.weathering.0.iter().chain(&self.waxed.0).collect()
    }

    pub fn zip_map<U, R>(
        first: &WeatheringCopperCollection<T>,
        second: &WeatheringCopperCollection<U>,
        mut operation: impl FnMut(&T, &U) -> R,
    ) -> WeatheringCopperCollection<R> {
        WeatheringCopperCollection::new(
            ByState::zip_map(&first.weathering, &second.weathering, &mut operation),
            ByState::zip_map(&first.waxed, &second.waxed, &mut operation),
        )
    }
}

impl WeatheringCopperCollection<String> {
    /// `PREFIXES`: what each state's name starts with.
    pub fn prefixes() -> WeatheringCopperCollection<String> {
        WeatheringCopperCollection::new(
            ByState::new("", "exposed_", "weathered_", "oxidized_")
                .map(|prefix| prefix.to_string()),
            ByState::new(
                "waxed_",
                "waxed_exposed_",
                "waxed_weathered_",
                "waxed_oxidized_",
            )
            .map(|prefix| prefix.to_string()),
        )
    }

    /// `prefixWithState`: `exposed_cut_copper` from `cut_copper`.
    pub fn prefix_with_state(
        ids: &WeatheringCopperCollection<String>,
    ) -> WeatheringCopperCollection<String> {
        WeatheringCopperCollection::zip_map(
            &WeatheringCopperCollection::prefixes(),
            ids,
            |state, id| format!("{state}{id}"),
        )
    }

    /// `create`: the same name for every state, before prefixing.
    pub fn create(name: &str) -> WeatheringCopperCollection<String> {
        WeatheringCopperCollection::same(ByState::create(name.to_owned()))
    }
}

impl WeatheringCopperCollection<Block> {
    /// How many blocks a field of `Blocks` holding one takes.
    pub(crate) const COUNT: u16 = 8;

    /// The eight blocks registered one after another from `first`, unwaxed first, as a field of
    /// `Blocks` names them.
    pub(crate) const fn starting_at(first: u16) -> WeatheringCopperCollection<Block> {
        WeatheringCopperCollection {
            weathering: ByState([
                Block(first),
                Block(first + 1),
                Block(first + 2),
                Block(first + 3),
            ]),
            waxed: ByState([
                Block(first + 4),
                Block(first + 5),
                Block(first + 6),
                Block(first + 7),
            ]),
        }
    }

    /// The unwaxed blocks, then the waxed ones, each registered in the states' order under its
    /// id.
    pub fn register_blocks<Id: BlockKey>(
        registry: &Registrar,
        ids: &WeatheringCopperCollection<Id>,
        waxed_block_factory: impl Fn(WeatherState, Properties) -> BlockData,
        weathering_factory: impl Fn(WeatherState, Properties) -> BlockData,
        properties_supplier: impl Fn(WeatherState) -> Properties,
    ) -> WeatheringCopperCollection<Block> {
        let weathering = ByState::zip_map(&ByState::STATES, &ids.weathering, |&state, id| {
            blocks::register_with(
                registry,
                id,
                |properties| weathering_factory(state, properties),
                properties_supplier(state),
            )
        });
        let waxed = ByState::zip_map(&ByState::STATES, &ids.waxed, |&state, id| {
            blocks::register_with(
                registry,
                id,
                |properties| waxed_block_factory(state, properties),
                properties_supplier(state),
            )
        });
        WeatheringCopperCollection::new(weathering, waxed)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::WeatheringCopperCollection;
    use crate::golden::Encode;

    /// `[weathering, waxed]`, each state's in order.
    impl<T: Encode> Encode for WeatheringCopperCollection<T> {
        fn encode(&self) -> Value {
            let by_state = |things: &[T; 4]| {
                Value::from(things.iter().map(Encode::encode).collect::<Vec<_>>())
            };
            Value::from(vec![by_state(&self.weathering.0), by_state(&self.waxed.0)])
        }
    }
}
