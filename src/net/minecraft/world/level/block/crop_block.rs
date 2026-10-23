//! net/minecraft/world/level/block/CropBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct CropBlock;

/// What `getAgeProperty()` and `getMaxAge()` give for the block's class, which `CropBlock`'s
/// constructor and `isRandomlyTicking` ask.
pub struct CropBlockFields {
    pub age_property: &'static IntegerProperty,
    pub max_age: i32,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(CropBlock::get_shape),
    is_randomly_ticking: Some(CropBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CropBlock",
        &vegetation_block::CLASS,
    )
};

impl CropBlock {
    pub const MAX_AGE: i32 = 7;
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_7;

    pub fn new(properties: Properties) -> BlockData {
        CropBlock::new_with(
            properties,
            &CLASS,
            CropBlock::create_block_state_definition,
            CropBlock::AGE,
            CropBlock::MAX_AGE,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
        age_property: &'static IntegerProperty,
        max_age: i32,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(age_property, 0),
        );
        block.set_fields(CropBlockFields {
            age_property,
            max_age,
        });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[CropBlock::AGE]);
    }

    /// While it isn't fully grown: `!isMaxAge(state)`.
    fn is_randomly_ticking(block: &BlockData, state: BlockState) -> bool {
        let fields = block.fields::<CropBlockFields>();
        state.get_value(fields.age_property) < fields.max_age
    }
}

static SHAPES: LazyLock<Vec<VoxelShape>> =
    LazyLock::new(|| Block::boxes(7, |age| Block::column(16.0, 0.0, f64::from(2 + age * 2))));

impl CropBlock {
    /// `getShape`: taller as it grows.
    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPES[state.get_value(block.fields::<CropBlockFields>().age_property) as usize].clone()
    }
}
