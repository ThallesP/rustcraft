//! com/mojang/blaze3d/vertex/ByteBufferBuilder.java
//!
//! A growing buffer that vertices are written into and built out of, in results. Java hands out
//! each result as a view of its memory, which stays valid until the result is closed; the port's
//! `build` copies the result's bytes out instead and frees it at once, like a caller that closes
//! it before writing more, so the builder's memory is reused the same way and the bytes are the
//! same. Memory Java leaves uninitialised is zero here.

use crate::net::minecraft::util::mth::Mth;

pub struct ByteBufferBuilder {
    buffer: Vec<u8>,
    capacity: i64,
    max_capacity: i64,
    write_offset: i64,
    next_result_offset: i64,
}

impl ByteBufferBuilder {
    const DEFAULT_MAX_CAPACITY: i64 = 4294967295;
    const MAX_GROWTH_SIZE: i64 = 2097152;

    pub fn with_max_capacity(initial_capacity: i32, max_capacity: i64) -> ByteBufferBuilder {
        ByteBufferBuilder {
            buffer: vec![0; initial_capacity as usize],
            capacity: i64::from(initial_capacity),
            max_capacity,
            write_offset: 0,
            next_result_offset: 0,
        }
    }

    pub fn new(initial_capacity: i32) -> ByteBufferBuilder {
        ByteBufferBuilder::with_max_capacity(
            initial_capacity,
            ByteBufferBuilder::DEFAULT_MAX_CAPACITY,
        )
    }

    pub fn exactly_sized(capacity: i32) -> ByteBufferBuilder {
        ByteBufferBuilder::with_max_capacity(capacity, i64::from(capacity))
    }

    /// The offset of `size` new bytes, where Java returns their address.
    pub fn reserve(&mut self, size: i32) -> usize {
        let offset = self.write_offset;
        let next_offset = offset
            .checked_add(i64::from(size))
            .expect("integer overflow");
        self.ensure_capacity(next_offset);
        self.write_offset = next_offset;
        offset as usize
    }

    /// Panics where Java throws `IllegalArgumentException` past the maximum capacity.
    fn ensure_capacity(&mut self, required_capacity: i64) {
        if required_capacity > self.capacity {
            assert!(
                required_capacity <= self.max_capacity,
                "Maximum capacity of ByteBufferBuilder ({}) exceeded, required {required_capacity}",
                self.max_capacity
            );
            let preferred_growth = self.capacity.min(ByteBufferBuilder::MAX_GROWTH_SIZE);
            let new_capacity = Mth::clamp_i64(
                self.capacity + preferred_growth,
                required_capacity,
                self.max_capacity,
            );
            self.buffer.resize(new_capacity as usize, 0);
            self.capacity = new_capacity;
        }
    }

    /// The bytes written since the last result, or `None` for none.
    pub fn build(&mut self) -> Option<Vec<u8>> {
        let offset = self.next_result_offset;
        let size = self.write_offset - offset;
        if size == 0 {
            return None;
        }
        assert!(
            size <= i64::from(i32::MAX),
            "Cannot build buffer larger than 2147483647 bytes (was {size})"
        );
        self.next_result_offset = self.write_offset;
        let bytes = self.buffer[offset as usize..self.write_offset as usize].to_vec();
        self.discard_results();
        Some(bytes)
    }

    /// The bytes `reserve` handed out, to write into.
    pub fn bytes_mut(&mut self, offset: usize, size: usize) -> &mut [u8] {
        &mut self.buffer[offset..offset + size]
    }

    fn discard_results(&mut self) {
        let current_size = self.write_offset - self.next_result_offset;
        if current_size > 0 {
            let start = self.next_result_offset as usize;
            self.buffer
                .copy_within(start..start + current_size as usize, 0);
        }
        self.write_offset = current_size;
        self.next_result_offset = 0;
    }
}
