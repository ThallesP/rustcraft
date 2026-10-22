//! net/minecraft/world/level/chunk/Palette.java
//!
//! `Palette.Factory`, a functional interface `Strategy` makes three constants of
//! (`SingleValuePalette::create`, `LinearPalette::create`, `HashMapPalette::create`), is an enum
//! of those three, so configurations holding one compare like Java's records holding the same
//! constant. `AnyPalette` is one of the four palettes: a container holds its palette as one, which
//! dispatches with a `match` where Java calls through the interface.

use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::world::level::chunk::global_palette::GlobalPalette;
use crate::net::minecraft::world::level::chunk::hash_map_palette::HashMapPalette;
use crate::net::minecraft::world::level::chunk::linear_palette::LinearPalette;
use crate::net::minecraft::world::level::chunk::palette_resize::PaletteResize;
use crate::net::minecraft::world::level::chunk::single_value_palette::SingleValuePalette;

/// The values a container holds, numbered: its storage holds their ids.
pub trait Palette<T> {
    /// The value's id, added if it's new. A full palette returns what `resize_handler` gives
    /// for the next bits up.
    fn id_for(&mut self, value: &T, resize_handler: &mut dyn PaletteResize<T>) -> i32;

    fn maybe_has(&self, predicate: &mut dyn FnMut(&T) -> bool) -> bool;

    fn value_for(&self, index: i32) -> T;

    fn read(&mut self, buffer: &mut FriendlyByteBuf, global_map: &dyn IdMap<T>);

    fn write(&self, buffer: &mut FriendlyByteBuf, global_map: &dyn IdMap<T>);

    fn get_serialized_size(&self, global_map: &dyn IdMap<T>) -> i32;

    fn get_size(&self) -> i32;
}

/// `Palette.Factory`: which palette a configuration makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Factory {
    SingleValue,
    Linear,
    HashMap,
}

impl Factory {
    pub fn create<T: Clone + PartialEq + std::hash::Hash>(
        &self,
        bits: i32,
        palette_entries: &[T],
    ) -> AnyPalette<T> {
        match self {
            Factory::SingleValue => {
                AnyPalette::SingleValue(SingleValuePalette::create(bits, palette_entries))
            }
            Factory::Linear => AnyPalette::Linear(LinearPalette::create(bits, palette_entries)),
            Factory::HashMap => AnyPalette::HashMap(HashMapPalette::create(bits, palette_entries)),
        }
    }
}

/// One of the palettes.
#[derive(Clone)]
pub enum AnyPalette<T> {
    SingleValue(SingleValuePalette<T>),
    Linear(LinearPalette<T>),
    HashMap(HashMapPalette<T>),
    Global(GlobalPalette<T>),
}

macro_rules! dispatch {
    ($self:expr, $palette:ident => $call:expr) => {
        match $self {
            AnyPalette::SingleValue($palette) => $call,
            AnyPalette::Linear($palette) => $call,
            AnyPalette::HashMap($palette) => $call,
            AnyPalette::Global($palette) => $call,
        }
    };
}

impl<T: Clone + PartialEq + std::hash::Hash> Palette<T> for AnyPalette<T> {
    fn id_for(&mut self, value: &T, resize_handler: &mut dyn PaletteResize<T>) -> i32 {
        dispatch!(self, palette => palette.id_for(value, resize_handler))
    }

    fn maybe_has(&self, predicate: &mut dyn FnMut(&T) -> bool) -> bool {
        dispatch!(self, palette => palette.maybe_has(predicate))
    }

    #[inline]
    fn value_for(&self, index: i32) -> T {
        dispatch!(self, palette => palette.value_for(index))
    }

    fn read(&mut self, buffer: &mut FriendlyByteBuf, global_map: &dyn IdMap<T>) {
        dispatch!(self, palette => palette.read(buffer, global_map))
    }

    fn write(&self, buffer: &mut FriendlyByteBuf, global_map: &dyn IdMap<T>) {
        dispatch!(self, palette => palette.write(buffer, global_map))
    }

    fn get_serialized_size(&self, global_map: &dyn IdMap<T>) -> i32 {
        dispatch!(self, palette => palette.get_serialized_size(global_map))
    }

    fn get_size(&self) -> i32 {
        dispatch!(self, palette => palette.get_size())
    }
}

impl<T: Clone + PartialEq + std::hash::Hash> AnyPalette<T> {
    /// `copy()`: a single value palette panics when it has no value yet, where Java throws
    /// `IllegalStateException`.
    pub fn copy(&self) -> AnyPalette<T> {
        match self {
            AnyPalette::SingleValue(palette) => AnyPalette::SingleValue(palette.copy()),
            other => other.clone(),
        }
    }
}
