//! net/minecraft/client/renderer/entity/state/AvatarRenderState.java, with what it inherits from
//! `HumanoidRenderState`, `ArmedEntityRenderState`, `LivingEntityRenderState` and
//! `EntityRenderState`.
//!
//! Only what the first-person hand reads so far: the skin and its sleeves, the main arm, the
//! swing, the pitch, the light and whether the player is invisible. The rest of a player's
//! state (its pose, its walk, its held items, its name tag, its layers') comes with entity
//! rendering.

use crate::net::minecraft::client::resources::default_player_skin::DefaultPlayerSkin;
use crate::net::minecraft::world::entity::humanoid_arm::HumanoidArm;
use crate::net::minecraft::world::entity::player::player_skin::PlayerSkin;

#[derive(Clone, Debug)]
pub struct AvatarRenderState {
    pub skin: PlayerSkin,
    pub show_hat: bool,
    pub show_jacket: bool,
    pub show_left_pants: bool,
    pub show_right_pants: bool,
    pub show_left_sleeve: bool,
    pub show_right_sleeve: bool,
    pub main_arm: HumanoidArm,
    pub swing_animation: f32,
    pub x_rot: f32,
    pub is_invisible: bool,
    pub light_coords: i32,
}

impl Default for AvatarRenderState {
    fn default() -> AvatarRenderState {
        AvatarRenderState {
            skin: DefaultPlayerSkin::get_default_skin(),
            show_hat: true,
            show_jacket: true,
            show_left_pants: true,
            show_right_pants: true,
            show_left_sleeve: true,
            show_right_sleeve: true,
            main_arm: HumanoidArm::RIGHT,
            swing_animation: 0.0,
            x_rot: 0.0,
            is_invisible: false,
            light_coords: 15728880,
        }
    }
}
