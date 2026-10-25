//! net/minecraft/world/level/block/BubbleColumnBlock.java
//!
//! What decides its blocks' states (its properties and its default state), its shapes and fluid,
//! `entityInside` (an entity precisely in a column is pulled down by one over magma, `drag`, and
//! pushed up by one over soul sand, harder at its top, where nothing is above it) and its
//! particles and sounds (`animateTick`). Not ported yet: the rest of its behaviour (the column
//! growing and going with the blocks under it, `updateColumn`, its ticks and shape updates;
//! picking it up).

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::entity::inside_block_effect_applier::InsideBlockEffectApplier;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_behaviour::RenderShape;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct BubbleColumnBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(BubbleColumnBlock::animate_tick),
    get_shape: Some(BubbleColumnBlock::get_shape),
    get_render_shape: Some(BubbleColumnBlock::get_render_shape),
    get_fluid_state: Some(BubbleColumnBlock::get_fluid_state),
    entity_inside: Some(BubbleColumnBlock::entity_inside),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BubbleColumnBlock",
        &block::CLASS,
    )
};

impl BubbleColumnBlock {
    pub const DRAG_DOWN: &'static BooleanProperty = BlockStateProperties::DRAG;

    pub fn new(properties: Properties) -> BlockData {
        BubbleColumnBlock::new_with(
            properties,
            &CLASS,
            BubbleColumnBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BubbleColumnBlock::DRAG_DOWN, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[BubbleColumnBlock::DRAG_DOWN]);
    }
}

impl BubbleColumnBlock {
    /// `entityInside`: with nothing above to collide with and no fluid, the entity is above the
    /// column (`onAboveBubbleColumn`), else inside it. Java asks the state above for its collision
    /// shape at the column's position, as here.
    fn entity_inside(
        _block: &BlockData,
        state: BlockState,
        level: &dyn Level,
        pos: &BlockPos,
        entity: &mut dyn EntityClass,
        _effect_applier: &mut dyn InsideBlockEffectApplier,
        is_precise: bool,
    ) {
        if is_precise {
            let state_above = level.get_block_state(&pos.above());
            let nothing_above = state_above.get_collision_shape(level, pos).is_empty()
                && state_above.get_fluid_state().is_empty();
            if nothing_above {
                entity.on_above_bubble_column(state.get_value(BubbleColumnBlock::DRAG_DOWN), pos);
            } else {
                entity.on_inside_bubble_column(state.get_value(BubbleColumnBlock::DRAG_DOWN));
            }
        }
    }

    /// `getFluidState`: always still water.
    fn get_fluid_state(_block: &BlockData, _state: BlockState) -> FluidState {
        Fluids::WATER.get_source(false)
    }
}

impl BubbleColumnBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::empty()
    }

    fn get_render_shape(_block: &BlockData, _state: BlockState) -> RenderShape {
        RenderShape::INVISIBLE
    }
}

impl BubbleColumnBlock {
    /// `animateTick`: its current or bubbles, always shown, and its sound now and then.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let x = f64::from(pos.get_x());
        let y = f64::from(pos.get_y());
        let z = f64::from(pos.get_z());
        if state.get_value(BubbleColumnBlock::DRAG_DOWN) {
            level.add_always_visible_particle(
                ParticleTypes::CURRENT_DOWN,
                x + 0.5,
                y + 0.8,
                z,
                0.0,
                0.0,
                0.0,
            );
            if random.next_int_bounded(200) == 0 {
                let volume = 0.2 + random.next_float() * 0.2;
                let pitch = 0.9 + random.next_float() * 0.15;
                level.play_local_sound(
                    x,
                    y,
                    z,
                    SoundEvents::BUBBLE_COLUMN_WHIRLPOOL_AMBIENT,
                    SoundSource::BLOCKS,
                    volume,
                    pitch,
                    false,
                );
            }
        } else {
            level.add_always_visible_particle(
                ParticleTypes::BUBBLE_COLUMN_UP,
                x + 0.5,
                y,
                z + 0.5,
                0.0,
                0.04,
                0.0,
            );
            let bx = x + f64::from(random.next_float());
            let by = y + f64::from(random.next_float());
            let bz = z + f64::from(random.next_float());
            level.add_always_visible_particle(
                ParticleTypes::BUBBLE_COLUMN_UP,
                bx,
                by,
                bz,
                0.0,
                0.04,
                0.0,
            );
            if random.next_int_bounded(200) == 0 {
                let volume = 0.2 + random.next_float() * 0.2;
                let pitch = 0.9 + random.next_float() * 0.15;
                level.play_local_sound(
                    x,
                    y,
                    z,
                    SoundEvents::BUBBLE_COLUMN_UPWARDS_AMBIENT,
                    SoundSource::BLOCKS,
                    volume,
                    pitch,
                    false,
                );
            }
        }
    }
}
