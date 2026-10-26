//! net/minecraft/world/level/block/LiquidBlock.java
//!
//! What decides its blocks' states (its properties, its default state, its fluid, which states
//! tick and their fluid states), its shapes (none to see, and none to collide with but for a
//! context that always collides with fluids, or a living entity above a source that stands on
//! fluids: `canStandOnFluid`, `getLiquidCollisionShape`, which the context carries; striders
//! aren't ported), `skipRendering` (against its own fluid) and its render shape (invisible: fluids
//! are drawn apart). Not ported yet: the rest.
//! `stateCache`, the fluid states Java's constructor makes for each level, are made when they're
//! asked for, which is the same: `FluidState`s are values.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::{Properties, RenderShape};
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
    update_shape: Some(LiquidBlock::update_shape),
    skip_rendering: Some(LiquidBlock::skip_rendering),
    get_render_shape: Some(LiquidBlock::get_render_shape),
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
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        if context.always_collide_with_fluid() {
            return Shapes::block();
        }
        if state.get_value(LiquidBlock::LEVEL) != 0 {
            return Shapes::empty();
        }
        // `ifMobIsColliding`: a living entity's context.
        context
            .as_entity_collision_context()
            .and_then(|context| context.get_entity())
            .and_then(|entity| entity.living.as_ref())
            .map(|mob| mob.liquid_collision_shape.clone())
            .filter(|liquid_stable_shape| {
                context.is_above(liquid_stable_shape, pos, true)
                    && context.can_stand_on_fluid(
                        &level.get_fluid_state(&pos.above()),
                        &state.get_fluid_state(),
                    )
            })
            .unwrap_or_else(Shapes::empty)
    }
}

impl LiquidBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        false
    }
}

impl LiquidBlock {
    /// `skipRendering`: a face against the same fluid.
    fn skip_rendering(
        block: &BlockData,
        _state: BlockState,
        neighbor_state: BlockState,
        _direction: Direction,
    ) -> bool {
        neighbor_state
            .get_fluid_state()
            .get_type()
            .is_same(block.fields::<LiquidBlock>().fluid)
    }

    fn get_render_shape(_block: &BlockData, _state: BlockState) -> RenderShape {
        RenderShape::INVISIBLE
    }
}

impl LiquidBlock {
    /// `updateShape`: its parent's. The ticks it schedules (its fluid's, and a bubble column's
    /// below) go nowhere on the client.
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
