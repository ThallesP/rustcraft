//! net/minecraft/server/RunningOnDifferentThreadException.java
//!
//! Java throws its one instance to stop a handler that has queued its packet for the game thread,
//! and the connection catches it and moves on. Here the handler returns it, and returns early.

/// A handler's packet went to the game thread's queue instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunningOnDifferentThreadException;

impl RunningOnDifferentThreadException {
    pub const RUNNING_ON_DIFFERENT_THREAD: RunningOnDifferentThreadException =
        RunningOnDifferentThreadException;
}
