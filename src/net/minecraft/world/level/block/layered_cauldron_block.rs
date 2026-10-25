//! net/minecraft/world/level/block/LayeredCauldronBlock.java
//!
//! What decides its blocks' states (its properties and its default state), and of its behaviour
//! what an entity has to touch to be in it (the cauldron, filled to its level). Not ported yet:
//! the rest of its behaviour (what it does to an entity inside is the entity's fire), and its
//! constructor's `precipitationType` and `interactionMap`.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::level::block::abstract_cauldron_block::{
    self, AbstractCauldronBlock,
};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct LayeredCauldronBlock;

pub static CLASS: BlockClass = BlockClass {
    get_entity_inside_collision_shape: Some(
        LayeredCauldronBlock::get_entity_inside_collision_shape,
    ),
    ..BlockClass::new(
        "net.minecraft.world.level.block.LayeredCauldronBlock",
        &abstract_cauldron_block::CLASS,
    )
};

impl LayeredCauldronBlock {
    pub const LEVEL: &'static IntegerProperty = BlockStateProperties::LEVEL_CAULDRON;

    pub fn new(properties: Properties) -> BlockData {
        LayeredCauldronBlock::new_with(
            properties,
            &CLASS,
            LayeredCauldronBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractCauldronBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(LayeredCauldronBlock::LEVEL, 1),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[LayeredCauldronBlock::LEVEL]);
    }
}

/// `FILLED_SHAPES`: the cauldron filled to each level.
static FILLED_SHAPES: LazyLock<Vec<VoxelShape>> = LazyLock::new(|| {
    Block::boxes(2, |level| {
        Shapes::or(
            &abstract_cauldron_block::SHAPE,
            &Block::column(
                12.0,
                4.0,
                LayeredCauldronBlock::get_pixel_content_height(level + 1),
            ),
        )
    })
});

impl LayeredCauldronBlock {
    /// `getPixelContentHeight(int)`: 6 sixteenths, and 3 more a level.
    fn get_pixel_content_height(level: i32) -> f64 {
        6.0 + f64::from(level) * 3.0
    }

    /// `getEntityInsideCollisionShape`: the cauldron, filled to its level.
    fn get_entity_inside_collision_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _entity: &dyn EntityClass,
    ) -> VoxelShape {
        FILLED_SHAPES[(state.get_value(LayeredCauldronBlock::LEVEL) - 1) as usize].clone()
    }
}
