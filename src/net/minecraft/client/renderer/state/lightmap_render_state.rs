//! net/minecraft/client/renderer/state/LightmapRenderState.java

use crate::org::joml::vector3f::Vector3f;

/// What the lightmap is drawn from, extracted each frame (`LightmapRenderStateExtractor`).
#[derive(Clone, Debug)]
pub struct LightmapRenderState {
    pub needs_update: bool,
    pub block_factor: f32,
    pub block_light_tint: Vector3f,
    pub sky_factor: f32,
    pub sky_light_color: Vector3f,
    pub ambient_color: Vector3f,
    pub brightness: f32,
    pub darkness_effect_scale: f32,
    pub night_vision_effect_intensity: f32,
    pub night_vision_color: Vector3f,
    pub boss_overlay_world_darkening: f32,
}

impl Default for LightmapRenderState {
    fn default() -> LightmapRenderState {
        let white = Vector3f::from_xyz(1.0, 1.0, 1.0);
        LightmapRenderState {
            needs_update: false,
            block_factor: 0.0,
            block_light_tint: white,
            sky_factor: 0.0,
            sky_light_color: white,
            ambient_color: white,
            brightness: 0.0,
            darkness_effect_scale: 0.0,
            night_vision_effect_intensity: 0.0,
            night_vision_color: white,
            boss_overlay_world_darkening: 0.0,
        }
    }
}
