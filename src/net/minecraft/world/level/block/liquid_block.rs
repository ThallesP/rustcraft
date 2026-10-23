//! net/minecraft/world/level/block/LiquidBlock.java
//!
//! What decides its blocks' states (its properties, its default state, its fluid, which states
//! tick and their fluid states) and its shapes: none to see, and none to collide with but for a
//! context that always collides with fluids. A source only holds up a living entity that stands on
//! fluids (`canStandOnFluid`, `getLiquidCollisionShape`), which no ported entity does: striders
//! aren't ported, so the source's stable shape isn't either. Not ported yet: the rest.
//! `stateCache`, the fluid states Java's constructor makes for each level, are made when they're
//! asked for, which is the same: `FluidState`s are values.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct LiquidBlock {
    pub fluid: Fluids,
}

pub static CLASS: BlockClass = BlockClass {
    propagates_skylight_down: Some(LiquidBlock::propagates_skylight_down),
    is_randomly_ticking: Some(LiquidBlock::is_randomly_ticking),
    get_fluid_state: Some(LiquidBlock::get_fluid_state),
    get_shape: Some(LiquidBlock::get_shape),
    get_collision_shape: Some(LiquidBlock::get_collision_shape),
    ..BlockClass::new("net.minecraft.world.level.block.LiquidBlock", &block::CLASS)
};

impl LiquidBlock {
    pub const LEVEL: &'static IntegerProperty = BlockStateProperties::LEVEL;

    pub fn new(fluid: Fluids, properties: Properties) -> BlockData {
        LiquidBlock::new_with(
            fluid,
            properties,
            &CLASS,
            LiquidBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        fluid: Fluids,
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.set_fields(LiquidBlock { fluid });
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(LiquidBlock::LEVEL, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[LiquidBlock::LEVEL]);
    }

    /// `state.getFluidState().isRandomlyTicking()`: whatever its level, the state's fluid is its
    /// block's, flowing or still, which ticks when it's lava.
    fn is_randomly_ticking(block: &BlockData, _state: BlockState) -> bool {
        block.fields::<LiquidBlock>().fluid.is_randomly_ticking()
    }
}

impl LiquidBlock {
    /// `getFluidState`: the source at level 0, flowing ones from 7 down to 1 at levels 1 to 7, and
    /// falling at 8 and past it.
    fn get_fluid_state(block: &BlockData, state: BlockState) -> FluidState {
        let level = state.get_value(LiquidBlock::LEVEL);
        let fluid = block.fields::<LiquidBlock>().fluid;
        match level.min(8) {
            0 => fluid.get_source(false),
            8 => fluid.get_flowing(8, true),
            level => fluid.get_flowing(8 - level, false),
        }
    }

    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::empty()
    }

    /// `getCollisionShape`: the whole block for a context that always collides with fluids, and
    /// otherwise nothing, since only a living entity that stands on fluids collides with a source
    /// (`ifMobIsColliding`), and none of the ported ones does.
    fn get_collision_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        if context.always_collide_with_fluid() {
            Shapes::block()
        } else {
            Shapes::empty()
        }
    }
}

impl LiquidBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        false
    }
}
