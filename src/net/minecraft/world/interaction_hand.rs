//! net/minecraft/world/InteractionHand.java
//!
//! Not ported yet: `STREAM_CODEC` and `asEquipmentSlot`, which come with the packets and the
//! equipment that use them.

use crate::net::minecraft::world::entity::humanoid_arm::HumanoidArm;

/// A hand an entity uses things with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InteractionHand {
    MAIN_HAND,
    OFF_HAND,
}

impl InteractionHand {
    pub fn id(&self) -> i32 {
        match self {
            InteractionHand::MAIN_HAND => 0,
            InteractionHand::OFF_HAND => 1,
        }
    }

    /// The arm the hand is, for the entity's main arm.
    pub fn as_arm(&self, main_arm: HumanoidArm) -> HumanoidArm {
        match self {
            InteractionHand::MAIN_HAND => main_arm,
            InteractionHand::OFF_HAND => main_arm.get_opposite(),
        }
    }
}
