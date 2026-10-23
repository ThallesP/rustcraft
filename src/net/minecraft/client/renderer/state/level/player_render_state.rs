//! net/minecraft/client/renderer/state/level/PlayerRenderState.java
//!
//! What the frame draws of the player it's seen by: its avatar's state and its hands. The screen
//! effects' parts (the portal and nausea effects, the block and water overlays, fire, the item
//! activation) come with `ScreenEffectRenderer`.

use crate::net::minecraft::client::renderer::entity::state::avatar_render_state::AvatarRenderState;
use crate::net::minecraft::client::renderer::state::level::first_person_hands_and_items_render_state::FirstPersonHandsAndItemsRenderState;

#[derive(Clone, Debug, Default)]
pub struct PlayerRenderState {
    pub has_player: bool,
    pub avatar_render_state: Option<AvatarRenderState>,
    pub first_person_hands_and_items: FirstPersonHandsAndItemsRenderState,
    pub is_under_water: bool,
    pub eye_position_y: f64,
}

impl PlayerRenderState {
    pub fn reset(&mut self) {
        self.has_player = false;
        self.avatar_render_state = None;
    }
}
