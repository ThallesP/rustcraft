//! net/minecraft/world/level/chunk/LinearPalette.java
//!
//! Java's array holds null past the values it has. One it hands out from there (only after a
//! `read` that threw halfway, which drops the chunk) panics here.

use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::var_int::VarInt;
use crate::net::minecraft::world::level::chunk::missing_palette_entry_exception::MissingPaletteEntryException;
use crate::net::minecraft::world::level::chunk::palette::Palette;
use crate::net::minecraft::world::level::chunk::palette_resize::PaletteResize;

/// Up to `2^bits` values, found by a linear search.
#[derive(Clone, Debug)]
pub struct LinearPalette<T> {
    values: Vec<Option<T>>,
    bits: i32,
    size: i32,
}

impl<T: Clone + PartialEq> LinearPalette<T> {
    /// Panics on more entries than it holds, where Java throws `IllegalArgumentException`.
    fn new(bits: i32, palette_entries: &[T]) -> LinearPalette<T> {
        let length = 1i32.wrapping_shl(bits as u32);
        assert!(length >= 0, "{length}");
        let mut values = vec![None; length as usize];
        if palette_entries.len() > values.len() {
            panic!(
                "Can't initialize LinearPalette of size {} with {} entries",
                values.len(),
                palette_entries.len()
            );
        }

        for (slot, entry) in values.iter_mut().zip(palette_entries) {
            *slot = Some(entry.clone());
        }

        LinearPalette {
            values,
            bits,
            size: palette_entries.len() as i32,
        }
    }

    pub fn create(bits: i32, palette_entries: &[T]) -> LinearPalette<T> {
        LinearPalette::new(bits, palette_entries)
    }

    // Java's `values[index]`, which is null past the values set.
    fn entry(&self, index: i32) -> &T {
        self.values[index as usize]
            .as_ref()
            .unwrap_or_else(|| panic!("no value at {index}, which Java hands out as null"))
    }
}

impl<T: Clone + PartialEq> Palette<T> for LinearPalette<T> {
    fn id_for(&mut self, value: &T, resize_handler: &mut dyn PaletteResize<T>) -> i32 {
        for i in 0..self.size {
            if self.values[i as usize].as_ref() == Some(value) {
                return i;
            }
        }

        let index = self.size;
        if (index as i64) < self.values.len() as i64 {
            self.values[index as usize] = Some(value.clone());
            self.size += 1;
            index
        } else {
            resize_handler.on_resize(self.bits + 1, value)
        }
    }

    fn maybe_has(&self, predicate: &mut dyn FnMut(&T) -> bool) -> bool {
        (0..self.size).any(|i| predicate(self.entry(i)))
    }

    #[inline]
    fn value_for(&self, index: i32) -> T {
        if index >= 0 && index < self.size {
            self.entry(index).clone()
        } else {
            MissingPaletteEntryException::throw(index)
        }
    }

    /// Reads the size first, so one past what it holds panics once the values fill it, where
    /// Java throws `ArrayIndexOutOfBoundsException`, after reading that id.
    fn read(&mut self, buffer: &mut FriendlyByteBuf, global_map: &dyn IdMap<T>) {
        self.size = buffer.read_var_int();

        for i in 0..self.size {
            let value = global_map.by_id_or_throw(buffer.read_var_int());
            self.values[i as usize] = Some(value);
        }
    }

    fn write(&self, buffer: &mut FriendlyByteBuf, global_map: &dyn IdMap<T>) {
        buffer.write_var_int(self.size);

        for i in 0..self.size {
            let id = global_map.get_id(self.entry(i));
            buffer.write_var_int(id);
        }
    }

    fn get_serialized_size(&self, global_map: &dyn IdMap<T>) -> i32 {
        let mut result = VarInt::get_byte_size(self.get_size());

        for i in 0..self.get_size() {
            result += VarInt::get_byte_size(global_map.get_id(self.entry(i)));
        }

        result
    }

    fn get_size(&self) -> i32 {
        self.size
    }
}
