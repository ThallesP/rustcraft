//! net/minecraft/client/resources/metadata/animation/AnimationFrame.java
//!
//! `CODEC` reads a frame as its index alone or as `{"index": …, "time": …}` (`Codec.either` of
//! `NON_NEGATIVE_INT` and `FULL_CODEC`), decoded by hand from serde_json until DataFixerUpper's
//! codecs come (PORTING.md).

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationFrame {
    pub index: i32,
    pub time: Option<i32>,
}

/// `ExtraCodecs.NON_NEGATIVE_INT` and `POSITIVE_INT`: an int at least `min`.
pub(crate) fn int_at_least(value: &Value, min: i32, what: &str) -> Result<i32, String> {
    let number = value
        .as_i64()
        .filter(|number| i32::try_from(*number).is_ok())
        .ok_or_else(|| format!("Not a number: {value}"))?;
    let number = number as i32;
    if number < min {
        return Err(format!("Value must be {what}: {number}"));
    }
    Ok(number)
}

impl AnimationFrame {
    pub fn new(index: i32) -> AnimationFrame {
        AnimationFrame { index, time: None }
    }

    pub fn codec(value: &Value) -> Result<AnimationFrame, String> {
        if let Some(object) = value.as_object() {
            let index = object
                .get("index")
                .ok_or("No key index in MapLike")
                .map_err(str::to_owned)
                .and_then(|index| int_at_least(index, 0, "non-negative"))?;
            let time = match object.get("time") {
                Some(time) => Some(int_at_least(time, 1, "positive")?),
                None => None,
            };
            return Ok(AnimationFrame { index, time });
        }
        int_at_least(value, 0, "non-negative").map(AnimationFrame::new)
    }

    pub fn time_or(&self, default_frame_time: i32) -> i32 {
        self.time.unwrap_or(default_frame_time)
    }
}
