//! net/minecraft/client/FramerateLimiter.java
//!
//! Waits out the rest of a frame at the frame rate limit: it sleeps for most of the time left,
//! keeping half a millisecond and the sleeps' average overshoot in hand, then spins. Java's
//! statics are the game thread's, the only thread that limits frames. `LockSupport.parkNanos` is a
//! sleep, which nothing here interrupts.

use std::cell::Cell;
use std::time::Duration;

use crate::net::minecraft::util::util::Util;

pub struct FramerateLimiter;

thread_local! {
    /// `lastFrameTime`, `System.nanoTime()` when the class loaded.
    static LAST_FRAME_TIME: Cell<i64> = Cell::new(Util::get_nanos());
    static AVERAGE_OVERSHOOT_NS: Cell<i64> = const { Cell::new(0) };
    static LAST_FRAMERATE_LIMIT: Cell<i32> = const { Cell::new(0) };
}

impl FramerateLimiter {
    const OVERSHOOT_SMOOTHING: f64 = 0.1;
    const MAX_CURRENT_OVERSHOOT_NS: i64 = 25000000;
    const MAX_AVERAGE_OVERSHOOT_NS: i64 = 2000000;
    const ONE_SECOND_IN_NANOSECONDS: i64 = 1000000000;
    const SPIN_SAFETY_BUFFER_NS: i64 = 500000;

    pub fn limit_display_fps(framerate_limit: i32) {
        let target_time_per_frame =
            FramerateLimiter::ONE_SECOND_IN_NANOSECONDS / i64::from(framerate_limit);
        let target_time_ns = LAST_FRAME_TIME.get() + target_time_per_frame;
        if framerate_limit != LAST_FRAMERATE_LIMIT.get() {
            AVERAGE_OVERSHOOT_NS.set(0);
            LAST_FRAMERATE_LIMIT.set(framerate_limit);
        }

        loop {
            let remaining_time_ns = target_time_ns - Util::get_nanos();
            if remaining_time_ns <= 0 {
                break;
            }
            let average_overshoot_ns = AVERAGE_OVERSHOOT_NS.get();
            if remaining_time_ns > average_overshoot_ns + FramerateLimiter::SPIN_SAFETY_BUFFER_NS {
                let sleep_start_time_ns = Util::get_nanos();
                let expected_sleep_time_ns = remaining_time_ns
                    - average_overshoot_ns
                    - FramerateLimiter::SPIN_SAFETY_BUFFER_NS;
                std::thread::sleep(Duration::from_nanos(expected_sleep_time_ns as u64));
                let sleep_duration_ns = Util::get_nanos() - sleep_start_time_ns;
                let current_overshoot_ns = sleep_duration_ns - expected_sleep_time_ns;
                if current_overshoot_ns > 0
                    && current_overshoot_ns < FramerateLimiter::MAX_CURRENT_OVERSHOOT_NS
                {
                    let smoothed =
                        (FramerateLimiter::OVERSHOOT_SMOOTHING * current_overshoot_ns as f64
                            + 0.9 * average_overshoot_ns as f64) as i64;
                    AVERAGE_OVERSHOOT_NS
                        .set(smoothed.min(FramerateLimiter::MAX_AVERAGE_OVERSHOOT_NS));
                }
            } else {
                std::hint::spin_loop();
            }
        }

        LAST_FRAME_TIME.set(Util::get_nanos());
    }
}
