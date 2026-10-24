//! net/minecraft/sounds/SoundEvent.java
//!
//! Not ported yet: its codecs.

use crate::net::minecraft::resources::identifier::Identifier;

/// A sound by its name in the sounds' definitions, and how far it's heard, if that's fixed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundEvent {
    location: &'static str,
    fixed_range: Option<f32>,
}

impl SoundEvent {
    /// `createVariableRangeEvent`, of the location's text.
    pub const fn create_variable_range_event(location: &'static str) -> SoundEvent {
        SoundEvent {
            location,
            fixed_range: None,
        }
    }

    /// `createFixedRangeEvent`, of the location's text.
    pub const fn create_fixed_range_event(location: &'static str, range: f32) -> SoundEvent {
        SoundEvent {
            location,
            fixed_range: Some(range),
        }
    }

    pub fn location(&self) -> Identifier {
        Identifier::parse(self.location)
    }

    pub fn fixed_range(&self) -> Option<f32> {
        self.fixed_range
    }

    /// `getRange(volume)`: its fixed range, or 16 blocks, more for a louder sound.
    pub fn get_range(&self, volume: f32) -> f32 {
        self.fixed_range
            .unwrap_or(if volume > 1.0 { 16.0 * volume } else { 16.0 })
    }
}
