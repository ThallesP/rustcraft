//! net/minecraft/world/level/block/state/properties/TestBlockMode.java
//!
//! Not ported yet: `getDisplayName`, `getDetailedMessage`, `CODEC`, `STREAM_CODEC`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// What a test block does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TestBlockMode {
    START,
    LOG,
    FAIL,
    ACCEPT,
}

impl TestBlockMode {
    pub const VALUES: [TestBlockMode; 4] = [
        TestBlockMode::START,
        TestBlockMode::LOG,
        TestBlockMode::FAIL,
        TestBlockMode::ACCEPT,
    ];

    pub fn values() -> [TestBlockMode; 4] {
        TestBlockMode::VALUES
    }
}

impl Enum for TestBlockMode {
    const ENUM_CONSTANTS: &'static [TestBlockMode] = &TestBlockMode::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for TestBlockMode {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            TestBlockMode::START => "start",
            TestBlockMode::LOG => "log",
            TestBlockMode::FAIL => "fail",
            TestBlockMode::ACCEPT => "accept",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for TestBlockMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for TestBlockMode {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.TestBlockMode";
}
