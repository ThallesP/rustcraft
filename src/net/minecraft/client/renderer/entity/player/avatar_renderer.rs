//! net/minecraft/client/renderer/entity/player/AvatarRenderer.java
//!
//! A player-shaped entity's renderer, with its model, wide or slim. Only the first-person hands
//! (`renderRightHand`, `renderLeftHand`) and what they read of the player are ported
//! (`extract_render_state`'s part of `extractRenderState`, `getPackedLightCoords`); drawing the
//! entity itself, its layers (armor, held items, arrows, ears, cape, head) and its name tag come
//! with entity rendering.

use crate::com::mojang::blaze3d::vertex::pose_stack::PoseStack;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::client::model::geom::builders::cube_deformation::CubeDeformation;
use crate::net::minecraft::client::model::geom::builders::layer_definition::LayerDefinition;
use crate::net::minecraft::client::model::player::player_model::PlayerModel;
use crate::net::minecraft::client::renderer::entity::state::avatar_render_state::AvatarRenderState;
use crate::net::minecraft::client::renderer::rendertype::render_types::RenderTypes;
use crate::net::minecraft::client::renderer::submit_node_storage::SubmitNodeStorage;
use crate::net::minecraft::client::renderer::texture::overlay_texture::OverlayTexture;
use crate::net::minecraft::client::resources::default_player_skin::DefaultPlayerSkin;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::light_coords_util::LightCoordsUtil;
use crate::net::minecraft::world::entity::humanoid_arm::HumanoidArm;
use crate::net::minecraft::world::entity::player::player_model_part::PlayerModelPart;
use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
use crate::net::minecraft::world::level::light_layer::LightLayer;
use crate::net::minecraft::world::phys::vec3::Vec3;

pub struct AvatarRenderer {
    model: PlayerModel,
}

/// What `extract_render_state` reads of a player.
pub struct AvatarEntity<'a> {
    pub uuid: UUID,
    /// `getEyePosition(partialTicks)`: `getLightProbePosition`.
    pub light_probe_position: Vec3,
    pub x_rot: f32,
    pub main_arm: HumanoidArm,
    /// `DATA_PLAYER_MODE_CUSTOMISATION`.
    pub model_customisation: i8,
    pub is_invisible: bool,
    pub is_on_fire: bool,
    pub level: &'a dyn BlockAndLightGetter,
}

impl AvatarEntity<'_> {
    /// `Avatar.isModelPartShown`.
    pub fn is_model_part_shown(&self, part: PlayerModelPart) -> bool {
        (i32::from(self.model_customisation) & part.get_mask()) == part.get_mask()
    }
}

impl AvatarRenderer {
    /// The renderer of slim or wide skins, its model baked from `ModelLayers.PLAYER_SLIM` or
    /// `PLAYER` (`LayerDefinitions`: the player's mesh on a 64 by 64 texture).
    pub fn new(slim_steve: bool) -> AvatarRenderer {
        let layer = LayerDefinition::create(
            PlayerModel::create_mesh(CubeDeformation::NONE, slim_steve),
            64,
            64,
        );
        AvatarRenderer {
            model: PlayerModel::new(layer.bake_root(), slim_steve),
        }
    }

    pub fn get_model(&self) -> &PlayerModel {
        &self.model
    }

    /// `extractRenderState`'s part the hands read: the skin (`AbstractClientPlayer.getSkin`, the
    /// default skin for the player's id until the player list's are ported, which a player without
    /// textures has too), its sleeves, the main arm, the pitch, invisibility and the light. The
    /// swing (`LivingEntity.SwingState`, which attacks and uses start, not ported yet) is none.
    pub fn extract_render_state(&self, entity: &AvatarEntity<'_>, state: &mut AvatarRenderState) {
        state.light_coords = AvatarRenderer::get_packed_light_coords(entity);
        state.is_invisible = entity.is_invisible;
        state.x_rot = entity.x_rot;
        state.main_arm = entity.main_arm;
        state.swing_animation = 0.0;
        state.skin = DefaultPlayerSkin::get(&entity.uuid);
        state.show_hat = entity.is_model_part_shown(PlayerModelPart::HAT);
        state.show_jacket = entity.is_model_part_shown(PlayerModelPart::JACKET);
        state.show_left_pants = entity.is_model_part_shown(PlayerModelPart::LEFT_PANTS_LEG);
        state.show_right_pants = entity.is_model_part_shown(PlayerModelPart::RIGHT_PANTS_LEG);
        state.show_left_sleeve = entity.is_model_part_shown(PlayerModelPart::LEFT_SLEEVE);
        state.show_right_sleeve = entity.is_model_part_shown(PlayerModelPart::RIGHT_SLEEVE);
    }

    /// `EntityRenderer.getPackedLightCoords`: the block and sky light where the player's eyes are
    /// (full block light when it's on fire).
    pub fn get_packed_light_coords(entity: &AvatarEntity<'_>) -> i32 {
        let block_pos = BlockPos::containing(&entity.light_probe_position);
        let block_light = if entity.is_on_fire {
            15
        } else {
            entity.level.get_brightness(LightLayer::BLOCK, &block_pos)
        };
        let sky_light = entity.level.get_brightness(LightLayer::SKY, &block_pos);
        LightCoordsUtil::pack(block_light, sky_light)
    }

    pub fn render_right_hand(
        &mut self,
        pose_stack: &PoseStack,
        submit_node_collector: &mut SubmitNodeStorage,
        light_coords: i32,
        skin_texture: &Identifier,
        has_sleeve: bool,
    ) {
        self.render_hand(
            pose_stack,
            submit_node_collector,
            light_coords,
            skin_texture,
            HumanoidArm::RIGHT,
            has_sleeve,
        );
    }

    pub fn render_left_hand(
        &mut self,
        pose_stack: &PoseStack,
        submit_node_collector: &mut SubmitNodeStorage,
        light_coords: i32,
        skin_texture: &Identifier,
        has_sleeve: bool,
    ) {
        self.render_hand(
            pose_stack,
            submit_node_collector,
            light_coords,
            skin_texture,
            HumanoidArm::LEFT,
            has_sleeve,
        );
    }

    /// The arm at its initial pose, tilted a tenth of a radian outwards, with its sleeve if the
    /// player shows it, submitted in the skin's translucent render type.
    fn render_hand(
        &mut self,
        pose_stack: &PoseStack,
        submit_node_collector: &mut SubmitNodeStorage,
        light_coords: i32,
        skin_texture: &Identifier,
        arm: HumanoidArm,
        has_sleeve: bool,
    ) {
        let model = &mut self.model;
        let arm_part = match arm {
            HumanoidArm::RIGHT => model.right_arm_mut(),
            HumanoidArm::LEFT => model.left_arm_mut(),
        };
        arm_part.reset_pose();
        arm_part.visible = true;
        model.left_sleeve_mut().visible = has_sleeve;
        model.right_sleeve_mut().visible = has_sleeve;
        model.left_arm_mut().z_rot = -0.1;
        model.right_arm_mut().z_rot = 0.1;
        let arm_part = match arm {
            HumanoidArm::RIGHT => model.right_arm(),
            HumanoidArm::LEFT => model.left_arm(),
        };
        submit_node_collector.submit_model_part(
            arm_part,
            pose_stack,
            &RenderTypes::entity_translucent(skin_texture),
            light_coords,
            OverlayTexture::NO_OVERLAY,
        );
    }
}
