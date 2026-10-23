//! net/minecraft/client/renderer/fog/environment/PowderedSnowFogEnvironment.java
//!
//! The fog with the camera in powder snow: white and close, farther for a spectator.

use crate::net::minecraft::client::renderer::fog::fog_data::FogData;
use crate::net::minecraft::util::argb::ARGB;
use crate::org::joml::vector3f::Vector3f;

pub struct PowderedSnowFogEnvironment;

impl PowderedSnowFogEnvironment {
    pub fn get_base_color() -> Vector3f {
        ARGB::vector3f_from_rgb24(-6308916)
    }

    pub fn setup_fog(fog: &mut FogData, render_distance: f32, is_spectator: bool) {
        if is_spectator {
            fog.environmental_start = -8.0;
            fog.environmental_end = render_distance * 0.5;
        } else {
            fog.environmental_start = 0.0;
            fog.environmental_end = 2.0;
        }

        fog.sky_end = fog.environmental_end;
        fog.cloud_end = fog.environmental_end;
    }
}
