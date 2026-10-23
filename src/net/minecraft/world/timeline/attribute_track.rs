//! net/minecraft/world/timeline/AttributeTrack.java
//!
//! A timeline's track of one attribute: a modifier, and its argument over time. `createCodec` is
//! `from_json` (`modifier`, override when it's missing, and the keyframe track of its argument);
//! `validatePeriod` comes with timelines' validation.

use crate::com::google::gson::json_element::JsonObject;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::util::keyframe_track::KeyframeTrack;
use crate::net::minecraft::world::attribute::attribute_type::AttributeValue;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::attribute::modifier::attribute_modifier::{
    AttributeModifier, OperationId,
};
use crate::net::minecraft::world::timeline::attribute_track_sampler::AttributeTrackSampler;

#[derive(Clone, Debug)]
pub struct AttributeTrack {
    pub modifier: AttributeModifier,
    pub argument_track: KeyframeTrack<AttributeValue>,
}

impl AttributeTrack {
    /// `createCodec(attribute)` over a track's object.
    pub fn from_json(attribute: usize, object: &JsonObject) -> Result<AttributeTrack, String> {
        let info = EnvironmentAttributes::info(attribute);
        let modifier = match object.get("modifier") {
            Some(modifier) => OperationId::decode_modifier(info.r#type, modifier)?,
            None => AttributeModifier::Override,
        };
        let argument_track = KeyframeTrack::from_json(object, |argument| match modifier {
            AttributeModifier::Override => info.decode_value(argument),
            _ => modifier.decode_argument(info.r#type, argument),
        })?;
        Ok(AttributeTrack {
            modifier,
            argument_track,
        })
    }

    pub fn bake_sampler(
        &self,
        attribute: usize,
        clock: &Holder,
        period_ticks: Option<i32>,
    ) -> AttributeTrackSampler {
        let lerp = self
            .modifier
            .argument_keyframe_lerp(EnvironmentAttributes::info(attribute).r#type);
        AttributeTrackSampler::new(
            clock.clone(),
            period_ticks,
            self.modifier,
            &self.argument_track,
            lerp,
        )
    }
}
