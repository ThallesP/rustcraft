//! net/minecraft/world/level/block/piston/PistonBaseBlock.java
//!
//! Its properties and default state, `useShapeForLightOcclusion`, its shape (the whole block, or
//! its base when it's extended), what it can push (`isPushable`), and its block event on the
//! client (`triggerEvent`): extending or retracting, what it moves becoming moving pistons
//! (`moveBlocks`, with `PistonStructureResolver`). Its constructor's `isSticky` is whether it's
//! the sticky piston. Not ported yet: the server's side (`neighborChanged`, `checkIfExtend`,
//! `getNeighborSignal`, placing it), the neighbours told of what it moved (`updateNeighborsAt`,
//! `updateNeighbourShapes`, `updateIndirectNeighbourShapes`), and the effects of a block it pops
//! (`levelEvent` 2001's particles and sound, whose random draw is kept), which say so.

use crate::java::util::hash_map::HashMap;
use crate::net::minecraft::client::multiplayer::client_level::{ClientLevel, LevelTicking};
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::directional_block::{self, DirectionalBlock};
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::piston::moving_piston_block::MovingPistonBlock;
use crate::net::minecraft::world::level::block::piston::piston_head_block::PistonHeadBlock;
use crate::net::minecraft::world::level::block::piston::piston_moving_block_entity::PistonMovingBlockEntity;
use crate::net::minecraft::world::level::block::piston::piston_structure_resolver::PistonStructureResolver;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::piston_type::PistonType;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::collision_getter::CollisionGetter;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;
use crate::net::minecraft::world::level::material::push_reaction::PushReaction;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct PistonBaseBlock;

pub static CLASS: BlockClass = BlockClass {
    trigger_event: Some(PistonBaseBlock::trigger_event),
    get_shape: Some(PistonBaseBlock::get_shape),
    use_shape_for_light_occlusion: Some(PistonBaseBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.piston.PistonBaseBlock",
        &directional_block::CLASS,
    )
};

impl PistonBaseBlock {
    pub const EXTENDED: &'static BooleanProperty = BlockStateProperties::EXTENDED;

    pub fn new(properties: Properties) -> BlockData {
        PistonBaseBlock::new_with(
            properties,
            &CLASS,
            PistonBaseBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(DirectionalBlock::FACING, Direction::NORTH)
                .set_value(PistonBaseBlock::EXTENDED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[DirectionalBlock::FACING, PistonBaseBlock::EXTENDED]);
    }
}

impl PistonBaseBlock {
    /// `useShapeForLightOcclusion`: when it's extended.
    fn use_shape_for_light_occlusion(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(PistonBaseBlock::EXTENDED)
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_all(&Block::box_z_square(16.0, 4.0, 16.0)));

impl PistonBaseBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(PistonBaseBlock::EXTENDED) {
            Shapes::get(&SHAPES, state.get_value(DirectionalBlock::FACING)).clone()
        } else {
            Shapes::block()
        }
    }
}

impl PistonBaseBlock {
    /// `isSticky`: the sticky piston's.
    fn is_sticky(block: &BlockData) -> bool {
        std::ptr::eq(block, Block::data(Blocks::STICKY_PISTON))
    }

    /// `triggerEvent` on the client: 0 extends it, moving what's in front of it; 1 and 2 retract
    /// it, the head a moving piston, a sticky piston pulling the block in front of the head back
    /// (or just placing it, for 2, or a block it can't pull).
    fn trigger_event(
        block: &BlockData,
        state: BlockState,
        level: &mut LevelTicking<'_>,
        pos: &BlockPos,
        b0: i32,
        b1: i32,
    ) -> bool {
        let direction = state.get_value(DirectionalBlock::FACING);
        let extended_state = state.set_value(PistonBaseBlock::EXTENDED, true);
        let is_sticky = PistonBaseBlock::is_sticky(block);
        if b0 == 0 {
            if !PistonBaseBlock::move_blocks(level, pos, direction, true, is_sticky) {
                return false;
            }

            level.level.set_block(pos, extended_state, 67, 512);
            // `playSound(null, …)`: no sound for the client's own player, its pitch drawn.
            level.level.get_random().next_float();
        } else if b0 == 1 || b0 == 2 {
            let arm_pos = pos.relative(direction);
            if level.level.get_block_entity(&arm_pos).is_some()
                && matches!(
                    level
                        .level
                        .get_existing_block_entity(&arm_pos)
                        .as_deref()
                        .map(|entity| &entity.data),
                    Some(BlockEntityData::Piston(_))
                )
            {
                PistonMovingBlockEntity::final_tick(level, &arm_pos);
            }

            let moving_piston_state = Blocks::MOVING_PISTON
                .default_block_state()
                .set_value(MovingPistonBlock::FACING, direction)
                .set_value(
                    MovingPistonBlock::TYPE,
                    if is_sticky {
                        PistonType::STICKY
                    } else {
                        PistonType::DEFAULT
                    },
                );
            level.level.set_block(pos, moving_piston_state, 276, 512);
            level
                .level
                .set_block_entity(PistonMovingBlockEntity::new_moving(
                    *pos,
                    moving_piston_state,
                    block.default_block_state().set_value(
                        DirectionalBlock::FACING,
                        Direction::from_3d_data_value(b1 & 7),
                    ),
                    direction,
                    false,
                    true,
                ));
            if is_sticky {
                let two_pos = pos.offset_xyz(
                    direction.get_step_x() * 2,
                    direction.get_step_y() * 2,
                    direction.get_step_z() * 2,
                );
                let moving_state = level.level.get_block_state(&two_pos);
                let mut piston_piece = false;
                if moving_state.is(Blocks::MOVING_PISTON) {
                    level.level.get_block_entity(&two_pos);
                    let extending_this_way = matches!(
                        level.level.get_existing_block_entity(&two_pos).as_deref().map(|entity| &entity.data),
                        Some(BlockEntityData::Piston(entity))
                            if entity.get_direction() == direction && entity.is_extending()
                    );
                    if extending_this_way {
                        PistonMovingBlockEntity::final_tick(level, &two_pos);
                        piston_piece = true;
                    }
                }

                if !piston_piece {
                    if b0 != 1
                        || moving_state.is_air()
                        || !PistonBaseBlock::is_pushable(
                            moving_state,
                            level.level,
                            &two_pos,
                            direction.get_opposite(),
                            false,
                            direction,
                        )
                        || moving_state.get_piston_push_reaction() != PushReaction::PUSH_PULL
                            && !moving_state.is(Blocks::PISTON)
                            && !moving_state.is(Blocks::STICKY_PISTON)
                    {
                        level.level.remove_block(&arm_pos, false);
                    } else {
                        PistonBaseBlock::move_blocks(level, pos, direction, false, is_sticky);
                    }
                }
            } else {
                level.level.remove_block(&arm_pos, false);
            }

            level.level.get_random().next_float();
        }

        true
    }

    /// `isPushable`: whether a piston pushing towards `direction` can move the block (or pop
    /// it, if `allow_destroyable`), `connection_direction` the way a `PUSH` block must be pushed.
    pub fn is_pushable(
        state: BlockState,
        level: &ClientLevel,
        pos: &BlockPos,
        direction: Direction,
        allow_destroyable: bool,
        connection_direction: Direction,
    ) -> bool {
        let min_y = LevelHeightAccessor::get_min_y(level);
        let max_y = LevelHeightAccessor::get_max_y(level);
        if pos.get_y() < min_y
            || pos.get_y() > max_y
            || !level.get_world_border().is_within_bounds_block_pos(pos)
        {
            return false;
        }

        if state.is_air() {
            return true;
        }

        if direction == Direction::DOWN && pos.get_y() == min_y {
            return false;
        }

        if direction == Direction::UP && pos.get_y() == max_y {
            return false;
        }

        if !state.is(Blocks::PISTON) && !state.is(Blocks::STICKY_PISTON) {
            if state.get_destroy_speed() == -1.0 {
                return false;
            }

            match state.get_piston_push_reaction() {
                PushReaction::IMMOVEABLE => return false,
                PushReaction::POPPED => return allow_destroyable,
                PushReaction::PUSH => return direction == connection_direction,
                _ => {}
            }
        } else if state.get_value(PistonBaseBlock::EXTENDED) {
            return false;
        }

        !state.has_block_entity()
    }

    /// `moveBlocks`: what the piston moves made moving pistons a block on (with its head, when
    /// it extends), what it pops gone, and what's left behind air. Whether it could.
    fn move_blocks(
        level: &mut LevelTicking<'_>,
        piston_pos: &BlockPos,
        direction: Direction,
        extending: bool,
        is_sticky: bool,
    ) -> bool {
        let arm_pos = piston_pos.relative(direction);
        if !extending
            && level
                .level
                .get_block_state(&arm_pos)
                .is(Blocks::PISTON_HEAD)
        {
            level
                .level
                .set_block(&arm_pos, Blocks::AIR.default_block_state(), 276, 512);
        }

        let mut resolver =
            PistonStructureResolver::new(level.level, *piston_pos, direction, extending);
        if !resolver.resolve() {
            return false;
        }
        let to_push = resolver.get_to_push().to_vec();
        let to_destroy = resolver.get_to_destroy().to_vec();

        let mut delete_after_move: HashMap<BlockPos, BlockState> = HashMap::new();
        let mut to_push_shapes = Vec::new();
        for pos in &to_push {
            let state = level.level.get_block_state(pos);
            to_push_shapes.push(state);
            delete_after_move.put(*pos, state);
        }

        let push_direction = if extending {
            direction
        } else {
            direction.get_opposite()
        };

        for pos in to_destroy.iter().rev() {
            let state = level.level.get_block_state(pos);
            if state.has_block_entity() {
                level.level.get_block_entity(pos);
            }
            // `dropResources`: a client drops nothing.
            if !state.is_tag(BlockTags::FIRE) {
                // `levelEvent(2001, pos, getId(state))`: the block's breaking sound (seeded
                // from the level's random) and particles.
                if !state.is_air() {
                    level.level.get_random().next_long();
                }
                eprintln!(
                    "PistonBaseBlock.moveBlocks's destroy effect (levelEvent 2001) isn't ported"
                );
            }

            level
                .level
                .set_block(pos, Blocks::AIR.default_block_state(), 18, 512);
        }

        for (i, pos) in to_push.iter().enumerate().rev() {
            let pos = pos.relative(push_direction);
            delete_after_move.remove(&pos);
            let actual_state = Blocks::MOVING_PISTON
                .default_block_state()
                .set_value(MovingPistonBlock::FACING, direction);
            level.level.set_block(&pos, actual_state, 324, 512);
            level
                .level
                .set_block_entity(PistonMovingBlockEntity::new_moving(
                    pos,
                    actual_state,
                    to_push_shapes[i],
                    direction,
                    extending,
                    false,
                ));
        }

        if extending {
            let r#type = if is_sticky {
                PistonType::STICKY
            } else {
                PistonType::DEFAULT
            };
            let state = Blocks::PISTON_HEAD
                .default_block_state()
                .set_value(DirectionalBlock::FACING, direction)
                .set_value(PistonHeadBlock::TYPE, r#type);
            let block_state = Blocks::MOVING_PISTON
                .default_block_state()
                .set_value(MovingPistonBlock::FACING, direction)
                .set_value(MovingPistonBlock::TYPE, r#type);
            delete_after_move.remove(&arm_pos);
            level.level.set_block(&arm_pos, block_state, 324, 512);
            level
                .level
                .set_block_entity(PistonMovingBlockEntity::new_moving(
                    arm_pos,
                    block_state,
                    state,
                    direction,
                    true,
                    true,
                ));
        }

        let air = Blocks::AIR.default_block_state();
        for pos in delete_after_move.keys().copied().collect::<Vec<_>>() {
            level.level.set_block(&pos, air, 82, 512);
        }

        true
    }
}
