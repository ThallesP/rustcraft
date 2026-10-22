//! net/minecraft/world/level/material/PushReaction.java

/// What a piston does to a block in its way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PushReaction {
    PUSH_PULL,
    PUSH,
    POPPED,
    IMMOVEABLE,
    IGNORE_ENTITY,
}
