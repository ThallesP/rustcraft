//! net/minecraft/world/level/block/state/properties/SpeleothemThickness.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// How thick a part of a dripstone or sulfur spike is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpeleothemThickness {
    TIP_MERGE,
    TIP,
    FRUSTUM,
    MIDDLE,
    BASE,
}

impl SpeleothemThickness {
    pub const VALUES: [SpeleothemThickness; 5] = [
        SpeleothemThickness::TIP_MERGE,
        SpeleothemThickness::TIP,
        SpeleothemThickness::FRUSTUM,
        SpeleothemThickness::MIDDLE,
        SpeleothemThickness::BASE,
    ];

    pub fn values() -> [SpeleothemThickness; 5] {
        SpeleothemThickness::VALUES
    }
}

impl Enum for SpeleothemThickness {
    const ENUM_CONSTANTS: &'static [SpeleothemThickness] = &SpeleothemThickness::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for SpeleothemThickness {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            SpeleothemThickness::TIP_MERGE => "tip_merge",
            SpeleothemThickness::TIP => "tip",
            SpeleothemThickness::FRUSTUM => "frustum",
            SpeleothemThickness::MIDDLE => "middle",
            SpeleothemThickness::BASE => "base",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for SpeleothemThickness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for SpeleothemThickness {
    const CLASS: &'static str =
        "net.minecraft.world.level.block.state.properties.SpeleothemThickness";
}
