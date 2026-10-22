//! net/minecraft/world/level/chunk/Configuration.java
//!
//! An interface with two records, which is an enum of them: equal configurations are the same
//! record with the same components, which is what `PalettedContainer` asks to reuse its data.

use std::hash::Hash;

use crate::net::minecraft::world::level::chunk::palette::{AnyPalette, Factory};
use crate::net::minecraft::world::level::chunk::strategy::Strategy;

/// A palette's kind and its storage's bits per entry, in memory and as written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Configuration {
    /// `Configuration.Global`: the registry's ids, at the bits the registry needs in memory
    /// whatever a buffer said.
    Global {
        bits_in_memory: i32,
        bits_in_storage: i32,
    },
    /// `Configuration.Simple`: a palette of its own, at its bits.
    Simple { factory: Factory, bits: i32 },
}

impl Configuration {
    pub fn always_repack(&self) -> bool {
        matches!(self, Configuration::Global { .. })
    }

    pub fn bits_in_memory(&self) -> i32 {
        match *self {
            Configuration::Global { bits_in_memory, .. } => bits_in_memory,
            Configuration::Simple { bits, .. } => bits,
        }
    }

    pub fn bits_in_storage(&self) -> i32 {
        match *self {
            Configuration::Global {
                bits_in_storage, ..
            } => bits_in_storage,
            Configuration::Simple { bits, .. } => bits,
        }
    }

    pub fn create_palette<T: Clone + PartialEq + Hash>(
        &self,
        strategy: &Strategy<T>,
        palette_entries: &[T],
    ) -> AnyPalette<T> {
        match *self {
            Configuration::Global { .. } => AnyPalette::Global(strategy.global_palette().clone()),
            Configuration::Simple { factory, bits } => factory.create(bits, palette_entries),
        }
    }
}
