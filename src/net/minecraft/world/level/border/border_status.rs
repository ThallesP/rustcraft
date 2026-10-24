//! net/minecraft/world/level/border/BorderStatus.java

/// Whether the border is moving, and which way: the colour it's drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderStatus {
    GROWING,
    SHRINKING,
    STATIONARY,
}

impl BorderStatus {
    pub fn get_color(self) -> i32 {
        match self {
            BorderStatus::GROWING => 4259712,
            BorderStatus::SHRINKING => 16724016,
            BorderStatus::STATIONARY => 2138367,
        }
    }
}
