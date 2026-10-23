//! net/minecraft/world/level/material/FogType.java

/// What the camera is in, which decides the fog.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FogType {
    LAVA,
    WATER,
    POWDER_SNOW,
    ATMOSPHERIC,
    NONE,
}
