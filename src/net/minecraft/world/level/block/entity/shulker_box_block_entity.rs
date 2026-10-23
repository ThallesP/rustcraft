//! net/minecraft/world/level/block/entity/ShulkerBoxBlockEntity.java
//!
//! A shulker box's colour (its block's, none for the plain box) and its lid's animation, which
//! starts closed. Not ported yet: its items (`loadFromTag`'s `Items`, item stacks) and loot table,
//! and the lid's moving (`triggerEvent` from `block_event`, the ticker's `updateAnimation`,
//! `getBoundingBox` and the shapes reading it), which come with block events and the level's
//! ticking of block entities: until then a box stays closed, a full block, as it loads.

use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::item::dye_color::DyeColor;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

/// `ShulkerBoxBlockEntity.AnimationStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum AnimationStatus {
    CLOSED,
    OPENING,
    OPENED,
    CLOSING,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShulkerBoxBlockEntity {
    open_count: i32,
    animation_status: AnimationStatus,
    progress: f32,
    progress_old: f32,
    color: Option<DyeColor>,
}

impl ShulkerBoxBlockEntity {
    /// `ShulkerBoxBlockEntity(color, worldPosition, blockState)`.
    pub fn new(color: Option<DyeColor>) -> ShulkerBoxBlockEntity {
        ShulkerBoxBlockEntity {
            open_count: 0,
            animation_status: AnimationStatus::CLOSED,
            progress: 0.0,
            progress_old: 0.0,
            color,
        }
    }

    /// `loadFromTag`'s items and loot table, which aren't ported.
    pub fn load_additional(&mut self, _input: &TagValueInput<'_>) {}

    pub fn get_animation_status(&self) -> AnimationStatus {
        self.animation_status
    }

    pub fn get_progress(&self, a: f32) -> f32 {
        Mth::lerp_f32(a, self.progress_old, self.progress)
    }

    pub fn get_color(&self) -> Option<DyeColor> {
        self.color
    }

    pub fn is_closed(&self) -> bool {
        self.animation_status == AnimationStatus::CLOSED
    }
}
