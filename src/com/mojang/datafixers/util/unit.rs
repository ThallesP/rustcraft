//! com/mojang/datafixers/util/Unit.java (DataFixerUpper 10.0.21)
//!
//! The one value of a type with nothing in it, which `Codec.EMPTY` reads and writes.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Unit {
    INSTANCE,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Unit")
    }
}
