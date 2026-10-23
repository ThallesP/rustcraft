//! net/minecraft/client/renderer/fog/environment/AtmosphericFogEnvironment.java
//!
//! The fog in the open air: the environment's fog colour, toward the sunrise's when looking at
//! it, blended with the sky's by how far the sky's fog reaches, and the fog's distances, which
//! rain brings closer. The rain's fog needs the level's rain (and the biome's precipitation and
//! the camera's sky light), which aren't ported: the port's levels are clear, so the rain's fog is
//! none, and a rainy level panics.

use crate::java::lang::math::Math;
use crate::net::minecraft::client::camera::Camera;
use crate::net::minecraft::client::renderer::fog::fog_data::FogData;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::attribute::environment_attribute_layer::LayerLevel;
use crate::net::minecraft::world::attribute::environment_attribute_system::EnvironmentAttributeSystem;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::org::joml::vector3f::Vector3f;
use crate::org::joml::vector4f::Vector4f;

#[derive(Default)]
pub struct AtmosphericFogEnvironment {
    rain_fog_multiplier: f32,
}

impl AtmosphericFogEnvironment {
    pub fn get_base_color(
        &self,
        camera: &mut Camera,
        system: &EnvironmentAttributeSystem,
        level: &dyn LayerLevel,
        render_distance: i32,
        partial_ticks: f32,
    ) -> Vector3f {
        let forward_vector = *camera.forward_vector();
        let probe = camera.attribute_probe_mut();
        let mut fog_color = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::FOG_COLOR,
            partial_ticks,
        );
        if render_distance >= 4 {
            let sun_angle = probe.get_value(
                system,
                level,
                &EnvironmentAttributes::SUN_ANGLE,
                partial_ticks,
            ) * (std::f64::consts::PI / 180.0) as f32;
            let sun_x = if Mth::sin(f64::from(sun_angle)) > 0.0 {
                -1.0
            } else {
                1.0
            };
            let looking_at_the_sun_factor = forward_vector.dot_xyz(sun_x, 0.0, 0.0);
            if looking_at_the_sun_factor > 0.0 {
                let color: Vector4f = probe.get_value(
                    system,
                    level,
                    &EnvironmentAttributes::SUNRISE_SUNSET_COLOR,
                    partial_ticks,
                );
                let alpha = color.w;
                if alpha > 0.0 {
                    fog_color = ARGB::srgb_lerp_vector3f(
                        looking_at_the_sun_factor * alpha,
                        &fog_color,
                        &Vector3f::from_xyz(color.x, color.y, color.z),
                    );
                }
            }
        }

        let sky_color = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::SKY_COLOR,
            partial_ticks,
        );
        let sky_color = AtmosphericFogEnvironment::apply_weather_darken(
            sky_color,
            level.rain_level(),
            level.thunder_level(),
        );
        let sky_fog_end = Math::min_f32(
            probe.get_value(
                system,
                level,
                &EnvironmentAttributes::SKY_FOG_END_DISTANCE,
                partial_ticks,
            ) / 16.0,
            render_distance as f32,
        );
        let sky_color_mix_factor = Mth::clamped_lerp_f32(sky_fog_end / 32.0, 0.25, 1.0);
        let sky_color_mix_factor = 1.0 - Math::pow(f64::from(sky_color_mix_factor), 0.25) as f32;
        ARGB::srgb_lerp_vector3f(sky_color_mix_factor, &fog_color, &sky_color)
    }

    fn apply_weather_darken(mut color: Vector3f, rain_level: f32, thunder_level: f32) -> Vector3f {
        if rain_level > 0.0 {
            let rain_color_modifier = 1.0 - rain_level * 0.5;
            let rain_blue_color_modifier = 1.0 - rain_level * 0.4;
            color = ARGB::scale_rgb3_vector3f(
                &color,
                rain_color_modifier,
                rain_color_modifier,
                rain_blue_color_modifier,
            );
        }

        if thunder_level > 0.0 {
            color = ARGB::scale_rgb_vector3f(&color, 1.0 - thunder_level * 0.5);
        }

        color
    }

    /// The fog's distances; `cloud_range` is `options.cloudRange()`.
    #[allow(clippy::too_many_arguments)]
    pub fn setup_fog(
        &mut self,
        fog: &mut FogData,
        camera: &mut Camera,
        system: &EnvironmentAttributeSystem,
        level: &dyn LayerLevel,
        render_distance: f32,
        delta_ticks: f32,
        partial_ticks: f32,
        cloud_range: i32,
    ) {
        self.update_rain_fog_state(level, delta_ticks);
        let probe = camera.attribute_probe_mut();
        fog.environmental_start = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::FOG_START_DISTANCE,
            partial_ticks,
        );
        fog.environmental_end = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::FOG_END_DISTANCE,
            partial_ticks,
        );
        fog.environmental_start += -160.0 * self.rain_fog_multiplier;
        let min_rain_fog_end = Math::min_f32(96.0, fog.environmental_end);
        fog.environmental_end = Math::max_f32(
            min_rain_fog_end,
            fog.environmental_end + -256.0 * self.rain_fog_multiplier,
        );
        fog.sky_end = Math::min_f32(
            render_distance,
            probe.get_value(
                system,
                level,
                &EnvironmentAttributes::SKY_FOG_END_DISTANCE,
                partial_ticks,
            ),
        );
        fog.cloud_end = Math::min_f32(
            (cloud_range * 16) as f32,
            probe.get_value(
                system,
                level,
                &EnvironmentAttributes::CLOUD_FOG_END_DISTANCE,
                partial_ticks,
            ),
        );
    }

    /// The rain's fog eases toward the level's rain: none in a clear level.
    fn update_rain_fog_state(&mut self, level: &dyn LayerLevel, delta_ticks: f32) {
        assert!(
            level.rain_level() == 0.0,
            "the rain's fog (the biome's precipitation, the camera's sky light) isn't ported"
        );
        let target_rain_fog_multiplier = 0.0;
        self.rain_fog_multiplier +=
            (target_rain_fog_multiplier - self.rain_fog_multiplier) * delta_ticks * 0.2;
    }
}
