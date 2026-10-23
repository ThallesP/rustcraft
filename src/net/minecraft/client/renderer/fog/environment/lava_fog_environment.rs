//! net/minecraft/client/renderer/fog/environment/LavaFogEnvironment.java
//!
//! The fog with the camera in lava: orange and close, farther for a spectator. Mob effects aren't
//! ported, so fire resistance never pushes it back.

use crate::net::minecraft::client::renderer::fog::fog_data::FogData;
use crate::net::minecraft::util::argb::ARGB;
use crate::org::joml::vector3f::Vector3f;

pub struct LavaFogEnvironment;

impl LavaFogEnvironment {
    pub fn get_base_color() -> Vector3f {
        ARGB::vector3f_from_rgb24(-6743808)
    }

    pub fn setup_fog(fog: &mut FogData, render_distance: f32, is_spectator: bool) {
        if is_spectator {
            fog.environmental_start = -8.0;
            fog.environmental_end = render_distance * 0.5;
        } else {
            fog.environmental_start = 0.25;
            fog.environmental_end = 1.0;
        }

        fog.sky_end = fog.environmental_end;
        fog.cloud_end = fog.environmental_end;
    }
}
