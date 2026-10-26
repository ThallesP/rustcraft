//! net/minecraft/world/level/block/entity/ShulkerBoxBlockEntity.java
//!
//! A shulker box's colour (its block's, none for the plain box) and its lid's animation, which
//! starts closed: the server's block event opens and closes it (`triggerEvent`), and its ticker,
//! on both sides, moves the lid a tenth a tick (`tick`, `updateAnimation`), pushing what's in its
//! way while it opens (`moveCollidedEntities`: the player, the only entity the port moves). Its
//! block's shapes follow the lid (`getBoundingBox`). Not ported yet: its items (`loadFromTag`'s
//! `Items`, item stacks) and loot table, its openers and sounds (the server's), and the updates to
//! its neighbours as the lid starts and stops moving (`doNeighborUpdates`: their shapes and their
//! `neighborChanged`, neither ported).

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::monster::shulker::Shulker;
use crate::net::minecraft::world::entity::mover_type::MoverType;
use crate::net::minecraft::world::item::dye_color::DyeColor;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::shulker_box_block::{self, ShulkerBoxBlock};
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::material::push_reaction::PushReaction;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::vec3::Vec3;

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
    pub const EVENT_SET_OPEN_COUNT: i32 = 1;

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

    /// `tick`, its ticker on both sides: `updateAnimation`.
    pub fn tick(level: &mut LevelTicking<'_>, pos: &BlockPos, state: BlockState) {
        let mut block_entity = level.block_entity(pos);
        let BlockEntityData::ShulkerBox(entity) = &mut block_entity.data else {
            return;
        };
        entity.progress_old = entity.progress;
        let opening = match entity.animation_status {
            AnimationStatus::CLOSED => {
                entity.progress = 0.0;
                false
            }
            AnimationStatus::OPENING => {
                entity.progress += 0.1;
                if entity.progress >= 1.0 {
                    entity.animation_status = AnimationStatus::OPENED;
                    entity.progress = 1.0;
                }
                true
            }
            AnimationStatus::OPENED => {
                entity.progress = 1.0;
                false
            }
            AnimationStatus::CLOSING => {
                entity.progress -= 0.1;
                if entity.progress <= 0.0 {
                    entity.animation_status = AnimationStatus::CLOSED;
                    entity.progress = 0.0;
                }
                false
            }
        };
        let (progress_old, progress) = (entity.progress_old, entity.progress);
        drop(block_entity);
        if opening {
            ShulkerBoxBlockEntity::move_collided_entities(
                level,
                pos,
                state,
                progress_old,
                progress,
            );
        }
    }

    pub fn get_animation_status(&self) -> AnimationStatus {
        self.animation_status
    }

    /// `getBoundingBox`: the box with its lid as far open as it is.
    pub fn get_bounding_box(&self, state: BlockState) -> AABB {
        let bottom_center = Vec3::new(0.5, 0.0, 0.5);
        Shulker::get_progress_aabb(
            1.0,
            state.get_value(ShulkerBoxBlock::FACING),
            0.5 * self.get_progress(1.0),
            &bottom_center,
        )
    }

    /// `moveCollidedEntities`: what the lid swept this tick moved out of its way, along its
    /// facing.
    fn move_collided_entities(
        level: &mut LevelTicking<'_>,
        pos: &BlockPos,
        state: BlockState,
        progress_old: f32,
        progress: f32,
    ) {
        if !state.get_block().is_instance_of(&shulker_box_block::CLASS) {
            return;
        }
        let direction: Direction = state.get_value(ShulkerBoxBlock::FACING);
        let aabb = Shulker::get_progress_delta_aabb(
            1.0,
            direction,
            progress_old,
            progress,
            &Vec3::at_bottom_center_of(pos),
        );
        let Some(entity) = level.player.as_deref_mut() else {
            return;
        };
        if entity.get_bounding_box().intersects(&aabb)
            && entity.get_piston_push_reaction() != PushReaction::IGNORE_ENTITY
        {
            entity.r#move(
                &*level.level,
                MoverType::SHULKER_BOX,
                Vec3::new(
                    (aabb.get_xsize() + 0.01) * f64::from(direction.get_step_x()),
                    (aabb.get_ysize() + 0.01) * f64::from(direction.get_step_y()),
                    (aabb.get_zsize() + 0.01) * f64::from(direction.get_step_z()),
                ),
            );
        }
    }

    /// `triggerEvent`: the number of players with it open, which opens or closes its lid.
    pub fn trigger_event(&mut self, b0: i32, b1: i32) -> bool {
        if b0 == ShulkerBoxBlockEntity::EVENT_SET_OPEN_COUNT {
            self.open_count = b1;
            if b1 == 0 {
                self.animation_status = AnimationStatus::CLOSING;
            }

            if b1 == 1 {
                self.animation_status = AnimationStatus::OPENING;
            }

            true
        } else {
            false
        }
    }

    pub fn open_count(&self) -> i32 {
        self.open_count
    }

    pub fn progress_old(&self) -> f32 {
        self.progress_old
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
