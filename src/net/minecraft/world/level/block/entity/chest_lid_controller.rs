//! net/minecraft/world/level/block/entity/ChestLidController.java

use crate::java::lang::math::Math;
use crate::net::minecraft::util::mth::Mth;

/// A chest's lid: whether it should be open, and how far it is open now and was last tick.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ChestLidController {
    should_be_open: bool,
    openness: f32,
    o_openness: f32,
}

impl ChestLidController {
    pub fn new() -> ChestLidController {
        ChestLidController::default()
    }

    /// `tickLid`: a tenth further open or closed, towards where it should be.
    pub fn tick_lid(&mut self) {
        self.o_openness = self.openness;
        if !self.should_be_open && self.openness > 0.0 {
            self.openness = Math::max_f32(self.openness - 0.1, 0.0);
        } else if self.should_be_open && self.openness < 1.0 {
            self.openness = Math::min_f32(self.openness + 0.1, 1.0);
        }
    }

    pub fn get_openness(&self, a: f32) -> f32 {
        Mth::lerp_f32(a, self.o_openness, self.openness)
    }

    pub fn should_be_open(&mut self, should_be_open: bool) {
        self.should_be_open = should_be_open;
    }

    /// Whether it should be open, which Java's suite reads by reflection.
    pub fn is_should_be_open(&self) -> bool {
        self.should_be_open
    }
}
