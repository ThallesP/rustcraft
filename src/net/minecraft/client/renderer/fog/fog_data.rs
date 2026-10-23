//! net/minecraft/client/renderer/fog/FogData.java

use crate::org::joml::vector4f::Vector4f;

/// A frame's fog: where it starts and ends, for the environment (the atmosphere, water…), the
/// render distance, the sky and the clouds, and its colour.
#[derive(Clone, Debug, Default)]
pub struct FogData {
    pub environmental_start: f32,
    pub render_distance_start: f32,
    pub environmental_end: f32,
    pub render_distance_end: f32,
    pub sky_end: f32,
    pub cloud_end: f32,
    pub color: Vector4f,
}
