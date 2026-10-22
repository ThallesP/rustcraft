//! io/netty/buffer/AbstractByteBufAllocator.java
//!
//! Only `calculateNewCapacity`, which picks the capacity a buffer grows to. The port has one
//! allocator, whose buffers are `Unpooled.buffer`'s (see `byte_buf.rs`).

/*
 * Copyright 2012 The Netty Project
 *
 * The Netty Project licenses this file to you under the Apache License,
 * version 2.0 (the "License"); you may not use this file except in compliance
 * with the License. You may obtain a copy of the License at:
 *
 *   https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations
 * under the License.
 *
 * Translated to Rust from Netty 4.2.16.Final, and changed: see above.
 */

pub struct AbstractByteBufAllocator;

impl AbstractByteBufAllocator {
    pub const DEFAULT_INITIAL_CAPACITY: i32 = 256;
    pub const DEFAULT_MAX_CAPACITY: i32 = i32::MAX;
    /// 4 MiB, a page.
    pub const CALCULATE_THRESHOLD: i32 = 1048576 * 4;

    /// Doubles up to the threshold (from 64), then grows by whole thresholds.
    pub fn calculate_new_capacity(min_new_capacity: i32, max_capacity: i32) -> i32 {
        assert!(
            min_new_capacity >= 0,
            "minNewCapacity : {min_new_capacity} (expected: >= 0)"
        );
        assert!(
            min_new_capacity <= max_capacity,
            "minNewCapacity: {min_new_capacity} (expected: not greater than maxCapacity({max_capacity})"
        );
        let threshold = AbstractByteBufAllocator::CALCULATE_THRESHOLD;

        if min_new_capacity == threshold {
            return threshold;
        }

        // If over threshold, do not double but just increase by threshold.
        if min_new_capacity > threshold {
            let mut new_capacity = min_new_capacity / threshold * threshold;
            if new_capacity > max_capacity - threshold {
                new_capacity = max_capacity;
            } else {
                new_capacity += threshold;
            }
            return new_capacity;
        }

        // 64 <= newCapacity is a power of 2 <= threshold
        let new_capacity = find_next_positive_power_of_two(min_new_capacity.max(64));
        new_capacity.min(max_capacity)
    }
}

/// netty-common's `MathUtil.findNextPositivePowerOfTwo`, for values from 1 to 2^30.
fn find_next_positive_power_of_two(value: i32) -> i32 {
    1 << (32 - (value - 1).leading_zeros())
}
