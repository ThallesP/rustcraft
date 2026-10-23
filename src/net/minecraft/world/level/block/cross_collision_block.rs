//! net/minecraft/world/level/block/CrossCollisionBlock.java
//!
//! Its shapes: a post, with an arm toward each side it connects to, as tall as its post and its
//! wall for its shape, and as its collision height (a fence's one and a half blocks) for its
//! collision shape, worked out for each state when it's made (`makeShapes`). The constructor's
//! sizes are its subclasses' (`FenceBlock`, `IronBarsBlock`). Not ported yet: the rest of its
//! behaviour.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::pipe_block::PipeBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct CrossCollisionBlock {
    /// `collisionShapes` and `shapes`, by state index.
    collision_shapes: Vec<VoxelShape>,
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(CrossCollisionBlock::get_fluid_state),
    get_shape: Some(CrossCollisionBlock::get_shape),
    get_collision_shape: Some(CrossCollisionBlock::get_collision_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CrossCollisionBlock",
        &block::CLASS,
    )
};

impl CrossCollisionBlock {
    pub const NORTH: &'static BooleanProperty = PipeBlock::NORTH;
    pub const EAST: &'static BooleanProperty = PipeBlock::EAST;
    pub const SOUTH: &'static BooleanProperty = PipeBlock::SOUTH;
    pub const WEST: &'static BooleanProperty = PipeBlock::WEST;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    /// `PROPERTY_BY_DIRECTION`: `PipeBlock`'s horizontal ones.
    pub const PROPERTY_BY_DIRECTION: [(Direction, &'static BooleanProperty); 4] = [
        (Direction::NORTH, CrossCollisionBlock::NORTH),
        (Direction::EAST, CrossCollisionBlock::EAST),
        (Direction::SOUTH, CrossCollisionBlock::SOUTH),
        (Direction::WEST, CrossCollisionBlock::WEST),
    ];

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_with(
        post_width: f32,
        post_height: f32,
        wall_width: f32,
        wall_height: f32,
        collision_height: f32,
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        let collision_shapes = CrossCollisionBlock::make_shapes(
            &block,
            post_width,
            collision_height,
            wall_width,
            0.0,
            collision_height,
        );
        let shapes = CrossCollisionBlock::make_shapes(
            &block,
            post_width,
            post_height,
            wall_width,
            0.0,
            wall_height,
        );
        block.set_fields(CrossCollisionBlock {
            collision_shapes,
            shapes,
        });
        block
    }

    /// Each state's post with its arms. The arms are joined in `PROPERTY_BY_DIRECTION`'s order,
    /// which doesn't change the union.
    fn make_shapes(
        block: &BlockData,
        post_width: f32,
        post_height: f32,
        wall_width: f32,
        wall_bottom: f32,
        wall_top: f32,
    ) -> Vec<VoxelShape> {
        let post = Block::column(f64::from(post_width), 0.0, f64::from(post_height));
        let arms = Shapes::rotate_horizontal(&Block::box_z(
            f64::from(wall_width),
            f64::from(wall_bottom),
            f64::from(wall_top),
            0.0,
            8.0,
        ));
        Block::get_shape_for_each_state(block, |state| {
            let mut shape = post.clone();
            for (direction, property) in CrossCollisionBlock::PROPERTY_BY_DIRECTION {
                if state.get_value(property) {
                    shape = Shapes::or(&shape, Shapes::get(&arms, direction));
                }
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
        block.fields::<CrossCollisionBlock>().shapes[state.index() as usize].clone()
    }

    fn get_collision_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<CrossCollisionBlock>().collision_shapes[state.index() as usize].clone()
    }
}

impl CrossCollisionBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}
