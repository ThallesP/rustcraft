//! net/minecraft/world/entity/Avatar.java
//!
//! Only the sizes of a player-shaped entity's poses (`POSES`, `getDefaultDimensions`). Not ported
//! yet: the main arm, the skin's parts and the attachments.

use crate::net::minecraft::world::entity::entity_dimensions::EntityDimensions;
use crate::net::minecraft::world::entity::living_entity::LivingEntity;
use crate::net::minecraft::world::entity::pose::Pose;

pub struct Avatar;

impl Avatar {
    pub const CROUCH_BB_HEIGHT: f32 = 1.5;
    pub const SWIMMING_BB_WIDTH: f32 = 0.6;
    pub const SWIMMING_BB_HEIGHT: f32 = 0.6;

    pub fn standing_dimensions() -> EntityDimensions {
        EntityDimensions::scalable(0.6, 1.8).with_eye_height(1.62)
    }

    /// `getDefaultDimensions(Pose)`: the pose's in `POSES`, or standing's.
    pub fn get_default_dimensions(pose: Pose) -> EntityDimensions {
        match pose {
            Pose::SLEEPING => LivingEntity::sleeping_dimensions(),
            Pose::FALL_FLYING | Pose::SWIMMING | Pose::SPIN_ATTACK => {
                EntityDimensions::scalable(0.6, 0.6).with_eye_height(0.4)
            }
            Pose::CROUCHING => EntityDimensions::scalable(0.6, 1.5).with_eye_height(1.27),
            Pose::DYING => EntityDimensions::fixed_size(0.2, 0.2).with_eye_height(1.62),
            _ => Avatar::standing_dimensions(),
        }
    }
}
