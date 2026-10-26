//! net/minecraft/world/level/block/FenceBlock.java
//!
//! What decides its blocks' states (its properties and its default state), the sizes its
//! parent's shapes are made with (4, 16, 4, 16 and 24 sixteenths), and its occlusion shapes (a
//! post 2 sixteenths wide, with arms from 6 to 15 sixteenths up), which the light engine and face
//! culling read, and `updateShape` with what it joins (`connectsTo`). Not ported yet: the rest of
//! its behaviour.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::cross_collision_block::{
    self, CrossCollisionBlock,
};
use crate::net::minecraft::world::level::block::fence_gate_block::{self, FenceGateBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct FenceBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(FenceBlock::update_shape),
    get_visual_shape: Some(FenceBlock::get_visual_shape),
    get_occlusion_shape: Some(FenceBlock::get_occlusion_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FenceBlock",
        &cross_collision_block::CLASS,
    )
};

impl FenceBlock {
    pub fn new(properties: Properties) -> BlockData {
        FenceBlock::new_with(
            properties,
            &CLASS,
            FenceBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = CrossCollisionBlock::new_with(
            4.0, 16.0, 4.0, 16.0, 24.0, properties, class, definition,
        );
        let occlusion_shapes = CrossCollisionBlock::make_shapes(&block, 4.0, 16.0, 2.0, 6.0, 15.0);
        let mut fields = block.fields::<CrossCollisionBlock>().clone();
        fields.occlusion_shapes = Some(occlusion_shapes);
        block.set_fields(fields);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CrossCollisionBlock::NORTH, false)
                .set_value(CrossCollisionBlock::EAST, false)
                .set_value(CrossCollisionBlock::SOUTH, false)
                .set_value(CrossCollisionBlock::WEST, false)
                .set_value(CrossCollisionBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CrossCollisionBlock::NORTH,
            CrossCollisionBlock::EAST,
            CrossCollisionBlock::WEST,
            CrossCollisionBlock::SOUTH,
            CrossCollisionBlock::WATERLOGGED,
        ]);
    }
}

impl FenceBlock {
    /// `getOcclusionShape`: the state's of `occlusionShapes`.
    fn get_occlusion_shape(block: &BlockData, state: BlockState) -> VoxelShape {
        block
            .fields::<CrossCollisionBlock>()
            .occlusion_shapes
            .as_ref()
            .expect("a fence makes its occlusion shapes")[state.index() as usize]
            .clone()
    }
}

impl FenceBlock {
    /// `getVisualShape`: its shape.
    fn get_visual_shape(
        _block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        state.get_shape_in(level, pos, context)
    }
}

impl FenceBlock {
    /// `connectsTo(state, faceSolid, direction)`: whether it joins a neighbour that way, whose
    /// face toward it is solid or not: a fence of its kind (wooden or not), a gate across that
    /// way, or a solid face, unless the neighbour is one fences don't join.
    pub fn connects_to(
        block: &BlockData,
        state: BlockState,
        face_solid: bool,
        direction: Direction,
    ) -> bool {
        let same_fence = FenceBlock::is_same_fence(block, state);
        let gate = state.get_block().is_instance_of(&fence_gate_block::CLASS)
            && FenceGateBlock::connects_to_direction(state, direction);
        !Block::is_exception_for_connection(state) && face_solid || same_fence || gate
    }

    /// `isSameFence(state)`: a fence, wooden if this one is.
    fn is_same_fence(block: &BlockData, state: BlockState) -> bool {
        state.is_tag(BlockTags::FENCES)
            && state.is_tag(BlockTags::WOODEN_FENCES)
                == block.default_block_state().is_tag(BlockTags::WOODEN_FENCES)
    }

    /// `updateShape`: joined to its neighbour that way or not, sideways; its parent's up and down.
    /// The water's tick it schedules when it's waterlogged goes nowhere on the client.
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
        if direction_to_neighbour.get_axis().is_horizontal() {
            let opposite = direction_to_neighbour.get_opposite();
            state.set_value(
                CrossCollisionBlock::property_by_direction(direction_to_neighbour),
                FenceBlock::connects_to(
                    block,
                    neighbour_state,
                    neighbour_state.is_face_sturdy(level, neighbour_pos, opposite),
                    opposite,
                ),
            )
        } else {
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
}
