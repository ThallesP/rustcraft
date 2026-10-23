//! net/minecraft/world/timeline/AttributeTrackSampler.java
//!
//! A track as a layer of its attribute: the argument at the clock's ticks, worked out once a tick
//! (the cache, in cells, since sampling reads the system), then the modifier with it. Java's
//! sampler holds the level's `ClockManager`; here the level is a parameter (`LayerLevel`).

use std::cell::{Cell, RefCell};

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::util::keyframe_track::KeyframeTrack;
use crate::net::minecraft::util::keyframe_track_sampler::KeyframeTrackSampler;
use crate::net::minecraft::world::attribute::attribute_type::AttributeValue;
use crate::net::minecraft::world::attribute::environment_attribute_layer::LayerLevel;
use crate::net::minecraft::world::attribute::lerp_function::LerpFunction;
use crate::net::minecraft::world::attribute::modifier::attribute_modifier::AttributeModifier;

pub struct AttributeTrackSampler {
    clock: Holder,
    modifier: AttributeModifier,
    argument_sampler: KeyframeTrackSampler<AttributeValue>,
    cached_tick_id: Cell<i32>,
    cached_argument: RefCell<Option<AttributeValue>>,
}

impl AttributeTrackSampler {
    pub fn new(
        clock: Holder,
        period_ticks: Option<i32>,
        modifier: AttributeModifier,
        argument_track: &KeyframeTrack<AttributeValue>,
        argument_lerp: LerpFunction,
    ) -> AttributeTrackSampler {
        AttributeTrackSampler {
            clock,
            modifier,
            argument_sampler: argument_track.bake_sampler(
                period_ticks,
                Box::new(move |alpha, from, to| argument_lerp.apply(alpha, *from, *to)),
            ),
            cached_tick_id: Cell::new(0),
            cached_argument: RefCell::new(None),
        }
    }

    pub fn apply_time_based(
        &self,
        base_value: AttributeValue,
        cache_tick_id: i32,
        level: &dyn LayerLevel,
    ) -> AttributeValue {
        let mut cached_argument = self.cached_argument.borrow_mut();
        if cached_argument.is_none() || cache_tick_id != self.cached_tick_id.get() {
            self.cached_tick_id.set(cache_tick_id);
            *cached_argument = Some(
                self.argument_sampler
                    .sample(level.clock_total_ticks(&self.clock)),
            );
        }

        self.modifier
            .apply(base_value, cached_argument.as_ref().expect("sampled above"))
    }
}
