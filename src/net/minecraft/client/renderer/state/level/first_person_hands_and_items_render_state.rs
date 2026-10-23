//! net/minecraft/client/renderer/state/level/FirstPersonHandsAndItemsRenderState.java
//!
//! The hands as the frame draws them. The player holds nothing in the port yet (its inventory
//! isn't ported), so the held items, their render states, their use and charge durations and the
//! maps' states aren't here: both hands are empty, which `FirstPersonHandsAndItemsRenderer` draws
//! as the main hand's arm.

use crate::net::minecraft::world::interaction_hand::InteractionHand;

#[derive(Clone, Debug)]
pub struct FirstPersonHandsAndItemsRenderState {
    pub attack_hand: InteractionHand,
    pub view_x_rot: f32,
    pub view_y_rot: f32,
    pub x_bob: f32,
    pub y_bob: f32,
    pub is_scoping: bool,
    pub use_item_remaining_ticks: i32,
    pub main_hand_swap_scale: f32,
    pub off_hand_swap_scale: f32,
    pub hand_render_selection: Option<HandRenderSelection>,
    pub main_hand_height: f32,
    pub old_main_hand_height: f32,
    pub off_hand_height: f32,
    pub old_off_hand_height: f32,
}

impl Default for FirstPersonHandsAndItemsRenderState {
    fn default() -> FirstPersonHandsAndItemsRenderState {
        FirstPersonHandsAndItemsRenderState {
            attack_hand: InteractionHand::MAIN_HAND,
            view_x_rot: 0.0,
            view_y_rot: 0.0,
            x_bob: 0.0,
            y_bob: 0.0,
            is_scoping: false,
            use_item_remaining_ticks: 0,
            main_hand_swap_scale: 0.0,
            off_hand_swap_scale: 0.0,
            hand_render_selection: None,
            main_hand_height: 0.0,
            old_main_hand_height: 0.0,
            off_hand_height: 0.0,
            old_off_hand_height: 0.0,
        }
    }
}

/// `FirstPersonHandsAndItemsRenderState.HandRenderSelection`: which hands draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandRenderSelection {
    RENDER_BOTH_HANDS,
    RENDER_MAIN_HAND_ONLY,
    RENDER_OFF_HAND_ONLY,
}

impl HandRenderSelection {
    pub fn render_main_hand(&self) -> bool {
        matches!(
            self,
            HandRenderSelection::RENDER_BOTH_HANDS | HandRenderSelection::RENDER_MAIN_HAND_ONLY
        )
    }

    pub fn render_off_hand(&self) -> bool {
        matches!(
            self,
            HandRenderSelection::RENDER_BOTH_HANDS | HandRenderSelection::RENDER_OFF_HAND_ONLY
        )
    }

    pub fn only_for_hand(hand: InteractionHand) -> HandRenderSelection {
        if hand == InteractionHand::MAIN_HAND {
            HandRenderSelection::RENDER_MAIN_HAND_ONLY
        } else {
            HandRenderSelection::RENDER_OFF_HAND_ONLY
        }
    }
}
