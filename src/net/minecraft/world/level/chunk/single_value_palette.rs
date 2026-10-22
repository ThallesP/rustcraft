//! net/minecraft/world/level/chunk/SingleValuePalette.java
//!
//! Java's `copy()` hands back the palette itself, so a container and its copy share it, and a
//! `read` into one (which sets the value in place) shows in the other. Nothing in the client reads
//! into a container that has been copied: `SectionCopy` copies a section's states for the
//! renderer and only reads them, and a chunk the server sends again is read into its sections.
//! The port copies the palette, which differs only after that.

use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::var_int::VarInt;
use crate::net::minecraft::world::level::chunk::palette::Palette;
use crate::net::minecraft::world::level::chunk::palette_resize::PaletteResize;

/// A palette of one value, which a storage of zero bits points every entry at.
#[derive(Clone, Debug)]
pub struct SingleValuePalette<T> {
    value: Option<T>,
}

impl<T: Clone + PartialEq> SingleValuePalette<T> {
    /// Panics on more than one entry, where Java throws `IllegalArgumentException`.
    pub fn new(palette_entries: &[T]) -> SingleValuePalette<T> {
        let mut value = None;
        if !palette_entries.is_empty() {
            if palette_entries.len() > 1 {
                panic!(
                    "Can't initialize SingleValuePalette with {} values.",
                    palette_entries.len()
                );
            }
            value = Some(palette_entries[0].clone());
        }
        SingleValuePalette { value }
    }

    pub fn create(_bits: i32, palette_entries: &[T]) -> SingleValuePalette<T> {
        SingleValuePalette::new(palette_entries)
    }

    fn value(&self) -> &T {
        self.value
            .as_ref()
            .unwrap_or_else(|| panic!("Use of an uninitialized palette"))
    }

    /// Panics when it has no value yet, where Java throws `IllegalStateException`.
    pub fn copy(&self) -> SingleValuePalette<T> {
        self.value();
        self.clone()
    }
}

impl<T: Clone + PartialEq> Palette<T> for SingleValuePalette<T> {
    fn id_for(&mut self, value: &T, resize_handler: &mut dyn PaletteResize<T>) -> i32 {
        if self.value.as_ref().is_some_and(|own| own != value) {
            return resize_handler.on_resize(1, value);
        }

        self.value = Some(value.clone());
        0
    }

    fn maybe_has(&self, predicate: &mut dyn FnMut(&T) -> bool) -> bool {
        predicate(self.value())
    }

    #[inline]
    fn value_for(&self, index: i32) -> T {
        match &self.value {
            Some(value) if index == 0 => value.clone(),
            _ => panic!("Missing Palette entry for id {index}."),
        }
    }

    fn read(&mut self, buffer: &mut FriendlyByteBuf, global_map: &dyn IdMap<T>) {
        self.value = Some(global_map.by_id_or_throw(buffer.read_var_int()));
    }

    fn write(&self, buffer: &mut FriendlyByteBuf, global_map: &dyn IdMap<T>) {
        let id = global_map.get_id(self.value());
        buffer.write_var_int(id);
    }

    fn get_serialized_size(&self, global_map: &dyn IdMap<T>) -> i32 {
        VarInt::get_byte_size(global_map.get_id(self.value()))
    }

    fn get_size(&self) -> i32 {
        1
    }
}
