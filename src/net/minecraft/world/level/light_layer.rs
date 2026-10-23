//! net/minecraft/world/level/LightLayer.java

/// Which of a block's two lights: from the sky, or from blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LightLayer {
    SKY,
    BLOCK,
}
