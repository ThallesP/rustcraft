//! net/minecraft/world/level/block/WallBlock.java
//!
//! What decides its blocks' states (its properties and its default state) and its shapes: a post
//! when it's up, and a low or tall side toward each neighbour it connects to, as high as its post
//! for its shape and a block and a half for its collision shape, worked out for each state when
//! it's made (`makeShapes`). Java works a waterlogged state's out once with its dry twin's
//! (`getShapeForEachState`'s ignored property), which is the same shape. Its shape updates are
//! ported: joining its neighbours (`connectsTo`), its sides tall under what covers them
//! (`TEST_SHAPES_WALL`) and its post raised (`TEST_SHAPE_POST`). Not ported yet: the rest of its
//! behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::fence_gate_block::{self, FenceGateBlock};
use crate::net::minecraft::world::level::block::iron_bars_block;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::wall_side::WallSide;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct WallBlock {
    /// `shapes` and `collisionShapes`, by state index.
    shapes: Vec<VoxelShape>,
    collision_shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(WallBlock::update_shape),
    propagates_skylight_down: Some(WallBlock::propagates_skylight_down),
    get_fluid_state: Some(WallBlock::get_fluid_state),
    get_shape: Some(WallBlock::get_shape),
    get_collision_shape: Some(WallBlock::get_collision_shape),
    ..BlockClass::new("net.minecraft.world.level.block.WallBlock", &block::CLASS)
};

impl WallBlock {
    pub const UP: &'static BooleanProperty = BlockStateProperties::UP;
    pub const EAST: &'static EnumProperty<WallSide> = BlockStateProperties::EAST_WALL;
    pub const NORTH: &'static EnumProperty<WallSide> = BlockStateProperties::NORTH_WALL;
    pub const SOUTH: &'static EnumProperty<WallSide> = BlockStateProperties::SOUTH_WALL;
    pub const WEST: &'static EnumProperty<WallSide> = BlockStateProperties::WEST_WALL;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    /// `PROPERTY_BY_DIRECTION`, an `EnumMap`'s copy: in the directions' order.
    pub const PROPERTY_BY_DIRECTION: [(Direction, &'static EnumProperty<WallSide>); 4] = [
        (Direction::NORTH, WallBlock::NORTH),
        (Direction::SOUTH, WallBlock::SOUTH),
        (Direction::WEST, WallBlock::WEST),
        (Direction::EAST, WallBlock::EAST),
    ];

    pub fn new(properties: Properties) -> BlockData {
        WallBlock::new_with(properties, &CLASS, WallBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(WallBlock::UP, true)
                .set_value(WallBlock::NORTH, WallSide::NONE)
                .set_value(WallBlock::EAST, WallSide::NONE)
                .set_value(WallBlock::SOUTH, WallSide::NONE)
                .set_value(WallBlock::WEST, WallSide::NONE)
                .set_value(WallBlock::WATERLOGGED, false),
        );
        let shapes = WallBlock::make_shapes(&block, 16.0, 14.0);
        let collision_shapes = WallBlock::make_shapes(&block, 24.0, 24.0);
        block.set_fields(WallBlock {
            shapes,
            collision_shapes,
        });
        block
    }

    /// Each state's post and sides, the sides joined in `PROPERTY_BY_DIRECTION`'s order.
    fn make_shapes(block: &BlockData, post_height: f32, wall_top: f32) -> Vec<VoxelShape> {
        let post = Block::column(8.0, 0.0, f64::from(post_height));
        let low =
            Shapes::rotate_horizontal(&Block::box_z(6.0, 0.0, f64::from(wall_top), 0.0, 11.0));
        let tall =
            Shapes::rotate_horizontal(&Block::box_z(6.0, 0.0, f64::from(post_height), 0.0, 11.0));
        Block::get_shape_for_each_state(block, |state| {
            let mut shape = if state.get_value(WallBlock::UP) {
                post.clone()
            } else {
                Shapes::empty()
            };

            for (direction, property) in WallBlock::PROPERTY_BY_DIRECTION {
                let side = match state.get_value(property) {
                    WallSide::NONE => Shapes::empty(),
                    WallSide::LOW => Shapes::get(&low, direction).clone(),
                    WallSide::TALL => Shapes::get(&tall, direction).clone(),
                };
                shape = Shapes::or(&shape, &side);
            }

            shape
        })
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<WallBlock>().shapes[state.index() as usize].clone()
    }

    fn get_collision_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<WallBlock>().collision_shapes[state.index() as usize].clone()
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            WallBlock::UP,
            WallBlock::NORTH,
            WallBlock::EAST,
            WallBlock::WEST,
            WallBlock::SOUTH,
            WallBlock::WATERLOGGED,
        ]);
    }
}

impl WallBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl WallBlock {
    /// `propagatesSkylightDown`: unless it's waterlogged.
    fn propagates_skylight_down(_block: &BlockData, state: BlockState) -> bool {
        !state.get_value(BlockStateProperties::WATERLOGGED)
    }
}

/// `TEST_SHAPE_POST`: what must be covered above for its post to stay down.
static TEST_SHAPE_POST: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(2.0, 0.0, 16.0));

/// `TEST_SHAPES_WALL`: what must be covered above a side for it to be tall.
static TEST_SHAPES_WALL: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&Block::box_z_sized(2.0, 16.0, 0.0, 9.0)));

impl WallBlock {
    /// `connectsTo(state, faceSolid, direction)`: whether it joins a neighbour that way, whose
    /// face toward it is solid or not: a wall, bars or panes, a gate across that way, or a solid
    /// face, unless the neighbour is one walls don't join.
    fn connects_to(state: BlockState, face_solid: bool, direction: Direction) -> bool {
        let block = state.get_block();
        let connected_fence_gate = block.is_instance_of(&fence_gate_block::CLASS)
            && FenceGateBlock::connects_to_direction(state, direction);
        state.is_tag(BlockTags::WALLS)
            || !Block::is_exception_for_connection(state) && face_solid
            || block.is_instance_of(&iron_bars_block::CLASS)
            || connected_fence_gate
    }

    /// `updateShape`: its sides and post from what's above it, or from its neighbour that way
    /// sideways; its parent's below. The water's tick it schedules when it's waterlogged goes
    /// nowhere on the client.
    #[allow(clippy::too_many_arguments)]
    fn update_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        random: &mut dyn RandomSource,
    ) -> BlockState {
        if direction_to_neighbour == Direction::DOWN {
            Block::super_update_shape(
                block,
                &CLASS,
                state,
                level,
                pos,
                direction_to_neighbour,
                neighbour_pos,
                neighbour_state,
                random,
            )
        } else if direction_to_neighbour == Direction::UP {
            WallBlock::top_update(level, state, neighbour_pos, neighbour_state)
        } else {
            WallBlock::side_update(
                level,
                pos,
                state,
                neighbour_pos,
                neighbour_state,
                direction_to_neighbour,
            )
        }
    }

    /// `isConnected(state, northWall)`: whether the side has any wall.
    fn is_connected(state: BlockState, side: &'static EnumProperty<WallSide>) -> bool {
        state.get_value(side) != WallSide::NONE
    }

    /// `isCovered(aboveShape, testShape)`: whether what's above covers all of the test shape.
    fn is_covered(above_shape: &VoxelShape, test_shape: &VoxelShape) -> bool {
        !Shapes::join_is_not_empty(test_shape, above_shape, BooleanOp::ONLY_FIRST)
    }

    /// `topUpdate(level, state, topPos, topNeighbour)`: the sides it has, under what's above.
    fn top_update(
        level: &dyn BlockGetter,
        state: BlockState,
        top_pos: &BlockPos,
        top_neighbour: BlockState,
    ) -> BlockState {
        let north = WallBlock::is_connected(state, WallBlock::NORTH);
        let east = WallBlock::is_connected(state, WallBlock::EAST);
        let south = WallBlock::is_connected(state, WallBlock::SOUTH);
        let west = WallBlock::is_connected(state, WallBlock::WEST);
        WallBlock::update_shape_with(
            level,
            state,
            top_pos,
            top_neighbour,
            north,
            east,
            south,
            west,
        )
    }

    /// `sideUpdate(level, pos, state, neighbourPos, neighbour, direction)`: the side that way
    /// joined to its neighbour or not, the others as they are, under what's above.
    fn side_update(
        level: &dyn BlockGetter,
        pos: &BlockPos,
        state: BlockState,
        neighbour_pos: &BlockPos,
        neighbour: BlockState,
        direction: Direction,
    ) -> BlockState {
        let opposite = direction.get_opposite();
        let connected = |side: Direction, property| {
            if direction == side {
                WallBlock::connects_to(
                    neighbour,
                    neighbour.is_face_sturdy(level, neighbour_pos, opposite),
                    opposite,
                )
            } else {
                WallBlock::is_connected(state, property)
            }
        };
        let is_north_connected = connected(Direction::NORTH, WallBlock::NORTH);
        let is_east_connected = connected(Direction::EAST, WallBlock::EAST);
        let is_south_connected = connected(Direction::SOUTH, WallBlock::SOUTH);
        let is_west_connected = connected(Direction::WEST, WallBlock::WEST);
        let above = pos.above();
        let above_state = level.get_block_state(&above);
        WallBlock::update_shape_with(
            level,
            state,
            &above,
            above_state,
            is_north_connected,
            is_east_connected,
            is_south_connected,
            is_west_connected,
        )
    }

    /// `updateShape(level, state, topPos, topNeighbour, north, east, south, west)`: its sides,
    /// tall where what's above covers them, and its post.
    #[allow(clippy::too_many_arguments)]
    fn update_shape_with(
        level: &dyn BlockGetter,
        state: BlockState,
        top_pos: &BlockPos,
        top_neighbour: BlockState,
        north: bool,
        east: bool,
        south: bool,
        west: bool,
    ) -> BlockState {
        let above_shape = top_neighbour
            .get_collision_shape(level, top_pos)
            .get_face_shape(Direction::DOWN);
        let sides_updated_state =
            WallBlock::update_sides(state, north, east, south, west, &above_shape);
        sides_updated_state.set_value(
            WallBlock::UP,
            WallBlock::should_raise_post(sides_updated_state, top_neighbour, &above_shape),
        )
    }

    /// `shouldRaisePost(state, topNeighbour, aboveShape)`: under a wall's post, at an end or a
    /// corner, or between low sides under something its post holds up.
    fn should_raise_post(
        state: BlockState,
        top_neighbour: BlockState,
        above_shape: &VoxelShape,
    ) -> bool {
        let top_neighbour_has_post = top_neighbour.get_block().is_instance_of(&CLASS)
            && top_neighbour.get_value(WallBlock::UP);
        if top_neighbour_has_post {
            return true;
        }

        let north_wall = state.get_value(WallBlock::NORTH);
        let south_wall = state.get_value(WallBlock::SOUTH);
        let east_wall = state.get_value(WallBlock::EAST);
        let west_wall = state.get_value(WallBlock::WEST);
        let south_none = south_wall == WallSide::NONE;
        let west_none = west_wall == WallSide::NONE;
        let east_none = east_wall == WallSide::NONE;
        let north_none = north_wall == WallSide::NONE;
        let has_corner = north_none && south_none && west_none && east_none
            || north_none != south_none
            || west_none != east_none;
        if has_corner {
            return true;
        }

        let has_high_wall = north_wall == WallSide::TALL && south_wall == WallSide::TALL
            || east_wall == WallSide::TALL && west_wall == WallSide::TALL;
        if has_high_wall {
            false
        } else {
            top_neighbour.is_tag(BlockTags::WALL_POST_OVERRIDE)
                || WallBlock::is_covered(above_shape, &TEST_SHAPE_POST)
        }
    }

    /// `updateSides(state, north, east, south, west, aboveShape)`.
    fn update_sides(
        state: BlockState,
        north_connection: bool,
        east_connection: bool,
        south_connection: bool,
        west_connection: bool,
        above_shape: &VoxelShape,
    ) -> BlockState {
        let side = |connects, direction| {
            WallBlock::make_wall_state(
                connects,
                above_shape,
                Shapes::get(&TEST_SHAPES_WALL, direction),
            )
        };
        state
            .set_value(WallBlock::NORTH, side(north_connection, Direction::NORTH))
            .set_value(WallBlock::EAST, side(east_connection, Direction::EAST))
            .set_value(WallBlock::SOUTH, side(south_connection, Direction::SOUTH))
            .set_value(WallBlock::WEST, side(west_connection, Direction::WEST))
    }

    /// `makeWallState(connectsToSide, aboveShape, testShape)`: none, or low, or tall when what's
    /// above covers it.
    fn make_wall_state(
        connects_to_side: bool,
        above_shape: &VoxelShape,
        test_shape: &VoxelShape,
    ) -> WallSide {
        if connects_to_side {
            if WallBlock::is_covered(above_shape, test_shape) {
                WallSide::TALL
            } else {
                WallSide::LOW
            }
        } else {
            WallSide::NONE
        }
    }
}
