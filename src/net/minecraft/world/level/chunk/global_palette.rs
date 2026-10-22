//! net/minecraft/world/level/chunk/GlobalPalette.java
//!
//! The registry is shared, like Java's reference to it, and so is the palette (`copy()` hands
//! back itself): a clone shares the registry.

use std::sync::Arc;

use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::world::level::chunk::missing_palette_entry_exception::MissingPaletteEntryException;
use crate::net::minecraft::world::level::chunk::palette::Palette;
use crate::net::minecraft::world::level::chunk::palette_resize::PaletteResize;

/// The registry's own ids, which the storage holds as they are.
pub struct GlobalPalette<T> {
    registry: Arc<dyn IdMap<T> + Send + Sync>,
}

impl<T> GlobalPalette<T> {
    pub fn new(registry: Arc<dyn IdMap<T> + Send + Sync>) -> GlobalPalette<T> {
        GlobalPalette { registry }
    }
}

impl<T> Clone for GlobalPalette<T> {
    fn clone(&self) -> GlobalPalette<T> {
        GlobalPalette {
            registry: Arc::clone(&self.registry),
        }
    }
}

impl<T> Palette<T> for GlobalPalette<T> {
    /// A value the registry doesn't have is 0.
    fn id_for(&mut self, value: &T, _resize_handler: &mut dyn PaletteResize<T>) -> i32 {
        let id = self.registry.get_id(value);
        if id == -1 { 0 } else { id }
    }

    fn maybe_has(&self, _predicate: &mut dyn FnMut(&T) -> bool) -> bool {
        true
    }

    #[inline]
    fn value_for(&self, index: i32) -> T {
        match self.registry.by_id(index) {
            Some(value) => value,
            None => MissingPaletteEntryException::throw(index),
        }
    }

    fn read(&mut self, _buffer: &mut FriendlyByteBuf, _global_map: &dyn IdMap<T>) {}

    fn write(&self, _buffer: &mut FriendlyByteBuf, _global_map: &dyn IdMap<T>) {}

    fn get_serialized_size(&self, _global_map: &dyn IdMap<T>) -> i32 {
        0
    }

    fn get_size(&self) -> i32 {
        self.registry.size()
    }
}
