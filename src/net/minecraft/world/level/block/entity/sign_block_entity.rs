//! net/minecraft/world/level/block/entity/SignBlockEntity.java, and HangingSignBlockEntity.java
//! (the same class of another type)
//!
//! A sign's two sides and whether it's waxed, as `loadAdditional` reads them; the client keeps
//! each side's lines as they came (`resolveLines` only changes them on a server). Not ported yet:
//! `allow_op_features` (which the server reads), the editing (`updateSignText`, the player
//! editing it), `executeClickCommandsIfPresent`, and the line heights and widths the renderer and
//! the screen read. Its ticker runs on both sides (`tick`), and does nothing on the client: only a
//! server sets the player who may edit it, whose editing it times out.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::entity::sign_text::SignText;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

#[derive(Clone, Debug, PartialEq)]
pub struct SignBlockEntity {
    front_text: SignText,
    back_text: SignText,
    is_waxed: bool,
}

impl SignBlockEntity {
    /// `tick`: the player who may edit it is let go of once too far away; a client's sign has
    /// none.
    pub fn tick(_level: &mut LevelTicking<'_>, _pos: &BlockPos, _state: BlockState) {}

    pub fn new() -> SignBlockEntity {
        SignBlockEntity {
            front_text: SignText::empty(),
            back_text: SignText::empty(),
            is_waxed: false,
        }
    }

    pub fn load_additional(&mut self, input: &TagValueInput<'_>) {
        self.front_text = input
            .read("front_text", SignText::CODEC)
            .unwrap_or_else(SignText::empty);
        self.back_text = input
            .read("back_text", SignText::CODEC)
            .unwrap_or_else(SignText::empty);
        self.is_waxed = input.get_boolean_or("is_waxed", false);
    }

    pub fn get_front_text(&self) -> &SignText {
        &self.front_text
    }

    pub fn get_back_text(&self) -> &SignText {
        &self.back_text
    }

    /// `getText(isFrontText)`.
    pub fn get_text(&self, is_front_text: bool) -> &SignText {
        if is_front_text {
            &self.front_text
        } else {
            &self.back_text
        }
    }

    pub fn is_waxed(&self) -> bool {
        self.is_waxed
    }
}

impl Default for SignBlockEntity {
    fn default() -> SignBlockEntity {
        SignBlockEntity::new()
    }
}
