//! net/minecraft/world/level/block/GrindstoneBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shapes (the wheel between two legs, on the face it hangs from) are ported.

use crate::com::mojang::math::octahedral_group::OctahedralGroup;
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
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct GrindstoneBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(GrindstoneBlock::get_shape),
    get_collision_shape: Some(GrindstoneBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.GrindstoneBlock",
        &face_attached_horizontal_directional_block::CLASS,
    )
};

impl GrindstoneBlock {
    pub fn new(properties: Properties) -> BlockData {
        GrindstoneBlock::new_with(
            properties,
            &CLASS,
            GrindstoneBlock::create_block_state_definition,
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
                .set_value(
                    FaceAttachedHorizontalDirectionalBlock::FACE,
                    AttachFace::WALL,
                ),
        );
        let shapes = GrindstoneBlock::make_shapes(&block);
        block.set_fields(GrindstoneBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            FaceAttachedHorizontalDirectionalBlock::FACE,
        ]);
    }
}

impl GrindstoneBlock {
    /// `makeShapes`: the wheel between its legs, facing north on a wall, turned to its face.
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let left_legs = Shapes::or(
            &Block::r#box(2.0, 6.0, 7.0, 4.0, 10.0, 16.0),
            &Block::r#box(2.0, 5.0, 3.0, 4.0, 11.0, 9.0),
        );
        let right_legs = Shapes::rotate(&left_legs, OctahedralGroup::INVERT_X);
        let north = Shapes::or_tail(
            &Block::box_z(8.0, 2.0, 14.0, 0.0, 12.0),
            &[left_legs, right_legs],
        );
        let attach_face = Shapes::rotate_attach_face(&north);
        Block::get_shape_for_each_state(block, |state| {
            Shapes::get(
                Shapes::get(
                    &attach_face,
                    state.get_value(FaceAttachedHorizontalDirectionalBlock::FACE),
                ),
                state.get_value(HorizontalDirectionalBlock::FACING),
            )
            .clone()
        })
    }

    /// `getVoxelShape`, which `getShape` and `getCollisionShape` both answer.
    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<GrindstoneBlock>().shapes[state.index() as usize].clone()
    }
}
