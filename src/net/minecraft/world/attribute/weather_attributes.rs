//! net/minecraft/world/attribute/WeatherAttributes.java
//!
//! What rain and thunder do to the sky, the fog, the clouds and the light, each blended in by how
//! hard it rains or thunders. `WeatherAccess` is what `LayerLevel` answers of the weather.

use std::sync::LazyLock;

use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::world::attribute::attribute_type::AttributeValue;
use crate::net::minecraft::world::attribute::environment_attribute_layer::TimeBased;
use crate::net::minecraft::world::attribute::environment_attribute_map::EnvironmentAttributeMap;
use crate::net::minecraft::world::attribute::environment_attribute_system::Builder;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::attribute::modifier::attribute_modifier::AttributeModifier;
use crate::net::minecraft::world::attribute::modifier::color_modifier::{
    BlendToGray, ColorModifier,
};
use crate::net::minecraft::world::attribute::modifier::float_modifier::FloatModifier;
use crate::net::minecraft::world::attribute::modifier::float_with_alpha::FloatWithAlpha;
use crate::net::minecraft::world::timeline::timelines::Timelines;

pub struct WeatherAttributes;

static RAIN: LazyLock<EnvironmentAttributeMap> = LazyLock::new(|| {
    EnvironmentAttributeMap::builder()
        .modify(
            EnvironmentAttributes::SKY_COLOR,
            AttributeModifier::Color(ColorModifier::BLEND_TO_GRAY_RGB),
            AttributeValue::BlendToGray(BlendToGray::new(0.6, 0.75)),
        )
        .modify(
            EnvironmentAttributes::FOG_COLOR,
            AttributeModifier::Color(ColorModifier::MULTIPLY_RGB),
            AttributeValue::Rgb(ARGB::vector3f_from_rgb24(ARGB::color_from_float(
                1.0, 0.5, 0.5, 0.6,
            ))),
        )
        .modify(
            EnvironmentAttributes::CLOUD_COLOR,
            AttributeModifier::Color(ColorModifier::BLEND_TO_GRAY_ARGB),
            AttributeValue::BlendToGray(BlendToGray::new(0.24, 0.5)),
        )
        .modify(
            EnvironmentAttributes::SKY_LIGHT_LEVEL,
            AttributeModifier::Float(FloatModifier::ALPHA_BLEND),
            AttributeValue::FloatWithAlpha(FloatWithAlpha::new(4.0, 0.3125)),
        )
        .modify(
            EnvironmentAttributes::SKY_LIGHT_COLOR,
            AttributeModifier::Color(ColorModifier::ALPHA_BLEND_RGB),
            AttributeValue::Argb(ARGB::vector4f_from_argb32(ARGB::color_alpha_f32(
                0.3125,
                Timelines::night_sky_light_color(),
            ))),
        )
        .modify(
            EnvironmentAttributes::SKY_LIGHT_FACTOR,
            AttributeModifier::Float(FloatModifier::ALPHA_BLEND),
            AttributeValue::FloatWithAlpha(FloatWithAlpha::new(0.24, 0.3125)),
        )
        .set(
            EnvironmentAttributes::STAR_BRIGHTNESS,
            AttributeValue::Float(0.0),
        )
        .modify(
            EnvironmentAttributes::SUNRISE_SUNSET_COLOR,
            AttributeModifier::Color(ColorModifier::MULTIPLY_ARGB),
            AttributeValue::Argb(ARGB::vector4f_from_argb32(ARGB::color_from_float(
                1.0, 0.5, 0.5, 0.6,
            ))),
        )
        .set(
            EnvironmentAttributes::BEES_STAY_IN_HIVE,
            AttributeValue::Boolean(true),
        )
        .build()
});

static THUNDER: LazyLock<EnvironmentAttributeMap> = LazyLock::new(|| {
    EnvironmentAttributeMap::builder()
        .modify(
            EnvironmentAttributes::SKY_COLOR,
            AttributeModifier::Color(ColorModifier::BLEND_TO_GRAY_RGB),
            AttributeValue::BlendToGray(BlendToGray::new(0.24, 0.94)),
        )
        .modify(
            EnvironmentAttributes::FOG_COLOR,
            AttributeModifier::Color(ColorModifier::MULTIPLY_RGB),
            AttributeValue::Rgb(ARGB::vector3f_from_rgb24(ARGB::color_from_float(
                1.0, 0.25, 0.25, 0.3,
            ))),
        )
        .modify(
            EnvironmentAttributes::CLOUD_COLOR,
            AttributeModifier::Color(ColorModifier::BLEND_TO_GRAY_ARGB),
            AttributeValue::BlendToGray(BlendToGray::new(0.095, 0.94)),
        )
        .modify(
            EnvironmentAttributes::SKY_LIGHT_LEVEL,
            AttributeModifier::Float(FloatModifier::ALPHA_BLEND),
            AttributeValue::FloatWithAlpha(FloatWithAlpha::new(4.0, 0.52734375)),
        )
        .modify(
            EnvironmentAttributes::SKY_LIGHT_COLOR,
            AttributeModifier::Color(ColorModifier::ALPHA_BLEND_RGB),
            AttributeValue::Argb(ARGB::vector4f_from_argb32(ARGB::color_alpha_f32(
                0.52734375,
                Timelines::night_sky_light_color(),
            ))),
        )
        .modify(
            EnvironmentAttributes::SKY_LIGHT_FACTOR,
            AttributeModifier::Float(FloatModifier::ALPHA_BLEND),
            AttributeValue::FloatWithAlpha(FloatWithAlpha::new(0.24, 0.52734375)),
        )
        .set(
            EnvironmentAttributes::STAR_BRIGHTNESS,
            AttributeValue::Float(0.0),
        )
        .modify(
            EnvironmentAttributes::SUNRISE_SUNSET_COLOR,
            AttributeModifier::Color(ColorModifier::MULTIPLY_ARGB),
            AttributeValue::Argb(ARGB::vector4f_from_argb32(ARGB::color_from_float(
                1.0, 0.25, 0.25, 0.3,
            ))),
        )
        .set(
            EnvironmentAttributes::BEES_STAY_IN_HIVE,
            AttributeValue::Boolean(true),
        )
        .build()
});

impl WeatherAttributes {
    pub fn rain() -> &'static EnvironmentAttributeMap {
        &RAIN
    }

    pub fn thunder() -> &'static EnvironmentAttributeMap {
        &THUNDER
    }

    /// A layer for each attribute rain or thunder changes (`Sets.union`'s, in no order that
    /// shows: each attribute's layers are its own).
    pub fn add_builtin_layers(system: &mut Builder) {
        let mut weather_attributes: Vec<usize> = RAIN.key_set().collect();
        weather_attributes.extend(THUNDER.key_set().filter(|id| !RAIN.contains(*id)));
        for attribute in weather_attributes {
            system.add_time_based_layer(
                attribute,
                TimeBased::Weather {
                    rain_entry: RAIN.get(attribute).copied(),
                    thunder_entry: THUNDER.get(attribute).copied(),
                },
            );
        }
    }
}
