//! net/minecraft/world/level/block/HopperBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shapes (a bowl on a neck, with its spout toward where it faces, and its
//! inside and spout to interact with) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

pub struct HopperBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
    /// `interactionShapes`, by where it faces.
    interaction_shapes: Vec<(Direction, VoxelShape)>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(HopperBlock::get_shape),
    get_interaction_shape: Some(HopperBlock::get_interaction_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.HopperBlock",
        &base_entity_block::CLASS,
    )
};

impl HopperBlock {
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::FACING_HOPPER;
    pub const ENABLED: &'static BooleanProperty = BlockStateProperties::ENABLED;

    pub fn new(properties: Properties) -> BlockData {
        HopperBlock::new_with(
            properties,
            &CLASS,
            HopperBlock::create_block_state_definition,
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
                .set_value(HopperBlock::FACING, Direction::DOWN)
                .set_value(HopperBlock::ENABLED, true),
        );
        let inside = Block::column(12.0, 11.0, 16.0);
        let shapes = HopperBlock::make_shapes(&block, &inside);
        let mut interaction_shapes = Shapes::rotate_horizontal(&Shapes::or(
            &inside,
            &Block::box_z(4.0, 8.0, 10.0, 0.0, 4.0),
        ));
        interaction_shapes.push((Direction::DOWN, inside));
        block.set_fields(HopperBlock {
            shapes,
            interaction_shapes,
        });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[HopperBlock::FACING, HopperBlock::ENABLED]);
    }
}

impl HopperBlock {
    /// `makeShapes`: the bowl less its inside, with its spout toward where it faces, for each
    /// state but for whether it's enabled.
    fn make_shapes(block: &BlockData, inside: &VoxelShape) -> Vec<VoxelShape> {
        let spoutless_hopper_outline = Shapes::or(
            &Block::column(16.0, 10.0, 16.0),
            &Block::column(8.0, 4.0, 10.0),
        );
        let spoutless_hopper =
            Shapes::join(&spoutless_hopper_outline, inside, BooleanOp::ONLY_FIRST);
        let spouts = Shapes::rotate_all_about(
            &Block::box_z(4.0, 4.0, 8.0, 0.0, 8.0),
            &Vec3::new(8.0, 6.0, 8.0).scale(0.0625),
        );
        Block::get_shape_for_each_state_ignoring(
            block,
            |state| {
                Shapes::or(
                    &spoutless_hopper,
                    &Shapes::join(
                        Shapes::get(&spouts, state.get_value(HopperBlock::FACING)),
                        &Shapes::block(),
                        BooleanOp::AND,
                    ),
                )
            },
            &[HopperBlock::ENABLED],
        )
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<HopperBlock>().shapes[state.index() as usize].clone()
    }

    fn get_interaction_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> VoxelShape {
        Shapes::get(
            &block.fields::<HopperBlock>().interaction_shapes,
            state.get_value(HopperBlock::FACING),
        )
        .clone()
    }
}
