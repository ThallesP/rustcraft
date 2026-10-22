//! io/netty/buffer/ByteBuf.java, as `Unpooled.buffer()` makes it: AbstractByteBuf.java's indexes
//! and bounds checks over UnpooledHeapByteBuf.java's array, read and written big-endian like
//! HeapByteBufUtil.java.
//!
//! The game only sees `ByteBuf`'s API, so the port has one kind of buffer. Netty's others (pooled,
//! direct, composite) keep their bytes elsewhere and grow differently, but read and write alike,
//! and a slice is a copy of its range. `retain` and `release` are Rust's ownership, and `alloc()`
//! is the one allocator.
//!
//! Only the methods the client calls are ported, as they come up (PORTING.md): so far the indexes
//! and marks, capacity, reads and writes of primitives and byte arrays, `readBytes`, `skipBytes`,
//! `writeBytes` from another buffer, `toString` of a range as UTF-8, and `slice`.
//!
//! Java throws `IndexOutOfBoundsException` or `IllegalArgumentException`, which are panics here. A
//! call that throws can have changed the buffer already, as in Java: `writeBytes` grows the array
//! before it checks the source's range.

/*
 * Copyright 2012, 2015 The Netty Project
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
use crate::java::lang::string;

/// A byte array with a reader index and a writer index: the bytes between them are readable, and
/// writes go after the writer index, growing the array up to the maximum capacity.
#[derive(Clone, Debug)]
pub struct ByteBuf {
    array: Vec<i8>,
    reader_index: i32,
    writer_index: i32,
    marked_reader_index: i32,
    marked_writer_index: i32,
    max_capacity: i32,
}

impl ByteBuf {
    /// `new UnpooledHeapByteBuf(alloc, initialCapacity, maxCapacity)`, after the allocator's
    /// checks.
    pub fn new(initial_capacity: i32, max_capacity: i32) -> ByteBuf {
        assert!(
            initial_capacity >= 0,
            "initialCapacity : {initial_capacity} (expected: >= 0)"
        );
        assert!(
            max_capacity >= 0,
            "maxCapacity : {max_capacity} (expected: >= 0)"
        );
        assert!(
            initial_capacity <= max_capacity,
            "initialCapacity({initial_capacity}) > maxCapacity({max_capacity})"
        );
        ByteBuf {
            array: vec![0; initial_capacity as usize],
            reader_index: 0,
            writer_index: 0,
            marked_reader_index: 0,
            marked_writer_index: 0,
            max_capacity,
        }
    }

    /// `new UnpooledHeapByteBuf(alloc, initialArray, maxCapacity)`: all of the array is readable.
    pub fn from_array(initial_array: Vec<i8>, max_capacity: i32) -> ByteBuf {
        assert!(
            max_capacity >= 0,
            "maxCapacity : {max_capacity} (expected: >= 0)"
        );
        let length = initial_array.len() as i32;
        assert!(
            length <= max_capacity,
            "initialCapacity({length}) > maxCapacity({max_capacity})"
        );
        let mut buffer = ByteBuf {
            array: initial_array,
            reader_index: 0,
            writer_index: 0,
            marked_reader_index: 0,
            marked_writer_index: 0,
            max_capacity,
        };
        buffer.set_index(0, length);
        buffer
    }

    /// `Unpooled.EMPTY_BUFFER`, which can't grow. Its own class (`EmptyByteBuf`) answers some
    /// questions differently (`isReadable(0)` is false), which isn't ported.
    pub const fn empty() -> ByteBuf {
        ByteBuf {
            array: Vec::new(),
            reader_index: 0,
            writer_index: 0,
            marked_reader_index: 0,
            marked_writer_index: 0,
            max_capacity: 0,
        }
    }

    pub fn capacity(&self) -> i32 {
        self.array.len() as i32
    }

    /// `capacity(int)`: a new array of that size with the old one's bytes, and indexes past it
    /// trimmed to it.
    pub fn set_capacity(&mut self, new_capacity: i32) -> &mut ByteBuf {
        self.check_new_capacity(new_capacity);
        let old_capacity = self.capacity();
        if new_capacity == old_capacity {
            return self;
        }

        if new_capacity < old_capacity {
            self.trim_indices_to_capacity(new_capacity);
        }
        self.array.resize(new_capacity as usize, 0);
        self
    }

    pub fn max_capacity(&self) -> i32 {
        self.max_capacity
    }

    pub fn reader_index(&self) -> i32 {
        self.reader_index
    }

    /// `readerIndex(int)`.
    pub fn set_reader_index(&mut self, reader_index: i32) -> &mut ByteBuf {
        check_index_bounds(reader_index, self.writer_index, self.capacity());
        self.reader_index = reader_index;
        self
    }

    pub fn writer_index(&self) -> i32 {
        self.writer_index
    }

    /// `writerIndex(int)`, which can move it past what was written, up to the capacity.
    pub fn set_writer_index(&mut self, writer_index: i32) -> &mut ByteBuf {
        check_index_bounds(self.reader_index, writer_index, self.capacity());
        self.writer_index = writer_index;
        self
    }

    pub fn set_index(&mut self, reader_index: i32, writer_index: i32) -> &mut ByteBuf {
        check_index_bounds(reader_index, writer_index, self.capacity());
        self.reader_index = reader_index;
        self.writer_index = writer_index;
        self
    }

    /// Both indexes to 0. The bytes stay, and so do the marks.
    pub fn clear(&mut self) -> &mut ByteBuf {
        self.reader_index = 0;
        self.writer_index = 0;
        self
    }

    pub fn is_readable(&self) -> bool {
        self.writer_index > self.reader_index
    }

    /// `isReadable(int)`.
    pub fn is_readable_num_bytes(&self, num_bytes: i32) -> bool {
        self.writer_index - self.reader_index >= num_bytes
    }

    pub fn is_writable(&self) -> bool {
        self.capacity() > self.writer_index
    }

    /// `isWritable(int)`.
    pub fn is_writable_num_bytes(&self, num_bytes: i32) -> bool {
        self.capacity() - self.writer_index >= num_bytes
    }

    pub fn readable_bytes(&self) -> i32 {
        self.writer_index - self.reader_index
    }

    pub fn writable_bytes(&self) -> i32 {
        self.capacity() - self.writer_index
    }

    pub fn max_writable_bytes(&self) -> i32 {
        self.max_capacity - self.writer_index
    }

    pub fn mark_reader_index(&mut self) -> &mut ByteBuf {
        self.marked_reader_index = self.reader_index;
        self
    }

    /// Back to the mark, which has to be within the indexes.
    pub fn reset_reader_index(&mut self) -> &mut ByteBuf {
        self.set_reader_index(self.marked_reader_index);
        self
    }

    pub fn mark_writer_index(&mut self) -> &mut ByteBuf {
        self.marked_writer_index = self.writer_index;
        self
    }

    pub fn reset_writer_index(&mut self) -> &mut ByteBuf {
        self.set_writer_index(self.marked_writer_index);
        self
    }

    pub fn ensure_writable(&mut self, min_writable_bytes: i32) -> &mut ByteBuf {
        assert!(
            min_writable_bytes >= 0,
            "minWritableBytes : {min_writable_bytes} (expected: >= 0)"
        );
        self.ensure_writable0(min_writable_bytes);
        self
    }

    /// Grows the array to fit that many more bytes after the writer index, to a power of two
    /// from 64 (`AbstractByteBufAllocator.calculateNewCapacity`).
    pub(super) fn ensure_writable0(&mut self, min_writable_bytes: i32) {
        let writer_index = self.writer_index;
        let target_capacity = writer_index.wrapping_add(min_writable_bytes);
        if target_capacity >= 0 && target_capacity <= self.capacity() {
            return;
        }
        assert!(
            target_capacity >= 0 && target_capacity <= self.max_capacity,
            "writerIndex({writer_index}) + minWritableBytes({min_writable_bytes}) exceeds maxCapacity({})",
            self.max_capacity
        );

        // Normalize the target capacity to the power of 2.
        let fast_writable = self.max_fast_writable_bytes();
        let new_capacity = if fast_writable >= min_writable_bytes {
            writer_index + fast_writable
        } else {
            AbstractByteBufAllocator::calculate_new_capacity(target_capacity, self.max_capacity)
        };

        // Adjust to the new capacity.
        self.set_capacity(new_capacity);
    }

    pub fn max_fast_writable_bytes(&self) -> i32 {
        self.writable_bytes()
    }

    pub fn read_boolean(&mut self) -> bool {
        self.read_byte() != 0
    }

    pub fn read_byte(&mut self) -> i8 {
        self.check_readable_bytes0(1);
        let i = self.reader_index;
        let b = self.array[i as usize];
        self.reader_index = i + 1;
        b
    }

    pub fn read_unsigned_byte(&mut self) -> i16 {
        i16::from(self.read_byte()) & 0xFF
    }

    pub fn read_short(&mut self) -> i16 {
        self.check_readable_bytes0(2);
        let v = i16::from_be_bytes(self.get_array(self.reader_index));
        self.reader_index += 2;
        v
    }

    pub fn read_unsigned_short(&mut self) -> i32 {
        i32::from(self.read_short()) & 0xFFFF
    }

    pub fn read_int(&mut self) -> i32 {
        self.check_readable_bytes0(4);
        let v = i32::from_be_bytes(self.get_array(self.reader_index));
        self.reader_index += 4;
        v
    }

    pub fn read_unsigned_int(&mut self) -> i64 {
        i64::from(self.read_int()) & 0xFFFFFFFF
    }

    pub fn read_long(&mut self) -> i64 {
        self.check_readable_bytes0(8);
        let v = i64::from_be_bytes(self.get_array(self.reader_index));
        self.reader_index += 8;
        v
    }

    /// A UTF-16 unit.
    pub fn read_char(&mut self) -> u16 {
        self.read_short() as u16
    }

    pub fn read_float(&mut self) -> f32 {
        f32::from_bits(self.read_int() as u32)
    }

    pub fn read_double(&mut self) -> f64 {
        f64::from_bits(self.read_long() as u64)
    }

    /// `readBytes(int)`: the next `length` bytes in a new buffer of that capacity, or
    /// `Unpooled.EMPTY_BUFFER` for none.
    pub fn read_bytes(&mut self, length: i32) -> ByteBuf {
        self.check_readable_bytes(length);
        if length == 0 {
            return ByteBuf::empty();
        }

        let mut buf = ByteBuf::new(length, self.max_capacity);
        buf.write_bytes_buf_range(self, self.reader_index, length);
        self.reader_index += length;
        buf
    }

    /// `readBytes(byte[])`: fills it.
    pub fn read_bytes_array(&mut self, dst: &mut [i8]) -> &mut ByteBuf {
        self.read_bytes_array_range(dst, 0, dst.len() as i32);
        self
    }

    /// `readBytes(byte[], int, int)`.
    pub fn read_bytes_array_range(
        &mut self,
        dst: &mut [i8],
        dst_index: i32,
        length: i32,
    ) -> &mut ByteBuf {
        self.check_readable_bytes(length);
        self.get_bytes_array(self.reader_index, dst, dst_index, length);
        self.reader_index += length;
        self
    }

    pub fn skip_bytes(&mut self, length: i32) -> &mut ByteBuf {
        self.check_readable_bytes(length);
        self.reader_index += length;
        self
    }

    pub fn write_boolean(&mut self, value: bool) -> &mut ByteBuf {
        self.write_byte(if value { 1 } else { 0 });
        self
    }

    /// The low 8 bits.
    pub fn write_byte(&mut self, value: i32) -> &mut ByteBuf {
        self.ensure_writable0(1);
        self.array[self.writer_index as usize] = value as i8;
        self.writer_index += 1;
        self
    }

    /// The low 16 bits.
    pub fn write_short(&mut self, value: i32) -> &mut ByteBuf {
        self.ensure_writable0(2);
        self.set_array(self.writer_index, (value as i16).to_be_bytes());
        self.writer_index += 2;
        self
    }

    pub fn write_int(&mut self, value: i32) -> &mut ByteBuf {
        self.ensure_writable0(4);
        self.set_array(self.writer_index, value.to_be_bytes());
        self.writer_index += 4;
        self
    }

    pub fn write_long(&mut self, value: i64) -> &mut ByteBuf {
        self.ensure_writable0(8);
        self.set_array(self.writer_index, value.to_be_bytes());
        self.writer_index += 8;
        self
    }

    /// A UTF-16 unit: the low 16 bits.
    pub fn write_char(&mut self, value: i32) -> &mut ByteBuf {
        self.write_short(value);
        self
    }

    /// `floatToRawIntBits`, so a NaN keeps its bits.
    pub fn write_float(&mut self, value: f32) -> &mut ByteBuf {
        self.write_int(value.to_bits() as i32);
        self
    }

    pub fn write_double(&mut self, value: f64) -> &mut ByteBuf {
        self.write_long(value.to_bits() as i64);
        self
    }

    /// `writeBytes(byte[])`.
    pub fn write_bytes_array(&mut self, src: &[i8]) -> &mut ByteBuf {
        self.write_bytes_array_range(src, 0, src.len() as i32);
        self
    }

    /// `writeBytes(byte[], int, int)`. It grows the array before it checks the range.
    pub fn write_bytes_array_range(
        &mut self,
        src: &[i8],
        src_index: i32,
        length: i32,
    ) -> &mut ByteBuf {
        self.ensure_writable(length);
        self.set_bytes_array(self.writer_index, src, src_index, length);
        self.writer_index += length;
        self
    }

    /// `writeBytes(ByteBuf)`: all of the source's readable bytes, which it reads.
    pub fn write_bytes_buf(&mut self, src: &mut ByteBuf) -> &mut ByteBuf {
        let length = src.readable_bytes();
        self.write_bytes_buf_range(src, src.reader_index, length);
        src.set_reader_index(src.reader_index + length);
        self
    }

    /// `writeBytes(ByteBuf, int, int)`: a range of the source's array, whatever its indexes. It
    /// grows the array before it checks the range.
    pub fn write_bytes_buf_range(
        &mut self,
        src: &ByteBuf,
        src_index: i32,
        length: i32,
    ) -> &mut ByteBuf {
        self.ensure_writable(length);
        self.set_bytes_array(self.writer_index, &src.array, src_index, length);
        self.writer_index += length;
        self
    }

    /// `slice(int, int)`: that range of the array as a buffer of its own, all of it readable, which
    /// can't grow. The range is checked against the array, not the writer index. Java's slice
    /// shares the array, so writes to either show in the other; a copy reads the same while
    /// neither is written to, and the game only reads a slice (`ByteBufCodecs.lengthPrefixed`).
    pub fn slice(&self, index: i32, length: i32) -> ByteBuf {
        // AbstractByteBuf(int)'s check of the slice's maximum capacity, then checkSliceOutOfBounds
        assert!(length >= 0, "maxCapacity : {length} (expected: >= 0)");
        check_range_bounds("index", index, length, self.capacity());
        ByteBuf::from_array(
            self.array[index as usize..(index + length) as usize].to_vec(),
            length,
        )
    }

    /// `toString(int, int, Charset)` with UTF-8, through `ByteBufUtil.decodeString`: the range is
    /// checked against the array, not the writer index, and only when it isn't empty.
    pub fn to_string_utf8(&self, index: i32, length: i32) -> String {
        if length == 0 {
            return String::new();
        }
        // String(byte[], int, int, Charset)'s checkBoundsOffCount
        assert!(
            index >= 0 && length >= 0 && index <= self.capacity() - length,
            "offset {index}, count {length}, length {}",
            self.capacity()
        );
        string::from_bytes_utf8(&self.array[index as usize..(index + length) as usize])
    }

    /// `getBytes(int, byte[], int, int)`, which checks both ranges against the arrays.
    pub fn get_bytes_array(
        &self,
        index: i32,
        dst: &mut [i8],
        dst_index: i32,
        length: i32,
    ) -> &ByteBuf {
        self.check_dst_index(index, length, dst_index, dst.len() as i32);
        dst[dst_index as usize..(dst_index + length) as usize]
            .copy_from_slice(&self.array[index as usize..(index + length) as usize]);
        self
    }

    /// `setBytes(int, byte[], int, int)`, which checks both ranges against the arrays.
    pub fn set_bytes_array(
        &mut self,
        index: i32,
        src: &[i8],
        src_index: i32,
        length: i32,
    ) -> &mut ByteBuf {
        self.check_src_index(index, length, src_index, src.len() as i32);
        self.array[index as usize..(index + length) as usize]
            .copy_from_slice(&src[src_index as usize..(src_index + length) as usize]);
        self
    }

    /// The bytes from `index` on, for HeapByteBufUtil's big-endian reads.
    fn get_array<const N: usize>(&self, index: i32) -> [u8; N] {
        let mut bytes = [0; N];
        for (byte, slot) in bytes.iter_mut().zip(&self.array[index as usize..]) {
            *byte = *slot as u8;
        }
        bytes
    }

    fn set_array<const N: usize>(&mut self, index: i32, bytes: [u8; N]) {
        for (slot, byte) in self.array[index as usize..].iter_mut().zip(bytes) {
            *slot = byte as i8;
        }
    }

    fn check_readable_bytes(&self, minimum_readable_bytes: i32) {
        assert!(
            minimum_readable_bytes >= 0,
            "minimumReadableBytes : {minimum_readable_bytes} (expected: >= 0)"
        );
        self.check_readable_bytes0(minimum_readable_bytes);
    }

    fn check_readable_bytes0(&self, minimum_readable_bytes: i32) {
        assert!(
            self.reader_index <= self.writer_index - minimum_readable_bytes,
            "readerIndex({}) + length({minimum_readable_bytes}) exceeds writerIndex({})",
            self.reader_index,
            self.writer_index
        );
    }

    fn check_index(&self, index: i32, field_length: i32) {
        check_range_bounds_trusted_capacity("index", index, field_length, self.capacity());
    }

    fn check_src_index(&self, index: i32, length: i32, src_index: i32, src_capacity: i32) {
        self.check_index(index, length);
        check_range_bounds("srcIndex", src_index, length, src_capacity);
    }

    fn check_dst_index(&self, index: i32, length: i32, dst_index: i32, dst_capacity: i32) {
        self.check_index(index, length);
        check_range_bounds("dstIndex", dst_index, length, dst_capacity);
    }

    fn check_new_capacity(&self, new_capacity: i32) {
        assert!(
            new_capacity >= 0 && new_capacity <= self.max_capacity,
            "newCapacity: {new_capacity} (expected: 0-{})",
            self.max_capacity
        );
    }

    // Called after a capacity reduction
    fn trim_indices_to_capacity(&mut self, new_capacity: i32) {
        if self.writer_index > new_capacity {
            self.reader_index = self.reader_index.min(new_capacity);
            self.writer_index = new_capacity;
        }
    }
}

/// A buffer is itself, for the codecs: a codec of a `ByteBuf` works on any buffer that is
/// `AsMut<ByteBuf>`, like Java's `StreamCodec<? super B, T>` (see `StreamCodec`).
impl AsMut<ByteBuf> for ByteBuf {
    fn as_mut(&mut self) -> &mut ByteBuf {
        self
    }
}

fn check_index_bounds(reader_index: i32, writer_index: i32, capacity: i32) {
    assert!(
        reader_index >= 0 && reader_index <= writer_index && writer_index <= capacity,
        "readerIndex: {reader_index}, writerIndex: {writer_index} (expected: 0 <= readerIndex <= writerIndex <= capacity({capacity}))"
    );
}

/// netty-common's `MathUtil.isOutOfBounds`, without its check for a negative capacity.
fn check_range_bounds_trusted_capacity(
    index_name: &str,
    index: i32,
    field_length: i32,
    capacity: i32,
) {
    let out_of_bounds = index < 0
        || field_length < 0
        || index.wrapping_add(field_length) < 0
        || index.wrapping_add(field_length) > capacity;
    assert!(
        !out_of_bounds,
        "{index_name}: {index}, length: {field_length} (expected: range(0, {capacity}))"
    );
}

/// netty-common's `MathUtil.isOutOfBounds`: any of them negative, or the end past the capacity.
fn check_range_bounds(index_name: &str, index: i32, field_length: i32, capacity: i32) {
    let end = index.wrapping_add(field_length);
    let out_of_bounds = (index | field_length | capacity | end | capacity.wrapping_sub(end)) < 0;
    assert!(
        !out_of_bounds,
        "{index_name}: {index}, length: {field_length} (expected: range(0, {capacity}))"
    );
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ByteBuf;
    use crate::golden::{Decode, Encode, Golden, elements};

    // [array, readerIndex, writerIndex, markedReaderIndex, markedWriterIndex], the array as hex.
    // Every buffer vanilla hands the suite is Unpooled.buffer's, which grows to Integer.MAX_VALUE.
    impl Decode for ByteBuf {
        fn decode(value: &Value) -> Self {
            let [
                array,
                reader_index,
                writer_index,
                marked_reader_index,
                marked_writer_index,
            ] = elements(value);
            let hex = array.as_str().expect("a buffer's array is hex");
            let array = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex") as i8)
                .collect();
            ByteBuf {
                array,
                reader_index: i32::decode(reader_index),
                writer_index: i32::decode(writer_index),
                marked_reader_index: i32::decode(marked_reader_index),
                marked_writer_index: i32::decode(marked_writer_index),
                max_capacity: i32::MAX,
            }
        }
    }

    impl Encode for ByteBuf {
        fn encode(&self) -> Value {
            let hex: String = self
                .array
                .iter()
                .map(|byte| format!("{:02x}", *byte as u8))
                .collect();
            Value::from(vec![
                Value::from(hex),
                Value::from(self.reader_index),
                Value::from(self.writer_index),
                Value::from(self.marked_reader_index),
                Value::from(self.marked_writer_index),
            ])
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("io.netty.buffer.ByteBuf");
        golden.check_method("capacity()I", |this: &mut ByteBuf| this.capacity());
        golden.check_method("maxCapacity()I", |this: &mut ByteBuf| this.max_capacity());
        golden.check_method("readerIndex()I", |this: &mut ByteBuf| this.reader_index());
        golden.check_method(
            "readerIndex(I)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, reader_index: i32| this.set_reader_index(reader_index).clone(),
        );
        golden.check_method("writerIndex()I", |this: &mut ByteBuf| this.writer_index());
        golden.check_method(
            "writerIndex(I)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, writer_index: i32| this.set_writer_index(writer_index).clone(),
        );
        golden.check_method(
            "setIndex(II)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, reader_index: i32, writer_index: i32| {
                this.set_index(reader_index, writer_index).clone()
            },
        );
        golden.check_method("clear()Lio/netty/buffer/ByteBuf;", |this: &mut ByteBuf| {
            this.clear().clone()
        });
        golden.check_method("isReadable()Z", |this: &mut ByteBuf| this.is_readable());
        golden.check_method("isReadable(I)Z", |this: &mut ByteBuf, num_bytes: i32| {
            this.is_readable_num_bytes(num_bytes)
        });
        golden.check_method("isWritable()Z", |this: &mut ByteBuf| this.is_writable());
        golden.check_method("isWritable(I)Z", |this: &mut ByteBuf, num_bytes: i32| {
            this.is_writable_num_bytes(num_bytes)
        });
        golden.check_method("readableBytes()I", |this: &mut ByteBuf| {
            this.readable_bytes()
        });
        golden.check_method("writableBytes()I", |this: &mut ByteBuf| {
            this.writable_bytes()
        });
        golden.check_method("maxWritableBytes()I", |this: &mut ByteBuf| {
            this.max_writable_bytes()
        });
        golden.check_method(
            "markReaderIndex()Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf| this.mark_reader_index().clone(),
        );
        golden.check_method(
            "resetReaderIndex()Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf| this.reset_reader_index().clone(),
        );
        golden.check_method(
            "markWriterIndex()Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf| this.mark_writer_index().clone(),
        );
        golden.check_method(
            "resetWriterIndex()Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf| this.reset_writer_index().clone(),
        );
        golden.check_method(
            "ensureWritable(I)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, min_writable_bytes: i32| {
                this.ensure_writable(min_writable_bytes).clone()
            },
        );
        golden.check_method("readBoolean()Z", |this: &mut ByteBuf| this.read_boolean());
        golden.check_method("readByte()B", |this: &mut ByteBuf| this.read_byte());
        golden.check_method("readUnsignedByte()S", |this: &mut ByteBuf| {
            this.read_unsigned_byte()
        });
        golden.check_method("readShort()S", |this: &mut ByteBuf| this.read_short());
        golden.check_method("readUnsignedShort()I", |this: &mut ByteBuf| {
            this.read_unsigned_short()
        });
        golden.check_method("readInt()I", |this: &mut ByteBuf| this.read_int());
        golden.check_method("readUnsignedInt()J", |this: &mut ByteBuf| {
            this.read_unsigned_int()
        });
        golden.check_method("readLong()J", |this: &mut ByteBuf| this.read_long());
        golden.check_method("readChar()C", |this: &mut ByteBuf| this.read_char());
        golden.check_method("readFloat()F", |this: &mut ByteBuf| this.read_float());
        golden.check_method("readDouble()D", |this: &mut ByteBuf| this.read_double());
        golden.check_method(
            "readBytes(I)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, length: i32| this.read_bytes(length),
        );
        golden.check_mut(
            "readBytes([B)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, dst: &mut Vec<i8>| this.read_bytes_array(dst).clone(),
        );
        golden.check_mut(
            "readBytes([BII)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, dst: &mut Vec<i8>, dst_index: &mut i32, length: &mut i32| {
                this.read_bytes_array_range(dst, *dst_index, *length)
                    .clone()
            },
        );
        golden.check_method(
            "skipBytes(I)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, length: i32| this.skip_bytes(length).clone(),
        );
        golden.check_method(
            "writeBoolean(Z)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, value: bool| this.write_boolean(value).clone(),
        );
        golden.check_method(
            "writeByte(I)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, value: i32| this.write_byte(value).clone(),
        );
        golden.check_method(
            "writeShort(I)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, value: i32| this.write_short(value).clone(),
        );
        golden.check_method(
            "writeInt(I)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, value: i32| this.write_int(value).clone(),
        );
        golden.check_method(
            "writeLong(J)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, value: i64| this.write_long(value).clone(),
        );
        golden.check_method(
            "writeChar(I)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, value: i32| this.write_char(value).clone(),
        );
        golden.check_method(
            "writeFloat(F)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, value: f32| this.write_float(value).clone(),
        );
        golden.check_method(
            "writeDouble(D)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, value: f64| this.write_double(value).clone(),
        );
        golden.check_method(
            "writeBytes([B)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, src: Vec<i8>| this.write_bytes_array(&src).clone(),
        );
        golden.check_method(
            "writeBytes([BII)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, src: Vec<i8>, src_index: i32, length: i32| {
                this.write_bytes_array_range(&src, src_index, length)
                    .clone()
            },
        );
        golden.check_mut(
            "writeBytes(Lio/netty/buffer/ByteBuf;)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, src: &mut ByteBuf| this.write_bytes_buf(src).clone(),
        );
        golden.check_method(
            "writeBytes(Lio/netty/buffer/ByteBuf;II)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, src: ByteBuf, src_index: i32, length: i32| {
                this.write_bytes_buf_range(&src, src_index, length).clone()
            },
        );
        golden.check_method(
            "toString(IILjava/nio/charset/Charset;)Ljava/lang/String;",
            |this: &mut ByteBuf, index: i32, length: i32, charset: String| {
                assert_eq!(charset, "UTF-8", "only UTF-8 is ported");
                this.to_string_utf8(index, length)
            },
        );
        golden.check_method(
            "slice(II)Lio/netty/buffer/ByteBuf;",
            |this: &mut ByteBuf, index: i32, length: i32| this.slice(index, length),
        );
        golden.finish();
    }
}
