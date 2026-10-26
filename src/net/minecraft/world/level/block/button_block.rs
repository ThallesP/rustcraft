//! net/minecraft/world/level/block/ButtonBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type` and `ticksToStayPressed`.
//!
//! Of its behaviour, its shape (a button on the face it hangs from, sunk in while it's pressed) is
//! ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
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
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct ButtonBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    is_signal_source: Some(ButtonBlock::is_signal_source),
    get_shape: Some(ButtonBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ButtonBlock",
        &face_attached_horizontal_directional_block::CLASS,
    )
};

impl ButtonBlock {
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        ButtonBlock::new_with(
            properties,
            &CLASS,
            ButtonBlock::create_block_state_definition,
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
                .set_value(ButtonBlock::POWERED, false)
                .set_value(
                    FaceAttachedHorizontalDirectionalBlock::FACE,
                    AttachFace::WALL,
                ),
        );
        let shapes = ButtonBlock::make_shapes(&block);
        block.set_fields(ButtonBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            ButtonBlock::POWERED,
            FaceAttachedHorizontalDirectionalBlock::FACE,
        ]);
    }
}

impl ButtonBlock {
    /// `makeShapes`: the button on its face, less the cube it's sunk into (deeper when pressed).
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let pressed_shaper = Block::cube(14.0);
        let unpressed_shaper = Block::cube(12.0);
        let attach_face = Shapes::rotate_attach_face(&Block::box_z_sized(6.0, 4.0, 8.0, 16.0));
        Block::get_shape_for_each_state(block, |state| {
            let on_face = Shapes::get(
                Shapes::get(
                    &attach_face,
                    state.get_value(FaceAttachedHorizontalDirectionalBlock::FACE),
                ),
                state.get_value(HorizontalDirectionalBlock::FACING),
            );
            let shaper = if state.get_value(ButtonBlock::POWERED) {
                &pressed_shaper
            } else {
                &unpressed_shaper
            };
            Shapes::join(on_face, shaper, BooleanOp::ONLY_FIRST)
        })
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<ButtonBlock>().shapes[state.index() as usize].clone()
    }
}

impl ButtonBlock {
    /// `isSignalSource`: it is one.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
