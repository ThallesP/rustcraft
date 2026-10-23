//! net/minecraft/world/timeline/Timeline.java
//!
//! A cycle of attribute tracks on a clock: the day, the moon's phases. The client reads its known
//! pack's timelines through `NETWORK_CODEC`, which is `from_json`: the clock, the period, and the
//! syncable attributes' tracks (`filterSyncableTracks`), decoded by hand from Gson's tree. Tracks
//! of attributes whose types aren't ported are left out. Not ported: time markers (which only
//! commands read), the period's validation, the builder, `getPeriodCount` and `CODEC`'s holders.

use crate::com::google::gson::json_element::JsonObject;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::timeline::attribute_track::AttributeTrack;
use crate::net::minecraft::world::timeline::attribute_track_sampler::AttributeTrackSampler;

#[derive(Clone, Debug)]
pub struct Timeline {
    clock: Holder,
    period_ticks: Option<i32>,
    tracks: Vec<(usize, AttributeTrack)>,
}

impl Timeline {
    /// `NETWORK_CODEC` over a timeline's object.
    pub fn from_json(object: &JsonObject) -> Result<Timeline, String> {
        let clock = object
            .get("clock")
            .ok_or("No key clock in MapLike")?
            .get_as_string();
        let clock = Holder::Reference(ResourceKey::create(
            Registries::WORLD_CLOCK,
            Identifier::parse(&clock),
        ));
        let period_ticks = match object.get("period_ticks") {
            Some(period) => {
                let period = period.get_as_int();
                if period < 1 {
                    return Err(format!("Value must be positive: {period}"));
                }
                Some(period)
            }
            None => None,
        };
        let mut tracks = Vec::new();
        if let Some(json) = object.get("tracks") {
            for (name, track) in json.get_as_json_object().entry_set() {
                let Some(attribute) = EnvironmentAttributes::by_name(&Identifier::parse(name))?
                else {
                    continue;
                };
                if !EnvironmentAttributes::info(attribute).is_syncable {
                    continue;
                }
                tracks.push((
                    attribute,
                    AttributeTrack::from_json(attribute, track.get_as_json_object())?,
                ));
            }
        }
        Ok(Timeline {
            clock,
            period_ticks,
            tracks,
        })
    }

    pub fn clock(&self) -> &Holder {
        &self.clock
    }

    pub fn period_ticks(&self) -> Option<i32> {
        self.period_ticks
    }

    /// `getCurrentTicks`: the clock's ticks into the period.
    pub fn get_current_ticks(&self, total_ticks: i64) -> i64 {
        match self.period_ticks {
            Some(period_ticks) => total_ticks % i64::from(period_ticks),
            None => total_ticks,
        }
    }

    /// The attributes it has tracks for, by place.
    pub fn attributes(&self) -> impl Iterator<Item = usize> + '_ {
        self.tracks.iter().map(|(attribute, _)| *attribute)
    }

    /// Panics where Java throws `IllegalStateException` for an attribute it has no track for.
    pub fn create_track_sampler(&self, attribute: usize) -> AttributeTrackSampler {
        let (_, track) = self
            .tracks
            .iter()
            .find(|(id, _)| *id == attribute)
            .unwrap_or_else(|| {
                panic!(
                    "Timeline has no track for {}",
                    EnvironmentAttributes::info(attribute).name
                )
            });
        track.bake_sampler(attribute, &self.clock, self.period_ticks)
    }
}
