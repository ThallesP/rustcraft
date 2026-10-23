//! net/minecraft/world/level/block/SupportType.java

use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// How a block's face holds up what's placed against it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SupportType {
    /// The whole face.
    FULL,
    /// A post's worth in the middle, like a torch on a fence.
    CENTER,
    /// A ring around the middle, like a bell's or a lantern's chain.
    RIGID,
}

// `CENTER`'s and `RIGID`'s shapes, fields of their constants in Java.
static CENTER_SUPPORT_SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(2.0, 0.0, 10.0));
static RIGID_SUPPORT_SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| {
    Shapes::join(
        &Shapes::block(),
        &Block::column(12.0, 0.0, 16.0),
        BooleanOp::ONLY_FIRST,
    )
});

impl SupportType {
    pub const VALUES: [SupportType; 3] =
        [SupportType::FULL, SupportType::CENTER, SupportType::RIGID];

    /// Whether the state's face in the direction supports this way: its support shape's face
    /// covers the whole side, or the support type's shape.
    pub fn is_supporting(
        self,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction: Direction,
    ) -> bool {
        let support_shape = state.get_block_support_shape(level, pos);
        match self {
            SupportType::FULL => Block::is_face_full(&support_shape, direction),
            SupportType::CENTER => !Shapes::join_is_not_empty(
                &support_shape.get_face_shape(direction),
                &CENTER_SUPPORT_SHAPE,
                BooleanOp::ONLY_SECOND,
            ),
            SupportType::RIGID => !Shapes::join_is_not_empty(
                &support_shape.get_face_shape(direction),
                &RIGID_SUPPORT_SHAPE,
                BooleanOp::ONLY_SECOND,
            ),
        }
    }
}
