//! net/minecraft/world/level/block/entity/BeaconBeamOwner.java
//!
//! Only its `Section`: the interface's `getBeamSections` is the beacon's own.

/// `BeaconBeamOwner.Section`: a stretch of a beam of one colour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    color: i32,
    height: i32,
}

impl Section {
    pub fn new(color: i32) -> Section {
        Section { color, height: 1 }
    }

    pub fn increase_height(&mut self) {
        self.height += 1;
    }

    pub fn get_color(&self) -> i32 {
        self.color
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }
}
