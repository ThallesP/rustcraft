//! net/minecraft/client/player/FirstPersonHandsAndItems.java
//!
//! The local player's hands as the first person sees them: how far each is raised (they lower
//! while an item is swapped and rise back as the attack recharges), ticked with the player, and
//! the state each frame draws them from. The player holds nothing in the port yet (its inventory
//! isn't ported), so the hands' items are always empty: they never swap (`itemUsed`, the use
//! durations, the item models and the maps come with items), and both hands are drawn.

use crate::net::minecraft::client::renderer::state::level::first_person_hands_and_items_render_state::{
    FirstPersonHandsAndItemsRenderState, HandRenderSelection,
};
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::player::player::Player;
use crate::net::minecraft::world::interaction_hand::InteractionHand;

#[derive(Clone, Debug, Default)]
pub struct FirstPersonHandsAndItems {
    main_hand_height: f32,
    o_main_hand_height: f32,
    off_hand_height: f32,
    o_off_hand_height: f32,
}

impl FirstPersonHandsAndItems {
    /// Each hand towards its height: the main hand's the cube of the swap's progress, the off
    /// hand's full, at most 0.4 a tick; both lower while the hands are busy (riding, not ported).
    pub fn tick(&mut self, player: &Player, hands_busy: bool) {
        self.o_main_hand_height = self.main_hand_height;
        self.o_off_hand_height = self.off_hand_height;
        if hands_busy {
            self.main_hand_height = Mth::clamp_f32(self.main_hand_height - 0.4, 0.0, 1.0);
            self.off_hand_height = Mth::clamp_f32(self.off_hand_height - 0.4, 0.0, 1.0);
        } else {
            let attack_anim = player.get_item_swap_scale(1.0);
            // The held items stay the same (empty), so neither hand's target drops to 0.
            let main_hand_target_height = attack_anim * attack_anim * attack_anim;
            let off_hand_target_height = 1.0;
            self.main_hand_height +=
                Mth::clamp_f32(main_hand_target_height - self.main_hand_height, -0.4, 0.4);
            self.off_hand_height +=
                Mth::clamp_f32(off_hand_target_height - self.off_hand_height, -0.4, 0.4);
        }
    }

    /// `extractRenderState`: the swing's hand (the main hand, without a swing), the view's
    /// rotation and its lag (`xBob`, `yBob`), and the hands' heights. Empty hands have no item
    /// model, whose swap animation scale is 1.
    #[allow(clippy::too_many_arguments)]
    pub fn extract_render_state(
        &self,
        view_x_rot: f32,
        view_y_rot: f32,
        x_bob: f32,
        y_bob: f32,
        state: &mut FirstPersonHandsAndItemsRenderState,
    ) {
        state.attack_hand = InteractionHand::MAIN_HAND;
        state.view_x_rot = view_x_rot;
        state.view_y_rot = view_y_rot;
        state.x_bob = x_bob;
        state.y_bob = y_bob;
        state.is_scoping = false;
        state.use_item_remaining_ticks = 0;
        state.hand_render_selection =
            Some(FirstPersonHandsAndItems::evaluate_which_hands_to_render());
        state.main_hand_height = self.main_hand_height;
        state.old_main_hand_height = self.o_main_hand_height;
        state.off_hand_height = self.off_hand_height;
        state.old_off_hand_height = self.o_off_hand_height;
        state.main_hand_swap_scale = 1.0;
        state.off_hand_swap_scale = 1.0;
    }

    /// Both hands, for a player holding neither a bow nor a crossbow.
    pub fn evaluate_which_hands_to_render() -> HandRenderSelection {
        HandRenderSelection::RENDER_BOTH_HANDS
    }
}
