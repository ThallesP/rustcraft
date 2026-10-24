//! net/minecraft/world/level/block/PotentSulfurBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its bubbles and sound (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::potent_sulfur_state::PotentSulfurState;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct PotentSulfurBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(PotentSulfurBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.PotentSulfurBlock",
        &base_entity_block::CLASS,
    )
};

impl PotentSulfurBlock {
    pub const STATE: &'static EnumProperty<PotentSulfurState> =
        BlockStateProperties::POTENT_SULFUR_STATE;

    pub fn new(properties: Properties) -> BlockData {
        PotentSulfurBlock::new_with(
            properties,
            &CLASS,
            PotentSulfurBlock::create_block_state_definition,
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
                .default_block_state()
                .set_value(PotentSulfurBlock::STATE, PotentSulfurState::DRY),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[PotentSulfurBlock::STATE]);
    }
}

impl PotentSulfurBlock {
    /// `animateTick`: wet and under still water, bubbles, always shown, and its sound now and then.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if state.get_value(PotentSulfurBlock::STATE) != PotentSulfurState::DRY
            && level
                .get_fluid_state(&pos.above())
                .is_source_of_type(Fluids::WATER)
        {
            PotentSulfurBlock::spawn_bubble_particles_at(
                level,
                random,
                f64::from(pos.get_x()),
                f64::from(pos.get_y() + 1),
                f64::from(pos.get_z()),
            );
            PotentSulfurBlock::spawn_bubble_particles_at(
                level,
                random,
                f64::from(pos.get_x()),
                f64::from(pos.get_y() + 1),
                f64::from(pos.get_z()),
            );
            if random.next_int_bounded(10) == 0 {
                level.play_local_sound(
                    f64::from(pos.get_x()),
                    f64::from(pos.get_y()),
                    f64::from(pos.get_z()),
                    SoundEvents::NOXIOUS_GAS,
                    SoundSource::AMBIENT,
                    1.0,
                    1.0,
                    false,
                );
            }
        }
    }

    fn spawn_bubble_particles_at(
        level: &LevelWithEffects<'_>,
        random: &mut dyn RandomSource,
        x: f64,
        y: f64,
        z: f64,
    ) {
        let bx = x + f64::from(random.next_float());
        let by = y + f64::from(random.next_float());
        let bz = z + f64::from(random.next_float());
        level.add_always_visible_particle(ParticleTypes::SULFUR_BUBBLES, bx, by, bz, 0.0, 0.0, 0.0);
    }
}
