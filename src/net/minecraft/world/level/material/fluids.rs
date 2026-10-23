//! net/minecraft/world/level/material/Fluids.java
//!
//! The fluids, in the order `Fluids` registers them, with what their classes say of their states
//! (`EmptyFluid`, `WaterFluid` and `LavaFluid`, each `Source` and `Flowing`): which tick randomly
//! (only lava, `LavaFluid`'s override), which are sources, and the states `FlowingFluid` makes.
//! Not ported yet: `Fluid`'s behaviour (ticking, spreading, heights, shapes) and
//! `Fluid.FLUID_STATE_REGISTRY`, which come with what calls them.

use crate::net::minecraft::world::level::material::fluid_state::FluidState;

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
    /// Its name in `BuiltInRegistries.FLUID`.
    pub fn key(&self) -> &'static str {
        match self {
            Fluids::EMPTY => "minecraft:empty",
            Fluids::FLOWING_WATER => "minecraft:flowing_water",
            Fluids::WATER => "minecraft:water",
            Fluids::FLOWING_LAVA => "minecraft:flowing_lava",
            Fluids::LAVA => "minecraft:lava",
        }
    }

    /// `Fluid.isRandomlyTicking()`: false, but `LavaFluid`'s, flowing or still, is true.
    pub fn is_randomly_ticking(&self) -> bool {
        matches!(self, Fluids::FLOWING_LAVA | Fluids::LAVA)
    }

    /// `Fluid.isEmpty()`: only `EmptyFluid`'s is true.
    pub fn is_empty(&self) -> bool {
        *self == Fluids::EMPTY
    }

    /// Whether it's a `FlowingFluid`'s `Flowing`, whose states have a level.
    pub fn is_flowing(&self) -> bool {
        matches!(self, Fluids::FLOWING_WATER | Fluids::FLOWING_LAVA)
    }

    /// `FlowingFluid.getSource()`.
    pub fn get_source_fluid(&self) -> Fluids {
        match self {
            Fluids::FLOWING_WATER | Fluids::WATER => Fluids::WATER,
            Fluids::FLOWING_LAVA | Fluids::LAVA => Fluids::LAVA,
            Fluids::EMPTY => panic!("the empty fluid isn't a FlowingFluid"),
        }
    }

    /// `FlowingFluid.getFlowing()`.
    pub fn get_flowing_fluid(&self) -> Fluids {
        match self {
            Fluids::FLOWING_WATER | Fluids::WATER => Fluids::FLOWING_WATER,
            Fluids::FLOWING_LAVA | Fluids::LAVA => Fluids::FLOWING_LAVA,
            Fluids::EMPTY => panic!("the empty fluid isn't a FlowingFluid"),
        }
    }

    /// `defaultFluidState()`: its definition's first state, not falling and (flowing) at level 1.
    pub fn default_fluid_state(&self) -> FluidState {
        FluidState::new(*self, false, if self.is_flowing() { 1 } else { 0 })
    }

    /// `FlowingFluid.getSource(falling)`.
    pub fn get_source(&self, falling: bool) -> FluidState {
        FluidState::new(self.get_source_fluid(), falling, 0)
    }

    /// `FlowingFluid.getFlowing(amount, falling)`: panics on an amount outside 1 to 8, where
    /// Java's `setValue` throws.
    pub fn get_flowing(&self, amount: i32, falling: bool) -> FluidState {
        assert!(
            (1..=8).contains(&amount),
            "Cannot set property level to {amount}"
        );
        FluidState::new(self.get_flowing_fluid(), falling, amount)
    }
}
