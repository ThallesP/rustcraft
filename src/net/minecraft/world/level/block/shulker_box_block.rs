//! net/minecraft/world/level/block/ShulkerBoxBlock.java
//!
//! Its properties and its default state, `propagatesSkylightDown`, its shapes, which follow its
//! block entity's lid (the whole block without one, as a closed box's lid gives), and its ticker.
//! Not ported yet: the rest of its behaviour, and its constructor's `color`.

use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::DirectionalBlock;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::entity::shulker_box_block_entity::ShulkerBoxBlockEntity;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct ShulkerBoxBlock;

pub static CLASS: BlockClass = BlockClass {
    get_ticker: Some(ShulkerBoxBlock::get_ticker),
    get_shape: Some(ShulkerBoxBlock::get_shape),
    get_block_support_shape: Some(ShulkerBoxBlock::get_block_support_shape),
    propagates_skylight_down: Some(ShulkerBoxBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ShulkerBoxBlock",
        &base_entity_block::CLASS,
    )
};

impl ShulkerBoxBlock {
    pub const FACING: &'static EnumProperty<Direction> = DirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        ShulkerBoxBlock::new_with(
            properties,
            &CLASS,
            ShulkerBoxBlock::create_block_state_definition,
        )
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
                .set_value(ShulkerBoxBlock::FACING, Direction::UP),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[ShulkerBoxBlock::FACING]);
    }
}

impl ShulkerBoxBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        false
    }
}

/// `SHAPES_OPEN_SUPPORT`: what an open box's sides support, the rim on the side it opens from.
static SHAPES_OPEN_SUPPORT: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_all(&Block::box_z_square(16.0, 0.0, 1.0)));

impl ShulkerBoxBlock {
    /// `getTicker`: the block entity's `tick`, on both sides.
    fn get_ticker(
        _block: &BlockData,
        _state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        base_entity_block::create_ticker_helper(
            r#type,
            BlockEntityTypes::SHULKER_BOX,
            ShulkerBoxBlockEntity::tick,
        )
    }

    /// `getBlockSupportShape`: the whole block, or an open box's rim on the side it opens from.
    fn get_block_support_shape(
        _block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> VoxelShape {
        match level.get_block_entity(pos).as_deref() {
            Some(block_entity) if matches!(&block_entity.data, BlockEntityData::ShulkerBox(shulker) if !shulker.is_closed()) => {
                Shapes::get(
                    &SHAPES_OPEN_SUPPORT,
                    state.get_value(ShulkerBoxBlock::FACING).get_opposite(),
                )
                .clone()
            }
            _ => Shapes::block(),
        }
    }

    /// `getShape`: the block entity's box, as far as its lid is open, or the whole block without
    /// one.
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        match level
            .get_block_entity(pos)
            .as_deref()
            .map(|block_entity| &block_entity.data)
        {
            Some(BlockEntityData::ShulkerBox(shulker_box_block_entity)) => {
                Shapes::create_aabb(&shulker_box_block_entity.get_bounding_box(state))
            }
            _ => Shapes::block(),
        }
    }
}
