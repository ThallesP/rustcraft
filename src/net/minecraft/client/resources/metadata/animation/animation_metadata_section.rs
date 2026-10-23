//! net/minecraft/client/resources/metadata/animation/AnimationMetadataSection.java
//!
//! A sprite's `animation` section, decoded by hand from serde_json until DataFixerUpper's codecs
//! come (PORTING.md), with the codec's defaults: `frametime` 1, `interpolate` false.

use crate::net::minecraft::client::resources::metadata::animation::animation_frame::{
    AnimationFrame, int_at_least,
};
use crate::net::minecraft::client::resources::metadata::animation::frame_size::FrameSize;
use crate::net::minecraft::server::packs::resources::resource_metadata::MetadataSectionType;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationMetadataSection {
    pub frames: Option<Vec<AnimationFrame>>,
    pub frame_width: Option<i32>,
    pub frame_height: Option<i32>,
    pub default_frame_time: i32,
    pub interpolated_frames: bool,
}

impl AnimationMetadataSection {
    pub const TYPE: MetadataSectionType<AnimationMetadataSection> = MetadataSectionType {
        name: "animation",
        codec: AnimationMetadataSection::codec,
    };

    pub fn codec(value: &Value) -> Result<AnimationMetadataSection, String> {
        let object = value
            .as_object()
            .ok_or_else(|| format!("Not a map: {value}"))?;
        let positive = |key: &str| -> Result<Option<i32>, String> {
            object
                .get(key)
                .map(|value| int_at_least(value, 1, "positive"))
                .transpose()
        };
        let frames = match object.get("frames") {
            Some(Value::Array(frames)) => Some(
                frames
                    .iter()
                    .map(AnimationFrame::codec)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Some(other) => return Err(format!("Not a list: {other}")),
            None => None,
        };
        let interpolated_frames = match object.get("interpolate") {
            Some(Value::Bool(interpolate)) => *interpolate,
            Some(other) => return Err(format!("Not a boolean: {other}")),
            None => false,
        };
        Ok(AnimationMetadataSection {
            frames,
            frame_width: positive("width")?,
            frame_height: positive("height")?,
            default_frame_time: positive("frametime")?.unwrap_or(1),
            interpolated_frames,
        })
    }

    pub fn calculate_frame_size(&self, sprite_width: i32, sprite_height: i32) -> FrameSize {
        if let Some(frame_width) = self.frame_width {
            return match self.frame_height {
                Some(frame_height) => FrameSize {
                    width: frame_width,
                    height: frame_height,
                },
                None => FrameSize {
                    width: frame_width,
                    height: sprite_height,
                },
            };
        }
        if let Some(frame_height) = self.frame_height {
            return FrameSize {
                width: sprite_width,
                height: frame_height,
            };
        }
        let min_dimension = sprite_width.min(sprite_height);
        FrameSize {
            width: min_dimension,
            height: min_dimension,
        }
    }
}
