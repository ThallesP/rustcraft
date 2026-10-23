//! net/minecraft/util/KeyframeTrackSampler.java
//!
//! A track's value at any tick: the segment between the keyframes around it, eased and blended
//! by the track's lerp (Java's `LerpFunction`, a closure here). A periodic track wraps around,
//! from its last keyframe to its first.

use crate::java::lang::math::Math;
use crate::net::minecraft::util::easing_type::EasingType;
use crate::net::minecraft::util::keyframe::Keyframe;
use crate::net::minecraft::util::keyframe_track::KeyframeTrack;

/// Java's `LerpFunction<T>`: how two values blend.
pub type Lerp<T> = Box<dyn Fn(f32, &T, &T) -> T>;

pub struct KeyframeTrackSampler<T> {
    period_ticks: Option<i32>,
    lerp: Lerp<T>,
    segments: Vec<Segment<T>>,
}

struct Segment<T> {
    easing: EasingType,
    from_value: T,
    from_ticks: i32,
    to_value: T,
    to_ticks: i32,
}

impl<T: Clone> Segment<T> {
    fn new(
        track: &KeyframeTrack<T>,
        from: &Keyframe<T>,
        from_ticks: i32,
        to: &Keyframe<T>,
        to_ticks: i32,
    ) -> Segment<T> {
        Segment {
            easing: track.easing_type(),
            from_value: from.value.clone(),
            from_ticks,
            to_value: to.value.clone(),
            to_ticks,
        }
    }
}

impl<T: Clone> KeyframeTrackSampler<T> {
    pub(crate) fn new(
        track: &KeyframeTrack<T>,
        period_ticks: Option<i32>,
        lerp: Lerp<T>,
    ) -> KeyframeTrackSampler<T> {
        KeyframeTrackSampler {
            period_ticks,
            lerp,
            segments: KeyframeTrackSampler::bake_segments(track, period_ticks),
        }
    }

    fn bake_segments(track: &KeyframeTrack<T>, period_ticks: Option<i32>) -> Vec<Segment<T>> {
        let keyframes = track.keyframes();
        if keyframes.len() == 1 {
            let value = &keyframes[0].value;
            return vec![Segment {
                easing: EasingType::CONSTANT,
                from_value: value.clone(),
                from_ticks: 0,
                to_value: value.clone(),
                to_ticks: 0,
            }];
        }

        let mut segments = Vec::new();
        match period_ticks {
            Some(period_ticks) => {
                let first_keyframe = &keyframes[0];
                let last_keyframe = &keyframes[keyframes.len() - 1];
                segments.push(Segment::new(
                    track,
                    last_keyframe,
                    last_keyframe.ticks - period_ticks,
                    first_keyframe,
                    first_keyframe.ticks,
                ));
                KeyframeTrackSampler::add_segments_from_keyframes(track, keyframes, &mut segments);
                segments.push(Segment::new(
                    track,
                    last_keyframe,
                    last_keyframe.ticks,
                    first_keyframe,
                    first_keyframe.ticks + period_ticks,
                ));
            }
            None => {
                KeyframeTrackSampler::add_segments_from_keyframes(track, keyframes, &mut segments)
            }
        }

        segments
    }

    fn add_segments_from_keyframes(
        track: &KeyframeTrack<T>,
        keyframes: &[Keyframe<T>],
        output: &mut Vec<Segment<T>>,
    ) {
        for pair in keyframes.windows(2) {
            let (keyframe, next_keyframe) = (&pair[0], &pair[1]);
            output.push(Segment::new(
                track,
                keyframe,
                keyframe.ticks,
                next_keyframe,
                next_keyframe.ticks,
            ));
        }
    }

    pub fn sample(&self, ticks: i64) -> T {
        let sample_ticks = self.loop_ticks(ticks);
        let segment = self.get_segment_at(sample_ticks);
        if sample_ticks <= i64::from(segment.from_ticks) {
            return segment.from_value.clone();
        }

        if sample_ticks >= i64::from(segment.to_ticks) {
            return segment.to_value.clone();
        }

        // A long over an int, both as floats.
        let alpha = (sample_ticks - i64::from(segment.from_ticks)) as f32
            / (segment.to_ticks - segment.from_ticks) as f32;
        let eased_alpha = segment.easing.apply(alpha);
        (self.lerp)(eased_alpha, &segment.from_value, &segment.to_value)
    }

    fn get_segment_at(&self, current_ticks: i64) -> &Segment<T> {
        self.segments
            .iter()
            .find(|segment| current_ticks < i64::from(segment.to_ticks))
            .unwrap_or_else(|| self.segments.last().expect("a track has keyframes"))
    }

    fn loop_ticks(&self, ticks: i64) -> i64 {
        match self.period_ticks {
            Some(period_ticks) => Math::floor_mod_i64(ticks, i64::from(period_ticks)),
            None => ticks,
        }
    }
}
