//! net/minecraft/world/level/block/BellBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shapes (the bell, with its frame on the floor, one wall, two walls or the
//! ceiling) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::bell_block_entity::BellBlockEntity;
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::bell_attach_type::BellAttachType;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct BellBlock;

pub static CLASS: BlockClass = BlockClass {
    get_ticker: Some(BellBlock::get_ticker),
    get_shape: Some(BellBlock::get_shape),
    get_collision_shape: Some(BellBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BellBlock",
        &base_entity_block::CLASS,
    )
};

impl BellBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const ATTACHMENT: &'static EnumProperty<BellAttachType> =
        BlockStateProperties::BELL_ATTACHMENT;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        BellBlock::new_with(properties, &CLASS, BellBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BellBlock::FACING, Direction::NORTH)
                .set_value(BellBlock::ATTACHMENT, BellAttachType::FLOOR)
                .set_value(BellBlock::POWERED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[BellBlock::FACING, BellBlock::ATTACHMENT, BellBlock::POWERED]);
    }
}

static BELL_SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| {
    Shapes::or(
        &Block::column(6.0, 6.0, 13.0),
        &Block::column(8.0, 4.0, 6.0),
    )
});
static SHAPE_CEILING: LazyLock<VoxelShape> =
    LazyLock::new(|| Shapes::or(&BELL_SHAPE, &Block::column(2.0, 13.0, 16.0)));
static SHAPE_FLOOR: LazyLock<Vec<(Axis, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal_axis(&Block::cube_xyz(16.0, 16.0, 8.0)));
static SHAPE_DOUBLE_WALL: LazyLock<Vec<(Axis, VoxelShape)>> = LazyLock::new(|| {
    Shapes::rotate_horizontal_axis(&Shapes::or(
        &BELL_SHAPE,
        &Block::column_xz(2.0, 16.0, 13.0, 15.0),
    ))
});
static SHAPE_SINGLE_WALL: LazyLock<Vec<(Direction, VoxelShape)>> = LazyLock::new(|| {
    Shapes::rotate_horizontal(&Shapes::or(
        &BELL_SHAPE,
        &Block::box_z(2.0, 13.0, 15.0, 0.0, 13.0),
    ))
});

impl BellBlock {
    /// `getVoxelShape`, which `getShape` and `getCollisionShape` both answer.
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let facing = state.get_value(BellBlock::FACING);
        match state.get_value(BellBlock::ATTACHMENT) {
            BellAttachType::FLOOR => Shapes::get(&SHAPE_FLOOR, facing.get_axis()).clone(),
            BellAttachType::SINGLE_WALL => Shapes::get(&SHAPE_SINGLE_WALL, facing).clone(),
            BellAttachType::DOUBLE_WALL => {
                Shapes::get(&SHAPE_DOUBLE_WALL, facing.get_axis()).clone()
            }
            BellAttachType::CEILING => SHAPE_CEILING.clone(),
        }
    }
}

impl BellBlock {
    /// `getTicker`: on the client, `clientTick`.
    fn get_ticker(
        _block: &BlockData,
        _state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        base_entity_block::create_ticker_helper(
            r#type,
            BlockEntityTypes::BELL,
            BellBlockEntity::client_tick,
        )
    }
}
