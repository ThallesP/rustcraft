//! net/minecraft/world/level/block/state/properties/SideChainPart.java
//!
//! Not ported yet: `isConnected`, `isConnectionTowards`, `isChainEnd`, `whenConnectedToTheRight`, `whenConnectedToTheLeft`, `whenDisconnectedFromTheRight`, `whenDisconnectedFromTheLeft`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Where a shelf is in a row of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SideChainPart {
    UNCONNECTED,
    RIGHT,
    CENTER,
    LEFT,
}

impl SideChainPart {
    pub const VALUES: [SideChainPart; 4] = [
        SideChainPart::UNCONNECTED,
        SideChainPart::RIGHT,
        SideChainPart::CENTER,
        SideChainPart::LEFT,
    ];

    pub fn values() -> [SideChainPart; 4] {
        SideChainPart::VALUES
    }
}

impl Enum for SideChainPart {
    const ENUM_CONSTANTS: &'static [SideChainPart] = &SideChainPart::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for SideChainPart {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            SideChainPart::UNCONNECTED => "unconnected",
            SideChainPart::RIGHT => "right",
            SideChainPart::CENTER => "center",
            SideChainPart::LEFT => "left",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for SideChainPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for SideChainPart {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.SideChainPart";
}
