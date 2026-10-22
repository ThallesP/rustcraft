//! net/minecraft/world/level/material/Fluids.java
//!
//! Only the fluids, by which a liquid block knows its own, and what `Fluid.isRandomlyTicking` says
//! of each: only lava ticks (`LavaFluid`'s override). Not ported yet: `Fluid`, `FlowingFluid`,
//! `WaterFluid`, `LavaFluid` and `EmptyFluid` themselves, their states and
//! `Fluid.FLUID_STATE_REGISTRY`, which come with fluids.

/// One of the registered fluids, in the order `Fluids` registers them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Fluids {
    EMPTY,
    FLOWING_WATER,
    WATER,
    FLOWING_LAVA,
    LAVA,
}

impl Fluids {
    /// `Fluid.isRandomlyTicking()`: false, but `LavaFluid`'s, flowing or still, is true.
    pub fn is_randomly_ticking(&self) -> bool {
        matches!(self, Fluids::FLOWING_LAVA | Fluids::LAVA)
    }
}
