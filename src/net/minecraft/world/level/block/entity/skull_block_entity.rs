//! net/minecraft/world/level/block/entity/SkullBlockEntity.java
//!
//! A head's note block sound and custom name, as `loadAdditional` reads them, and the animation
//! the client's ticker runs for a dragon's or a piglin's head while it's powered (`animation`).
//! Not ported yet: its owner's profile (`profile`, `ResolvableProfile.CODEC`: a player head's
//! skin), which comes with skins.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::level::block::abstract_skull_block::AbstractSkullBlock;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SkullBlockEntity {
    note_block_sound: Option<Identifier>,
    custom_name: Option<Component>,
    animation_tick_count: i32,
    is_animating: bool,
}

impl SkullBlockEntity {
    pub fn new() -> SkullBlockEntity {
        SkullBlockEntity::default()
    }

    pub fn load_additional(&mut self, input: &TagValueInput<'_>) {
        self.note_block_sound = input.read("note_block_sound", Identifier::CODEC);
        self.custom_name = input.read("custom_name", ComponentSerialization::CODEC);
    }

    pub fn get_note_block_sound(&self) -> Option<&Identifier> {
        self.note_block_sound.as_ref()
    }

    pub fn get_custom_name(&self) -> Option<&Component> {
        self.custom_name.as_ref()
    }

    /// `animation`, the client's ticker for the heads that move: a tick on while powered.
    pub fn animation(level: &mut LevelTicking<'_>, pos: &BlockPos, state: BlockState) {
        let mut block_entity = level.block_entity(pos);
        let BlockEntityData::Skull(entity) = &mut block_entity.data else {
            return;
        };
        if state.has_property(AbstractSkullBlock::POWERED)
            && state.get_value(AbstractSkullBlock::POWERED)
        {
            entity.is_animating = true;
            entity.animation_tick_count = entity.animation_tick_count.wrapping_add(1);
        } else {
            entity.is_animating = false;
        }
    }

    pub fn get_animation(&self, partial_tick: f32) -> f32 {
        if self.is_animating {
            self.animation_tick_count as f32 + partial_tick
        } else {
            self.animation_tick_count as f32
        }
    }

    pub fn animation_tick_count(&self) -> i32 {
        self.animation_tick_count
    }

    pub fn is_animating(&self) -> bool {
        self.is_animating
    }
}
