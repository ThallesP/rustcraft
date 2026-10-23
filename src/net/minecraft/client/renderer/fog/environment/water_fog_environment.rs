//! net/minecraft/client/renderer/fog/environment/WaterFogEnvironment.java
//!
//! The fog with the camera in water: the biome's water fog colour and distances, the end drawn in
//! while the player's eyes get used to the water (`getWaterVision`).

use crate::java::lang::math::Math;
use crate::net::minecraft::client::camera::Camera;
use crate::net::minecraft::client::renderer::fog::fog_data::FogData;
use crate::net::minecraft::world::attribute::environment_attribute_layer::LayerLevel;
use crate::net::minecraft::world::attribute::environment_attribute_system::EnvironmentAttributeSystem;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::org::joml::vector3f::Vector3f;

pub struct WaterFogEnvironment;

impl WaterFogEnvironment {
    /// `water_vision` is the camera's player's (`LocalPlayer.getWaterVision`), none for another
    /// entity.
    pub fn setup_fog(
        fog: &mut FogData,
        camera: &mut Camera,
        system: &EnvironmentAttributeSystem,
        level: &dyn LayerLevel,
        partial_ticks: f32,
        water_vision: Option<f32>,
    ) {
        let probe = camera.attribute_probe_mut();
        fog.environmental_start = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::WATER_FOG_START_DISTANCE,
            partial_ticks,
        );
        fog.environmental_end = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::WATER_FOG_END_DISTANCE,
            partial_ticks,
        );
        if let Some(water_vision) = water_vision {
            fog.environmental_end *= Math::max_f32(0.25, water_vision);
        }

        fog.sky_end = fog.environmental_end;
        fog.cloud_end = fog.environmental_end;
    }

    pub fn get_base_color(
        camera: &mut Camera,
        system: &EnvironmentAttributeSystem,
        level: &dyn LayerLevel,
        partial_ticks: f32,
    ) -> Vector3f {
        camera.attribute_probe_mut().get_value(
            system,
            level,
            &EnvironmentAttributes::WATER_FOG_COLOR,
            partial_ticks,
        )
    }
}
