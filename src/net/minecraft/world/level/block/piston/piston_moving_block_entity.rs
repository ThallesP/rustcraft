//! net/minecraft/world/level/block/piston/PistonMovingBlockEntity.java
//!
//! A block a piston moves (or its head, `isSourcePiston`), as its moving piston block's block
//! entity: half a block a tick (`tick`), pushing what's in its way (`moveCollidedEntities`; the
//! player, the only entity the port moves) and carrying what stands on honey
//! (`moveStuckEntities`), its collision shape where it is so far (`getCollisionShape`, without the
//! moving block for the entity it's pushing: `NOCLIP`), then placed (`finalTick`, or the tick
//! after it's done, five ticks later on the client). Not ported yet: `loadAdditional`'s
//! `blockState` (`BlockState.CODEC`), so a moving block a chunk comes with moves air, the
//! neighbours told of the placed block (`neighborChanged`), and what the entities it moves do
//! with their blocks (`applyEffectsFromBlocks`, `removeLatestMovementRecording`).

use std::cell::Cell;

use crate::java::lang::math::Math;
use crate::net::minecraft::client::multiplayer::client_level::{LevelPlayer, LevelTicking};
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::world::entity::mover_type::MoverType;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::entity::block_entity::{
    BlockEntity, BlockEntityData,
};
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::entity::brushable_block_entity::LEGACY_ID_CODEC;
use crate::net::minecraft::world::level::block::piston::piston_base_block::{
    self, PistonBaseBlock,
};
use crate::net::minecraft::world::level::block::piston::piston_head_block::PistonHeadBlock;
use crate::net::minecraft::world::level::block::piston::piston_math::PistonMath;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::piston_type::PistonType;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::push_reaction::PushReaction;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

thread_local! {
    /// `NOCLIP`: the direction a piston is moving an entity, whose moving block doesn't collide
    /// with it meanwhile.
    static NOCLIP: Cell<Option<Direction>> = const { Cell::new(None) };
}

#[derive(Clone, Debug, PartialEq)]
pub struct PistonMovingBlockEntity {
    moved_state: BlockState,
    direction: Direction,
    extending: bool,
    is_source_piston: bool,
    progress: f32,
    progress_o: f32,
    last_ticked: i64,
    death_ticks: i32,
}

impl Default for PistonMovingBlockEntity {
    fn default() -> PistonMovingBlockEntity {
        PistonMovingBlockEntity::new()
    }
}

impl PistonMovingBlockEntity {
    /// `PistonMovingBlockEntity(worldPosition, blockState)`: moving air, its direction as a
    /// loaded one's default (Java's is null until it's loaded).
    pub fn new() -> PistonMovingBlockEntity {
        PistonMovingBlockEntity {
            moved_state: Blocks::AIR.default_block_state(),
            direction: Direction::DOWN,
            extending: false,
            is_source_piston: false,
            progress: 0.0,
            progress_o: 0.0,
            last_ticked: 0,
            death_ticks: 0,
        }
    }

    /// `PistonMovingBlockEntity(worldPosition, blockState, movedState, direction, extending,
    /// isSourcePiston)`, as `MovingPistonBlock.newMovingBlockEntity` makes it.
    pub fn new_moving(
        position: BlockPos,
        block_state: BlockState,
        moved_state: BlockState,
        direction: Direction,
        extending: bool,
        is_source_piston: bool,
    ) -> BlockEntity {
        let mut entity = BlockEntity::new(BlockEntityTypes::PISTON, position, block_state);
        entity.data = BlockEntityData::Piston(PistonMovingBlockEntity {
            moved_state,
            direction,
            extending,
            is_source_piston,
            ..PistonMovingBlockEntity::new()
        });
        entity
    }

    /// `loadAdditional`: its direction, progress and kind; the moved state isn't ported.
    pub fn load_additional(&mut self, input: &TagValueInput<'_>) {
        self.direction = input
            .read("facing", &LEGACY_ID_CODEC)
            .unwrap_or(Direction::DOWN);
        self.progress = input.get_float_or("progress", 0.0);
        self.progress_o = self.progress;
        self.extending = input.get_boolean_or("extending", false);
        self.is_source_piston = input.get_boolean_or("source", false);
    }

    pub fn is_extending(&self) -> bool {
        self.extending
    }

    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    pub fn is_source_piston(&self) -> bool {
        self.is_source_piston
    }

    pub fn get_moved_state(&self) -> BlockState {
        self.moved_state
    }

    pub fn get_last_ticked(&self) -> i64 {
        self.last_ticked
    }

    pub fn death_ticks(&self) -> i32 {
        self.death_ticks
    }

    /// `getProgress(a)`, the partial tick at most 1.
    pub fn get_progress(&self, a: f32) -> f32 {
        let a = if a > 1.0 { 1.0 } else { a };
        crate::net::minecraft::util::mth::Mth::lerp_f32(a, self.progress_o, self.progress)
    }

    pub fn get_x_off(&self, a: f32) -> f32 {
        self.direction.get_step_x() as f32 * self.get_extended_progress(self.get_progress(a))
    }

    pub fn get_y_off(&self, a: f32) -> f32 {
        self.direction.get_step_y() as f32 * self.get_extended_progress(self.get_progress(a))
    }

    pub fn get_z_off(&self, a: f32) -> f32 {
        self.direction.get_step_z() as f32 * self.get_extended_progress(self.get_progress(a))
    }

    fn get_extended_progress(&self, progress: f32) -> f32 {
        if self.extending {
            progress - 1.0
        } else {
            1.0 - progress
        }
    }

    /// `getCollisionRelatedBlockState`: what it moves, or a retracting piston's head.
    fn get_collision_related_block_state(&self) -> BlockState {
        if !self.extending
            && self.is_source_piston
            && self
                .moved_state
                .get_block()
                .is_instance_of(&piston_base_block::CLASS)
        {
            Blocks::PISTON_HEAD
                .default_block_state()
                .set_value(PistonHeadBlock::SHORT, self.progress > 0.25)
                .set_value(
                    PistonHeadBlock::TYPE,
                    if self.moved_state.is(Blocks::STICKY_PISTON) {
                        PistonType::STICKY
                    } else {
                        PistonType::DEFAULT
                    },
                )
                .set_value(
                    BlockStateProperties::FACING,
                    self.moved_state.get_value(BlockStateProperties::FACING),
                )
        } else {
            self.moved_state
        }
    }

    pub fn get_movement_direction(&self) -> Direction {
        if self.extending {
            self.direction
        } else {
            self.direction.get_opposite()
        }
    }

    /// `getPushDirection()`.
    pub fn get_push_direction(&self) -> Direction {
        self.get_movement_direction()
    }

    /// `getMovement`: how far the box has to move along the movement to be out of the other.
    fn get_movement(aabb_to_be_outside_of: &AABB, movement: Direction, aabb: &AABB) -> f64 {
        match movement {
            Direction::EAST => aabb_to_be_outside_of.max_x - aabb.min_x,
            Direction::WEST => aabb.max_x - aabb_to_be_outside_of.min_x,
            Direction::UP => aabb_to_be_outside_of.max_y - aabb.min_y,
            Direction::DOWN => aabb.max_y - aabb_to_be_outside_of.min_y,
            Direction::SOUTH => aabb_to_be_outside_of.max_z - aabb.min_z,
            Direction::NORTH => aabb.max_z - aabb_to_be_outside_of.min_z,
        }
    }

    /// `moveByPositionAndProgress`: a box of the block moved to where it is so far.
    fn move_by_position_and_progress(&self, pos: &BlockPos, aabb: &AABB) -> AABB {
        let current_position = f64::from(self.get_extended_progress(self.progress));
        aabb.move_xyz(
            f64::from(pos.get_x()) + current_position * f64::from(self.direction.get_step_x()),
            f64::from(pos.get_y()) + current_position * f64::from(self.direction.get_step_y()),
            f64::from(pos.get_z()) + current_position * f64::from(self.direction.get_step_z()),
        )
    }

    /// `getCollisionShape(level, pos)`: what it moves where it is so far (a piston's head for a
    /// source), with a retracting piston's extended head; only the head for the entity it's
    /// pushing, while it moves.
    pub fn get_collision_shape(&self, level: &dyn BlockGetter, pos: &BlockPos) -> VoxelShape {
        let piston_head_shape = if !self.extending
            && self.is_source_piston
            && self
                .moved_state
                .get_block()
                .is_instance_of(&piston_base_block::CLASS)
        {
            self.moved_state
                .set_value(PistonBaseBlock::EXTENDED, true)
                .get_collision_shape(level, pos)
        } else {
            Shapes::empty()
        };

        let no_clip_direction = NOCLIP.get();
        if f64::from(self.progress) < 1.0
            && no_clip_direction == Some(self.get_movement_direction())
        {
            return piston_head_shape;
        }

        let block_state = if self.is_source_piston {
            Blocks::PISTON_HEAD
                .default_block_state()
                .set_value(BlockStateProperties::FACING, self.direction)
                .set_value(
                    PistonHeadBlock::SHORT,
                    self.extending != (1.0 - self.progress < 0.25),
                )
        } else {
            self.moved_state
        };
        let extended_progress = self.get_extended_progress(self.progress);
        let dx = f64::from(self.direction.get_step_x() as f32 * extended_progress);
        let dy = f64::from(self.direction.get_step_y() as f32 * extended_progress);
        let dz = f64::from(self.direction.get_step_z() as f32 * extended_progress);
        Shapes::or(
            &piston_head_shape,
            &block_state
                .get_collision_shape(level, pos)
                .move_xyz(dx, dy, dz),
        )
    }

    /// `tick`, on both sides.
    pub fn tick(level: &mut LevelTicking<'_>, pos: &BlockPos, _state: BlockState) {
        let game_time = level.level.get_game_time();
        let entity = {
            let mut block_entity = level.block_entity(pos);
            let BlockEntityData::Piston(entity) = &mut block_entity.data else {
                return;
            };
            entity.last_ticked = game_time;
            entity.progress_o = entity.progress;
            if entity.progress_o >= 1.0 && entity.death_ticks < 5 {
                // `level.isClientSide()`.
                entity.death_ticks += 1;
                return;
            }
            entity.clone()
        };

        if entity.progress_o >= 1.0 {
            level.level.remove_block_entity(pos);
            if level.level.get_block_state(pos).is(Blocks::MOVING_PISTON) {
                let mut new_state =
                    Block::update_from_neighbour_shapes(entity.moved_state, &*level.level, pos);
                if new_state.is_air() {
                    level.level.set_block(pos, entity.moved_state, 340, 512);
                    Block::update_or_destroy(
                        entity.moved_state,
                        new_state,
                        level.level,
                        pos,
                        3,
                        512,
                    );
                } else {
                    if new_state.has_property(BlockStateProperties::WATERLOGGED)
                        && new_state.get_value(BlockStateProperties::WATERLOGGED)
                    {
                        new_state = new_state.set_value(BlockStateProperties::WATERLOGGED, false);
                    }

                    level.level.set_block(pos, new_state, 67, 512);
                }
            }
        } else {
            let new_progress = entity.progress + 0.5;
            PistonMovingBlockEntity::move_collided_entities(level, pos, new_progress, &entity);
            PistonMovingBlockEntity::move_stuck_entities(level, pos, new_progress, &entity);
            if let Some(mut block_entity) = level.level.get_block_entity_mut(pos)
                && let BlockEntityData::Piston(entity) = &mut block_entity.data
            {
                entity.progress = new_progress;
                if entity.progress >= 1.0 {
                    entity.progress = 1.0;
                }
            }
        }
    }

    /// `moveCollidedEntities`: what the moving block's shape sweeps this tick, moved out of its
    /// way (bounced, for slime), and a retracting piston's head pulling it out of the base.
    fn move_collided_entities(
        level: &mut LevelTicking<'_>,
        pos: &BlockPos,
        new_progress: f32,
        this: &PistonMovingBlockEntity,
    ) {
        let movement = this.get_movement_direction();
        let delta_progress = f64::from(new_progress - this.progress);
        let shape = this
            .get_collision_related_block_state()
            .get_collision_shape(&*level.level, pos);
        if shape.is_empty() {
            return;
        }
        let aabb = this.move_by_position_and_progress(pos, &shape.bounds());
        let area = PistonMath::get_movement_area(&aabb, movement, delta_progress).minmax(&aabb);
        let Some(entity) = level.player.as_deref_mut() else {
            return;
        };
        if !entity.get_bounding_box().intersects(&area) {
            return;
        }
        let shape_aabbs = shape.to_aabbs();
        let cause_bounce = this.moved_state.is(Blocks::SLIME_BLOCK);
        if entity.get_piston_push_reaction() == PushReaction::IGNORE_ENTITY {
            return;
        }
        // `canSimulateMovement()`: the local player always may.
        if cause_bounce {
            let delta_movement = entity.get_delta_movement();
            let (mut dx, mut dy, mut dz) = (delta_movement.x, delta_movement.y, delta_movement.z);
            match movement.get_axis() {
                Axis::X => dx = f64::from(movement.get_step_x()),
                Axis::Y => dy = f64::from(movement.get_step_y()),
                Axis::Z => dz = f64::from(movement.get_step_z()),
            }
            entity.set_delta_movement(Vec3::new(dx, dy, dz));
        }

        let mut delta = 0.0;
        for shape_aabb in &shape_aabbs {
            let moving_aabb = PistonMath::get_movement_area(
                &this.move_by_position_and_progress(pos, shape_aabb),
                movement,
                delta_progress,
            );
            let entity_aabb = entity.get_bounding_box();
            if moving_aabb.intersects(&entity_aabb) {
                delta = Math::max_f64(
                    delta,
                    PistonMovingBlockEntity::get_movement(&moving_aabb, movement, &entity_aabb),
                );
                if delta >= delta_progress {
                    break;
                }
            }
        }

        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(delta <= 0.0) {
            delta = Math::min_f64(delta, delta_progress) + 0.01;
            PistonMovingBlockEntity::move_entity_by_piston(level, movement, delta, movement);
            if !this.extending && this.is_source_piston {
                PistonMovingBlockEntity::fix_entity_within_piston_base(
                    level,
                    pos,
                    movement,
                    delta_progress,
                );
            }
        }
    }

    /// `moveEntityByPiston`: the player moved, its own moving block not in its way.
    fn move_entity_by_piston(
        level: &mut LevelTicking<'_>,
        piston_direction: Direction,
        delta: f64,
        movement: Direction,
    ) {
        let Some(entity) = level.player.as_deref_mut() else {
            return;
        };
        NOCLIP.set(Some(piston_direction));
        entity.r#move(
            &*level.level,
            MoverType::PISTON,
            Vec3::new(
                delta * f64::from(movement.get_step_x()),
                delta * f64::from(movement.get_step_y()),
                delta * f64::from(movement.get_step_z()),
            ),
        );
        NOCLIP.set(None);
    }

    /// `moveStuckEntities`: what stands on honey moving sideways, carried along.
    fn move_stuck_entities(
        level: &mut LevelTicking<'_>,
        pos: &BlockPos,
        new_progress: f32,
        this: &PistonMovingBlockEntity,
    ) {
        if !this.moved_state.is(Blocks::HONEY_BLOCK) {
            return;
        }
        let movement = this.get_movement_direction();
        if !movement.get_axis().is_horizontal() {
            return;
        }
        let sticky_top = this
            .moved_state
            .get_collision_shape(&*level.level, pos)
            .max(Axis::Y);
        let aabb = this.move_by_position_and_progress(
            pos,
            &AABB::new(0.0, sticky_top, 0.0, 1.0, 1.5000010000000001, 1.0),
        );
        let delta_progress = f64::from(new_progress - this.progress);
        let Some(entity) = level.player.as_deref() else {
            return;
        };
        if entity.get_bounding_box().intersects(&aabb)
            && PistonMovingBlockEntity::matches_sticky_critera(&aabb, entity, pos)
        {
            PistonMovingBlockEntity::move_entity_by_piston(
                level,
                movement,
                delta_progress,
                movement,
            );
        }
    }

    fn matches_sticky_critera(aabb: &AABB, entity: &dyn LevelPlayer, pos: &BlockPos) -> bool {
        let position = entity.position();
        entity.get_piston_push_reaction() == PushReaction::PUSH_PULL
            && entity.on_ground()
            && (entity.is_supported_by(pos)
                || position.x >= aabb.min_x
                    && position.x <= aabb.max_x
                    && position.z >= aabb.min_z
                    && position.z <= aabb.max_z)
    }

    /// `fixEntityWithinPistonBase`: the player inside the base a head retracts into, moved
    /// out of it if it's in only as far as the head took it.
    fn fix_entity_within_piston_base(
        level: &mut LevelTicking<'_>,
        pos: &BlockPos,
        direction: Direction,
        delta_progress: f64,
    ) {
        let Some(entity) = level.player.as_deref() else {
            return;
        };
        let entity_aabb = entity.get_bounding_box();
        let r#box = Shapes::block().bounds().move_block_pos(pos);
        if entity_aabb.intersects(&r#box) {
            let opposite = direction.get_opposite();
            let mut delta =
                PistonMovingBlockEntity::get_movement(&r#box, opposite, &entity_aabb) + 0.01;
            let delta_intersected = PistonMovingBlockEntity::get_movement(
                &r#box,
                opposite,
                &entity_aabb.intersect(&r#box),
            ) + 0.01;
            if (delta - delta_intersected).abs() < 0.01 {
                delta = Math::min_f64(delta, delta_progress) + 0.01;
                PistonMovingBlockEntity::move_entity_by_piston(level, direction, delta, opposite);
            }
        }
    }

    /// `finalTick`: done at once, its block placed (air for a piston's head) where it's still
    /// moving there.
    pub fn final_tick(level: &mut LevelTicking<'_>, world_position: &BlockPos) {
        let entity = {
            let Some(mut block_entity) = level.level.get_block_entity_mut(world_position) else {
                return;
            };
            let BlockEntityData::Piston(entity) = &mut block_entity.data else {
                return;
            };
            // `progressO < 1.0F || level.isClientSide()`.
            entity.progress = 1.0;
            entity.progress_o = entity.progress;
            entity.clone()
        };
        level.level.remove_block_entity(world_position);
        if level
            .level
            .get_block_state(world_position)
            .is(Blocks::MOVING_PISTON)
        {
            let new_state = if entity.is_source_piston {
                Blocks::AIR.default_block_state()
            } else {
                Block::update_from_neighbour_shapes(
                    entity.moved_state,
                    &*level.level,
                    world_position,
                )
            };
            level.level.set_block(world_position, new_state, 3, 512);
        }
    }
}
