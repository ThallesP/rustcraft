//! net/minecraft/world/level/block/IronBarsBlock.java
//!
//! What decides its blocks' states (its properties and its default state), and the sizes its
//! parent's shapes are made with (2, 16, 2, 16 and 16 sixteenths). Not ported yet: the rest of its
//! behaviour.
//!
//! Of its behaviour, its visual shape (nothing) and `skipRendering` (between bars joined to each
//! other, and above and below bars) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::cross_collision_block::{
    self, CrossCollisionBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct IronBarsBlock;

pub static CLASS: BlockClass = BlockClass {
    get_visual_shape: Some(IronBarsBlock::get_visual_shape),
    skip_rendering: Some(IronBarsBlock::skip_rendering),
    ..BlockClass::new(
        "net.minecraft.world.level.block.IronBarsBlock",
        &cross_collision_block::CLASS,
    )
};

impl IronBarsBlock {
    pub fn new(properties: Properties) -> BlockData {
        IronBarsBlock::new_with(
            properties,
            &CLASS,
            IronBarsBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = CrossCollisionBlock::new_with(
            2.0, 16.0, 2.0, 16.0, 16.0, properties, class, definition,
        );
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CrossCollisionBlock::NORTH, false)
                .set_value(CrossCollisionBlock::EAST, false)
                .set_value(CrossCollisionBlock::SOUTH, false)
                .set_value(CrossCollisionBlock::WEST, false)
                .set_value(CrossCollisionBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CrossCollisionBlock::NORTH,
            CrossCollisionBlock::EAST,
            CrossCollisionBlock::WEST,
            CrossCollisionBlock::SOUTH,
            CrossCollisionBlock::WATERLOGGED,
        ]);
    }
}

impl IronBarsBlock {
    fn get_visual_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::empty()
    }

    /// `skipRendering`: up and down against the same bars or bars with a side toward it, and
    /// sideways where both sides join.
    fn skip_rendering(
        _block: &BlockData,
        state: BlockState,
        neighbor_state: BlockState,
        direction: Direction,
    ) -> bool {
        // `PROPERTY_BY_DIRECTION.get`, which has no up or down.
        let side = |direction: Direction| {
            CrossCollisionBlock::PROPERTY_BY_DIRECTION
                .iter()
                .find(|(side, _)| *side == direction)
                .map(|(_, property)| *property)
        };
        let opposite = side(direction.get_opposite());
        if neighbor_state.is(state.get_block())
            || neighbor_state.is_tag(BlockTags::BARS)
                && state.is_tag(BlockTags::BARS)
                && opposite.is_some_and(|property| neighbor_state.has_property(property))
        {
            if !direction.get_axis().is_horizontal() {
                return true;
            }
            if let (Some(own), Some(opposite)) = (side(direction), opposite)
                && state.get_value(own)
                && neighbor_state.get_value(opposite)
            {
                return true;
            }
        }
        false
    }
}
