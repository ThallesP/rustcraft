//! net/minecraft/core/component/DataComponentType.java
//!
//! A kind of item component, by its id in `BuiltInRegistries.DATA_COMPONENT_TYPE` (`DataComponents`'
//! table): Java's types are objects the game compares by reference, so here a type is its id,
//! `Copy`, like an `EntityType`. Not ported yet: each type's codec and stream codec (their value
//! classes), `CODEC`, `PERSISTENT_CODEC` and `STREAM_CODEC`; a value is kept as the data it was read
//! from (`DataComponentPatch`).

use std::fmt;

use crate::net::minecraft::core::component::data_components;
use crate::net::minecraft::resources::identifier::Identifier;

/// An item component's kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataComponentType(u16);

impl DataComponentType {
    /// The type registered under the identifier, if there's one.
    pub fn by_name(id: &Identifier) -> Option<DataComponentType> {
        if id.get_namespace() != "minecraft" {
            return None;
        }
        data_components::TYPES
            .iter()
            .position(|(name, _)| *name == id.get_path())
            .map(|index| DataComponentType(index as u16))
    }

    pub fn id(&self) -> i32 {
        i32::from(self.0)
    }

    pub fn key(&self) -> Identifier {
        Identifier::with_default_namespace(data_components::TYPES[usize::from(self.0)].0)
    }

    /// Never saved: no codec.
    pub fn is_transient(&self) -> bool {
        data_components::TYPES[usize::from(self.0)].1
    }
}

/// `toString`: its registered name (`Util.getRegisteredName`).
impl fmt::Display for DataComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key())
    }
}

impl fmt::Debug for DataComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
