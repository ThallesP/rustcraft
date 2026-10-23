//! net/minecraft/world/level/block/entity/SkullBlockEntity.java
//!
//! A head's note block sound and custom name, as `loadAdditional` reads them. Not ported yet: its
//! owner's profile (`profile`, `ResolvableProfile.CODEC`: a player head's skin), which comes with
//! skins, and the animation the dragon head and piglin head tick (`animation`, `isAnimating`).

use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SkullBlockEntity {
    note_block_sound: Option<Identifier>,
    custom_name: Option<Component>,
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
}
