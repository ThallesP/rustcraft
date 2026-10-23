//! net/minecraft/client/renderer/state/level/BlockOutlineRenderState.java
//!
//! The debug shapes' extra outlines (`SharedConstants.DEBUG_SHAPES`: the collision, occlusion and
//! interaction shapes) aren't kept: the debug flag is off in vanilla's builds.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// The block the player looks at, outlined: where it is, whether it draws translucently (then its
/// outline draws after the translucent terrain), the high contrast option, and its shape.
#[derive(Clone, Debug)]
pub struct BlockOutlineRenderState {
    pub pos: BlockPos,
    pub is_translucent: bool,
    pub high_contrast: bool,
    pub shape: VoxelShape,
}
