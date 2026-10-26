//! net/minecraft/client/PrioritizeChunkUpdates.java
//!
//! Which dirty sections compile on the render thread before the frame goes on, rather than on
//! the worker threads: none, those the player changed, or those near the camera too. The
//! caption and the options screen's codec aren't ported.

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrioritizeChunkUpdates {
    NONE,
    PLAYER_AFFECTED,
    NEARBY,
}

impl PrioritizeChunkUpdates {
    pub fn id(self) -> i32 {
        self as i32
    }
}
