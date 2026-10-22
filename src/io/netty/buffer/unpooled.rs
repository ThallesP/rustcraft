//! io/netty/buffer/Unpooled.java
//!
//! Only the heap buffers: `buffer()`, `buffer(int)` and `wrappedBuffer(byte[])`. Java's wrapped
//! buffer shares the array it wraps, so writing to either shows in the other; the port's owns it,
//! which is the same while only the buffer is used, as the chunk packets use theirs.

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

use crate::io::netty::buffer::abstract_byte_buf_allocator::AbstractByteBufAllocator;
use crate::io::netty::buffer::byte_buf::ByteBuf;

/// An empty buffer of 256 bytes, which grows as far as an int goes.
pub fn buffer() -> ByteBuf {
    buffer_initial_capacity(AbstractByteBufAllocator::DEFAULT_INITIAL_CAPACITY)
}

/// `buffer(int)`.
pub fn buffer_initial_capacity(initial_capacity: i32) -> ByteBuf {
    ByteBuf::new(
        initial_capacity,
        AbstractByteBufAllocator::DEFAULT_MAX_CAPACITY,
    )
}

/// `wrappedBuffer(byte[])`: the array, all of it readable, which can't grow. An empty one is
/// `EMPTY_BUFFER`.
pub fn wrapped_buffer(array: Vec<i8>) -> ByteBuf {
    if array.is_empty() {
        ByteBuf::empty()
    } else {
        let length = array.len() as i32;
        ByteBuf::from_array(array, length)
    }
}
