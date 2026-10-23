//! net/minecraft/world/level/block/WallBlock.java
//!
//! What decides its blocks' states (its properties and its default state) and its shapes: a post
//! when it's up, and a low or tall side toward each neighbour it connects to, as high as its post
//! for its shape and a block and a half for its collision shape, worked out for each state when
//! it's made (`makeShapes`). Java works a waterlogged state's out once with its dry twin's
//! (`getShapeForEachState`'s ignored property), which is the same shape. Not ported yet: the rest
//! of its behaviour (connecting to its neighbours, `TEST_SHAPE_POST`, `TEST_SHAPES_WALL`).
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
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
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct WallBlock {
    /// `shapes` and `collisionShapes`, by state index.
    shapes: Vec<VoxelShape>,
    collision_shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
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
