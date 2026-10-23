//! net/minecraft/client/renderer/FirstPersonHandsAndItemsRenderer.java
//!
//! The hands and held items in front of the first person. The player holds nothing in the port
//! yet (its inventory isn't ported), so a hand is only ever empty: the main hand draws the
//! player's arm (`renderPlayerArm`), swaying with the view's lag and raised by its height, and an
//! empty off hand draws nothing, like Java's. The items' paths (maps, crossbows, bows, eating,
//! brushing, spears, the spin attack and the swing animations of held items) come with items.

use crate::com::mojang::blaze3d::vertex::pose_stack::PoseStack;
use crate::com::mojang::math::axis::Axis;
use crate::net::minecraft::client::renderer::entity::entity_render_dispatcher::EntityRenderDispatcher;
use crate::net::minecraft::client::renderer::state::level::player_render_state::PlayerRenderState;
use crate::net::minecraft::client::renderer::submit_node_storage::SubmitNodeStorage;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::humanoid_arm::HumanoidArm;
use crate::net::minecraft::world::interaction_hand::InteractionHand;

/// `(float) Math.PI`.
const PI: f32 = std::f64::consts::PI as f32;
/// `(float) (Math.PI * 2)`.
const TWO_PI: f32 = (std::f64::consts::PI * 2.0) as f32;

pub struct FirstPersonHandsAndItemsRenderer;

impl FirstPersonHandsAndItemsRenderer {
    /// The player's arm, drawn by its avatar's renderer with its skin, its sleeve if it shows it.
    fn render_player_hand(
        pose_stack: &PoseStack,
        submit_node_collector: &mut SubmitNodeStorage,
        light_coords: i32,
        arm: HumanoidArm,
        player_state: &PlayerRenderState,
        entity_render_dispatcher: &mut EntityRenderDispatcher,
    ) {
        let Some(avatar_render_state) = &player_state.avatar_render_state else {
            return;
        };
        let avatar_renderer = entity_render_dispatcher.get_renderer(avatar_render_state);
        let skin_texture = avatar_render_state.skin.body.texture_path().clone();
        if arm == HumanoidArm::RIGHT {
            avatar_renderer.render_right_hand(
                pose_stack,
                submit_node_collector,
                light_coords,
                &skin_texture,
                avatar_render_state.show_right_sleeve,
            );
        } else {
            avatar_renderer.render_left_hand(
                pose_stack,
                submit_node_collector,
                light_coords,
                &skin_texture,
                avatar_render_state.show_left_sleeve,
            );
        }
    }

    /// The arm swung by `attack_value` and lowered by `inverse_arm_height`, turned to come in
    /// from the screen's side.
    #[allow(clippy::too_many_arguments)]
    fn render_player_arm(
        pose_stack: &mut PoseStack,
        submit_node_collector: &mut SubmitNodeStorage,
        light_coords: i32,
        inverse_arm_height: f32,
        attack_value: f32,
        arm: HumanoidArm,
        player_state: &PlayerRenderState,
        entity_render_dispatcher: &mut EntityRenderDispatcher,
    ) {
        let is_right_arm = arm != HumanoidArm::LEFT;
        let invert = if is_right_arm { 1.0 } else { -1.0 };
        let sqrt_attack_value = Mth::sqrt(attack_value);
        let x_swing_position = -0.3 * Mth::sin(f64::from(sqrt_attack_value * PI));
        let y_swing_position = 0.4 * Mth::sin(f64::from(sqrt_attack_value * TWO_PI));
        let z_swing_position = -0.4 * Mth::sin(f64::from(attack_value * PI));
        pose_stack.translate_f32(
            invert * (x_swing_position + 0.64000005),
            y_swing_position + -0.6 + inverse_arm_height * -0.6,
            z_swing_position + -0.71999997,
        );
        pose_stack.rotate_degrees(Axis::YP, invert * 45.0);
        let z_swing_rotation = Mth::sin(f64::from(attack_value * attack_value * PI));
        let y_swing_rotation = Mth::sin(f64::from(sqrt_attack_value * PI));
        pose_stack.rotate_degrees(Axis::YP, invert * y_swing_rotation * 70.0);
        pose_stack.rotate_degrees(Axis::ZP, invert * z_swing_rotation * -20.0);
        // Java's `invert * -1.0F`.
        pose_stack.translate_f32(-invert, 3.6, 3.5);
        pose_stack.rotate_degrees(Axis::ZP, invert * 120.0);
        pose_stack.rotate_degrees(Axis::XP, 200.0);
        pose_stack.rotate_degrees(Axis::YP, invert * -135.0);
        pose_stack.translate_f32(invert * 5.6, 0.0, 0.0);
        FirstPersonHandsAndItemsRenderer::render_player_hand(
            pose_stack,
            submit_node_collector,
            light_coords,
            arm,
            player_state,
            entity_render_dispatcher,
        );
    }

    /// `submitHandsWithItems`: the view's lag as a sway, then each hand the state draws.
    pub fn submit_hands_with_items(
        partial_ticks: f32,
        pose_stack: &mut PoseStack,
        submit_node_collector: &mut SubmitNodeStorage,
        player_state: &PlayerRenderState,
        entity_render_dispatcher: &mut EntityRenderDispatcher,
    ) {
        let state = &player_state.first_person_hands_and_items;
        let (Some(avatar_render_state), Some(hand_render_selection)) = (
            &player_state.avatar_render_state,
            state.hand_render_selection,
        ) else {
            return;
        };
        let attack_value = avatar_render_state.swing_animation;
        let attack_hand = state.attack_hand;
        pose_stack.rotate_degrees(Axis::XP, (state.view_x_rot - state.x_bob) * 0.1);
        pose_stack.rotate_degrees(Axis::YP, (state.view_y_rot - state.y_bob) * 0.1);
        let light_coords = avatar_render_state.light_coords;
        if hand_render_selection.render_main_hand() {
            let main_hand_attack = if attack_hand == InteractionHand::MAIN_HAND {
                attack_value
            } else {
                0.0
            };
            let mainhand_inverse_arm_height = state.main_hand_swap_scale
                * (1.0
                    - Mth::lerp_f32(
                        partial_ticks,
                        state.old_main_hand_height,
                        state.main_hand_height,
                    ));
            FirstPersonHandsAndItemsRenderer::submit_arm_with_item(
                player_state,
                InteractionHand::MAIN_HAND,
                main_hand_attack,
                mainhand_inverse_arm_height,
                pose_stack,
                submit_node_collector,
                light_coords,
                entity_render_dispatcher,
            );
        }

        if hand_render_selection.render_off_hand() {
            let off_hand_attack = if attack_hand == InteractionHand::OFF_HAND {
                attack_value
            } else {
                0.0
            };
            let offhand_inverse_arm_height = state.off_hand_swap_scale
                * (1.0
                    - Mth::lerp_f32(
                        partial_ticks,
                        state.old_off_hand_height,
                        state.off_hand_height,
                    ));
            FirstPersonHandsAndItemsRenderer::submit_arm_with_item(
                player_state,
                InteractionHand::OFF_HAND,
                off_hand_attack,
                offhand_inverse_arm_height,
                pose_stack,
                submit_node_collector,
                light_coords,
                entity_render_dispatcher,
            );
        }
    }

    /// An empty hand: the main hand's arm, unless the player is invisible; nothing for the off
    /// hand.
    #[allow(clippy::too_many_arguments)]
    fn submit_arm_with_item(
        player_state: &PlayerRenderState,
        hand: InteractionHand,
        attack: f32,
        inverse_arm_height: f32,
        pose_stack: &mut PoseStack,
        submit_node_collector: &mut SubmitNodeStorage,
        light_coords: i32,
        entity_render_dispatcher: &mut EntityRenderDispatcher,
    ) {
        if player_state.first_person_hands_and_items.is_scoping {
            return;
        }
        let Some(avatar_render_state) = &player_state.avatar_render_state else {
            return;
        };
        let is_main_hand = hand == InteractionHand::MAIN_HAND;
        let arm = if is_main_hand {
            avatar_render_state.main_arm
        } else {
            avatar_render_state.main_arm.get_opposite()
        };
        pose_stack.push_pose();
        if is_main_hand && !avatar_render_state.is_invisible {
            FirstPersonHandsAndItemsRenderer::render_player_arm(
                pose_stack,
                submit_node_collector,
                light_coords,
                inverse_arm_height,
                attack,
                arm,
                player_state,
                entity_render_dispatcher,
            );
        }

        pose_stack.pop_pose();
    }
}
