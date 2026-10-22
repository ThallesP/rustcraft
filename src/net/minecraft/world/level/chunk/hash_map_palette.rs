//! net/minecraft/world/level/chunk/HashMapPalette.java
//!
//! The constructor taking only bits is `with_bits`, next to `new(bits, values)`.

use std::hash::Hash;

use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::var_int::VarInt;
use crate::net::minecraft::util::crude_incremental_int_identity_hash_bi_map::CrudeIncrementalIntIdentityHashBiMap;
use crate::net::minecraft::world::level::chunk::missing_palette_entry_exception::MissingPaletteEntryException;
use crate::net::minecraft::world::level::chunk::palette::Palette;
use crate::net::minecraft::world::level::chunk::palette_resize::PaletteResize;

/// Up to `2^bits` values, found by hash.
#[derive(Clone, Debug)]
pub struct HashMapPalette<T> {
    values: CrudeIncrementalIntIdentityHashBiMap<T>,
    bits: i32,
}

impl<T: Clone + PartialEq + Hash> HashMapPalette<T> {
    pub fn new(bits: i32, values: &[T]) -> HashMapPalette<T> {
        let mut palette = HashMapPalette::with_bits(bits);
        for value in values {
            palette.values.add(value.clone());
        }
        palette
    }

    pub fn with_bits(bits: i32) -> HashMapPalette<T> {
        HashMapPalette {
            values: CrudeIncrementalIntIdentityHashBiMap::create(1i32.wrapping_shl(bits as u32)),
            bits,
        }
    }

    pub fn create(bits: i32, palette_entries: &[T]) -> HashMapPalette<T> {
        HashMapPalette::new(bits, palette_entries)
    }

    pub fn get_entries(&self) -> Vec<T> {
        self.values.iter().cloned().collect()
    }

    fn entry(&self, index: i32) -> T {
        self.values
            .by_id(index)
            .unwrap_or_else(|| panic!("no value at {index}, which Java hands out as null"))
    }
}

impl<T: Clone + PartialEq + Hash> Palette<T> for HashMapPalette<T> {
    /// A new value is added first, so a full palette holds it too when it asks for a resize.
    fn id_for(&mut self, value: &T, resize_handler: &mut dyn PaletteResize<T>) -> i32 {
        let mut id = self.values.get_id(value);
        if id == -1 {
            id = self.values.add(value.clone());
            if id >= 1i32.wrapping_shl(self.bits as u32) {
                id = resize_handler.on_resize(self.bits + 1, value);
            }
        }

        id
    }

    fn maybe_has(&self, predicate: &mut dyn FnMut(&T) -> bool) -> bool {
        (0..self.get_size()).any(|i| predicate(&self.entry(i)))
    }

    #[inline]
    fn value_for(&self, index: i32) -> T {
        match self.values.by_id(index) {
            Some(value) => value,
            None => MissingPaletteEntryException::throw(index),
        }
    }

    fn read(&mut self, buffer: &mut FriendlyByteBuf, global_map: &dyn IdMap<T>) {
        self.values.clear();
        let size = buffer.read_var_int();

        for _ in 0..size {
            self.values
                .add(global_map.by_id_or_throw(buffer.read_var_int()));
        }
    }

    fn write(&self, buffer: &mut FriendlyByteBuf, global_map: &dyn IdMap<T>) {
        let size = self.get_size();
        buffer.write_var_int(size);

        for i in 0..size {
            let id = global_map.get_id(&self.entry(i));
            buffer.write_var_int(id);
        }
    }

    fn get_serialized_size(&self, global_map: &dyn IdMap<T>) -> i32 {
        let mut size = VarInt::get_byte_size(self.get_size());

        for i in 0..self.get_size() {
            size += VarInt::get_byte_size(global_map.get_id(&self.entry(i)));
        }

        size
    }

    fn get_size(&self) -> i32 {
        self.values.size()
    }
}
