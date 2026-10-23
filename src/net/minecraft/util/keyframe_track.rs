//! net/minecraft/util/KeyframeTrack.java
//!
//! `mapCodec` is `from_json`, decoded by hand like its record codec: `keyframes`, each `ticks`
//! (non-negative) and `value` by the value's decoder, validated like `validateKeyframes`, and
//! `ease`, linear when it's missing. `validatePeriod` and the builder come with their callers.

use crate::com::google::gson::json_element::{JsonElement, JsonObject};
use crate::net::minecraft::util::easing_type::EasingType;
use crate::net::minecraft::util::keyframe::Keyframe;
use crate::net::minecraft::util::keyframe_track_sampler::{KeyframeTrackSampler, Lerp};

/// Keyframes in tick order, and how to ease between them.
#[derive(Clone, Debug)]
pub struct KeyframeTrack<T> {
    keyframes: Vec<Keyframe<T>>,
    easing_type: EasingType,
}

impl<T: Clone> KeyframeTrack<T> {
    /// Panics where Java throws `IllegalArgumentException` for no keyframes.
    pub fn new(keyframes: Vec<Keyframe<T>>, easing_type: EasingType) -> KeyframeTrack<T> {
        assert!(!keyframes.is_empty(), "Track has no keyframes");
        KeyframeTrack {
            keyframes,
            easing_type,
        }
    }

    pub fn keyframes(&self) -> &[Keyframe<T>] {
        &self.keyframes
    }

    pub fn easing_type(&self) -> EasingType {
        self.easing_type
    }

    /// `mapCodec(valueCodec)` over a track's object, each value by `value`.
    pub fn from_json(
        object: &JsonObject,
        value: impl Fn(&JsonElement) -> Result<T, String>,
    ) -> Result<KeyframeTrack<T>, String> {
        let keyframes = object
            .get("keyframes")
            .and_then(JsonElement::as_json_array)
            .ok_or("No key keyframes in MapLike")?
            .iter()
            .map(|keyframe| {
                let keyframe = keyframe
                    .as_json_object()
                    .ok_or("a keyframe isn't an object")?;
                let ticks = keyframe
                    .get("ticks")
                    .ok_or("No key ticks in MapLike")?
                    .get_as_int();
                if ticks < 0 {
                    return Err(format!("Value must be non-negative: {ticks}"));
                }
                let value = value(keyframe.get("value").ok_or("No key value in MapLike")?)?;
                Ok(Keyframe::new(ticks, value))
            })
            .collect::<Result<Vec<_>, String>>()?;
        KeyframeTrack::validate_keyframes(&keyframes)?;
        let easing_type = match object.get("ease") {
            Some(ease) => EasingType::from_json(ease)?,
            None => EasingType::LINEAR,
        };
        Ok(KeyframeTrack::new(keyframes, easing_type))
    }

    fn validate_keyframes(keyframes: &[Keyframe<T>]) -> Result<(), String> {
        if keyframes.is_empty() {
            return Err("Keyframes must not be empty".to_owned());
        }

        if keyframes
            .windows(2)
            .any(|pair| pair[0].ticks > pair[1].ticks)
        {
            return Err("Keyframes must be ordered by ticks field".to_owned());
        }

        if keyframes.len() > 1 {
            let mut repeat_count = 0;
            let mut last_ticks = keyframes.last().expect("not empty").ticks;

            for keyframe in keyframes {
                if keyframe.ticks == last_ticks {
                    repeat_count += 1;
                    if repeat_count > 2 {
                        return Err(format!(
                            "More than 2 keyframes on same tick: {}",
                            keyframe.ticks
                        ));
                    }
                } else {
                    repeat_count = 0;
                }

                last_ticks = keyframe.ticks;
            }
        }

        Ok(())
    }

    pub fn bake_sampler(
        &self,
        period_ticks: Option<i32>,
        lerp: Lerp<T>,
    ) -> KeyframeTrackSampler<T> {
        KeyframeTrackSampler::new(self, period_ticks, lerp)
    }
}
