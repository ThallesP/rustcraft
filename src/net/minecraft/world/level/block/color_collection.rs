//! net/minecraft/world/level/block/ColorCollection.java
//!
//! Not ported yet: `registerBlockItems` and `registerItems`, which come with items, and `zipApply`.
//! `registerBlocks` takes the registry `Blocks` registers into, and always registers with
//! `Blocks.register`, the one function vanilla passes it.
//!
//! The record's sixteen components are an array in their order, which is `DyeColor`'s.

use crate::net::minecraft::world::item::dye_color::DyeColor;
use crate::net::minecraft::world::level::block::block::{Block, BlockData};
use crate::net::minecraft::world::level::block::blocks::{self, BlockKey, Registrar};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

/// One of a thing for each dye colour, like the sixteen wools.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ColorCollection<T>([T; 16]);

impl<T> ColorCollection<T> {
    pub fn new(values: [T; 16]) -> ColorCollection<T> {
        ColorCollection(values)
    }

    /// The same value for every colour.
    pub fn create(value: T) -> ColorCollection<T>
    where
        T: Clone,
    {
        ColorCollection(std::array::from_fn(|_| value.clone()))
    }

    /// Each colour's, in the colours' order.
    pub fn as_list(&self) -> &[T; 16] {
        &self.0
    }

    pub fn pick(&self, dye_color: DyeColor) -> &T {
        &self.0[dye_color as usize]
    }

    pub fn map<U>(&self, mapper: impl FnMut(&T) -> U) -> ColorCollection<U> {
        ColorCollection(self.0.each_ref().map(mapper))
    }

    /// Each colour's from the two, combined in the colours' order.
    pub fn zip_map<U, R>(
        first: &ColorCollection<T>,
        second: &ColorCollection<U>,
        mut operation: impl FnMut(&T, &U) -> R,
    ) -> ColorCollection<R> {
        let mut index = 0;
        ColorCollection(std::array::from_fn(|_| {
            let result = operation(&first.0[index], &second.0[index]);
            index += 1;
            result
        }))
    }
}

impl ColorCollection<DyeColor> {
    /// `VALUES`: each colour itself.
    pub const VALUES: ColorCollection<DyeColor> = ColorCollection(DyeColor::VALUES);
}

impl ColorCollection<String> {
    /// `prefixWithColor`: `white_bed` from `bed`.
    pub fn prefix_with_color(ids: &ColorCollection<String>) -> ColorCollection<String> {
        ColorCollection::zip_map(&ColorCollection::VALUES, ids, |color, id| {
            format!("{}_{id}", color.get_name())
        })
    }
}

impl ColorCollection<Block> {
    /// How many blocks a field of `Blocks` holding one takes.
    pub(crate) const COUNT: u16 = 16;

    /// The sixteen blocks registered one after another from `first`, as a field of `Blocks`
    /// names them.
    pub(crate) const fn starting_at(first: u16) -> ColorCollection<Block> {
        let mut blocks = [Block(0); 16];
        let mut i = 0;
        while i < 16 {
            blocks[i] = Block(first + i as u16);
            i += 1;
        }
        ColorCollection(blocks)
    }

    /// A block of each colour, registered in the colours' order under its id.
    pub fn register_blocks<Id: BlockKey>(
        registry: &Registrar,
        ids: &ColorCollection<Id>,
        color_block_factory: impl Fn(DyeColor, Properties) -> BlockData,
        properties_supplier: impl Fn(DyeColor) -> Properties,
    ) -> ColorCollection<Block> {
        ColorCollection::zip_map(&ColorCollection::VALUES, ids, |&color, id| {
            blocks::register_with(
                registry,
                id,
                |properties| color_block_factory(color, properties),
                properties_supplier(color),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ColorCollection;
    use crate::golden::Encode;

    /// Each colour's, in order, like `asList()`.
    impl<T: Encode> Encode for ColorCollection<T> {
        fn encode(&self) -> Value {
            Value::from(self.0.iter().map(Encode::encode).collect::<Vec<_>>())
        }
    }
}
