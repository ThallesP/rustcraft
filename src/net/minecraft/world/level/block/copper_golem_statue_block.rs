//! net/minecraft/world/level/block/CopperGolemStatueBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and `Pose`'s
//! constants. Not ported yet: the rest, including `Pose`'s `BY_ID`, `CODEC` and `getNextPose`.
//!
//! Of its behaviour, `getFluidState` and its shape are ported.

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::fmt;
use std::sync::LazyLock;

pub struct CopperGolemStatueBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(CopperGolemStatueBlock::update_shape),
    get_shape: Some(CopperGolemStatueBlock::get_shape),
    get_fluid_state: Some(CopperGolemStatueBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CopperGolemStatueBlock",
        &base_entity_block::CLASS,
    )
};

impl CopperGolemStatueBlock {
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;
    pub const POSE: &'static EnumProperty<Pose> = BlockStateProperties::COPPER_GOLEM_POSE;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        CopperGolemStatueBlock::new_with(
            properties,
            &CLASS,
            CopperGolemStatueBlock::create_block_state_definition,
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
                .default_block_state()
                .set_value(CopperGolemStatueBlock::FACING, Direction::NORTH)
                .set_value(CopperGolemStatueBlock::POSE, Pose::STANDING)
                .set_value(CopperGolemStatueBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        Block::create_block_state_definition(builder);
        builder.add(&[
            CopperGolemStatueBlock::FACING,
            CopperGolemStatueBlock::POSE,
            CopperGolemStatueBlock::WATERLOGGED,
        ]);
    }
}

/// `CopperGolemStatueBlock.Pose`: how a copper golem statue stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pose {
    STANDING,
    SITTING,
    RUNNING,
    STAR,
}

impl Pose {
    pub const VALUES: [Pose; 4] = [Pose::STANDING, Pose::SITTING, Pose::RUNNING, Pose::STAR];

    pub fn values() -> [Pose; 4] {
        Pose::VALUES
    }
}

impl Enum for Pose {
    const ENUM_CONSTANTS: &'static [Pose] = &Pose::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for Pose {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            Pose::STANDING => "standing",
            Pose::SITTING => "sitting",
            Pose::RUNNING => "running",
            Pose::STAR => "star",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for Pose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for Pose {
    const CLASS: &'static str = "net.minecraft.world.level.block.CopperGolemStatueBlock$Pose";
}

impl CopperGolemStatueBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(10.0, 0.0, 14.0));

impl CopperGolemStatueBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }
}

impl CopperGolemStatueBlock {
    /// `updateShape`: its parent's. The water's tick it schedules when it's waterlogged goes
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
    }
}
