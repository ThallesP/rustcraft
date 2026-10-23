//! net/minecraft/world/level/block/LeverBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (on the face it hangs from) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::face_attached_horizontal_directional_block::{
    self, FaceAttachedHorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::attach_face::AttachFace;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct LeverBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(LeverBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.LeverBlock",
        &face_attached_horizontal_directional_block::CLASS,
    )
};

impl LeverBlock {
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        LeverBlock::new_with(
            properties,
            &CLASS,
            LeverBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block =
            FaceAttachedHorizontalDirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(LeverBlock::POWERED, false)
                .set_value(
                    FaceAttachedHorizontalDirectionalBlock::FACE,
                    AttachFace::WALL,
                ),
        );
        let shapes = LeverBlock::make_shapes(&block);
        block.set_fields(LeverBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            FaceAttachedHorizontalDirectionalBlock::FACE,
            HorizontalDirectionalBlock::FACING,
            LeverBlock::POWERED,
        ]);
    }
}

impl LeverBlock {
    /// `makeShapes`: on the face it hangs from, for each state but for whether it's powered.
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let attach_face = Shapes::rotate_attach_face(&Block::box_z_sized(6.0, 8.0, 10.0, 16.0));
        Block::get_shape_for_each_state_ignoring(
            block,
            |state| {
                Shapes::get(
                    Shapes::get(
                        &attach_face,
                        state.get_value(FaceAttachedHorizontalDirectionalBlock::FACE),
                    ),
                    state.get_value(HorizontalDirectionalBlock::FACING),
                )
                .clone()
            },
            &[LeverBlock::POWERED],
        )
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<LeverBlock>().shapes[state.index() as usize].clone()
    }
}
