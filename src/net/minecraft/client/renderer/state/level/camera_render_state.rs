//! net/minecraft/client/renderer/state/level/CameraRenderState.java
//!
//! The camera entity's state (`entityRenderState`) has its hurt and death so far, which the game
//! renderer fills in from the player. Not ported yet: the panoramic mode.

use crate::net::minecraft::client::renderer::culling::frustum::Frustum;
use crate::net::minecraft::client::renderer::fog::fog_data::FogData;
use crate::net::minecraft::client::renderer::state::level::camera_entity_render_state::CameraEntityRenderState;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::material::fog_type::FogType;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::quaternionf::Quaternionf;

/// The camera as the frame renders it, extracted from `Camera`.
#[derive(Clone)]
pub struct CameraRenderState {
    pub initialized: bool,
    pub camera_entity_partial_ticks: f32,
    pub is_frustum_captured: bool,
    pub is_first_person: bool,
    pub smart_cull: bool,
    pub pos: Vec3,
    pub x_rot: f32,
    pub y_rot: f32,
    pub block_pos: BlockPos,
    pub orientation: Quaternionf,
    pub cull_frustum: Frustum,
    pub depth_far: f32,
    pub projection_matrix: Matrix4f,
    pub view_rotation_matrix: Matrix4f,
    pub hud_fov: f32,
    pub fog_type: FogType,
    pub fog_data: FogData,
    pub entity_render_state: CameraEntityRenderState,
}

impl Default for CameraRenderState {
    fn default() -> CameraRenderState {
        CameraRenderState {
            initialized: false,
            camera_entity_partial_ticks: 0.0,
            is_frustum_captured: false,
            is_first_person: true,
            smart_cull: true,
            pos: Vec3::ZERO,
            x_rot: 0.0,
            y_rot: 0.0,
            block_pos: BlockPos::ZERO,
            orientation: Quaternionf::new(),
            cull_frustum: Frustum::new(&Matrix4f::new(), &Matrix4f::new()),
            depth_far: 0.0,
            projection_matrix: Matrix4f::new(),
            view_rotation_matrix: Matrix4f::new(),
            hud_fov: 0.0,
            fog_type: FogType::NONE,
            fog_data: FogData::default(),
            entity_render_state: CameraEntityRenderState::default(),
        }
    }
}
