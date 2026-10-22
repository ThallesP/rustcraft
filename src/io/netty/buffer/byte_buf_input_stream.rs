//! io/netty/buffer/ByteBufInputStream.java
//!
//! A `DataInput` over a buffer's readable bytes, which the game reads a packet's NBT through
//! (`FriendlyByteBuf.readNbt`). Only what `DataInput` asks of it, `available` and the
//! constructor taking just the buffer are ported: `InputStream`'s methods, `readLine`, marks and
//! closing aren't called.
//!
//! Java's `EOFException` and `IndexOutOfBoundsException` are panics here.

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

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::io::data_input::DataInput;
use crate::java::io::data_input_stream::DataInputStream;

/// Reads the buffer from its reader index up to where its writer index was when the stream was
/// made.
pub struct ByteBufInputStream<'a> {
    buffer: &'a mut ByteBuf,
    end_index: i32,
}

impl<'a> ByteBufInputStream<'a> {
    /// Marks the buffer's reader index, so a stream overwrites any mark the buffer had.
    pub fn new(buffer: &'a mut ByteBuf) -> ByteBufInputStream<'a> {
        let length = buffer.readable_bytes();
        let start_index = buffer.reader_index();
        buffer.mark_reader_index();
        ByteBufInputStream {
            buffer,
            end_index: start_index + length,
        }
    }

    /// How many bytes are left to read.
    pub fn available(&self) -> i32 {
        self.end_index - self.buffer.reader_index()
    }

    /// Panics, before reading anything, unless `field_size` bytes are left.
    fn check_available(&self, field_size: i32) {
        assert!(field_size >= 0, "fieldSize cannot be a negative number");
        assert!(
            field_size <= self.available(),
            "fieldSize is too long! Length is {field_size}, but maximum is {}",
            self.available()
        );
    }
}

impl DataInput for ByteBufInputStream<'_> {
    fn read_fully(&mut self, b: &mut [i8]) {
        let len = b.len() as i32;
        self.read_fully_range(b, 0, len);
    }

    fn read_fully_range(&mut self, b: &mut [i8], off: i32, len: i32) {
        self.check_available(len);
        self.buffer.read_bytes_array_range(b, off, len);
    }

    /// As many as are left, at most `n`; a negative `n` panics in the buffer's `skipBytes`.
    fn skip_bytes(&mut self, n: i32) -> i32 {
        let n_bytes = self.available().min(n);
        self.buffer.skip_bytes(n_bytes);
        n_bytes
    }

    fn read_byte(&mut self) -> i8 {
        self.check_available(1);
        self.buffer.read_byte()
    }

    fn read_unsigned_byte(&mut self) -> i32 {
        i32::from(self.read_byte()) & 0xff
    }

    fn read_short(&mut self) -> i16 {
        self.check_available(2);
        self.buffer.read_short()
    }

    fn read_unsigned_short(&mut self) -> i32 {
        i32::from(self.read_short()) & 0xffff
    }

    fn read_int(&mut self) -> i32 {
        self.check_available(4);
        self.buffer.read_int()
    }

    fn read_long(&mut self) -> i64 {
        self.check_available(8);
        self.buffer.read_long()
    }

    fn read_float(&mut self) -> f32 {
        f32::from_bits(self.read_int() as u32)
    }

    fn read_double(&mut self) -> f64 {
        f64::from_bits(self.read_long() as u64)
    }

    fn read_utf(&mut self) -> String {
        DataInputStream::read_utf(self)
    }
}

#[cfg(test)]
mod tests {
    use super::ByteBufInputStream;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::java::io::data_input::DataInput;

    // A stream is the buffer it reads, which each case makes it over.
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("io.netty.buffer.ByteBufInputStream");
        golden.check_mut("available()I", |buffer: &mut ByteBuf| {
            ByteBufInputStream::new(buffer).available()
        });
        golden.check_mut("readByte()B", |buffer: &mut ByteBuf| {
            ByteBufInputStream::new(buffer).read_byte()
        });
        golden.check_mut("readUnsignedByte()I", |buffer: &mut ByteBuf| {
            ByteBufInputStream::new(buffer).read_unsigned_byte()
        });
        golden.check_mut("readShort()S", |buffer: &mut ByteBuf| {
            ByteBufInputStream::new(buffer).read_short()
        });
        golden.check_mut("readUnsignedShort()I", |buffer: &mut ByteBuf| {
            ByteBufInputStream::new(buffer).read_unsigned_short()
        });
        golden.check_mut("readInt()I", |buffer: &mut ByteBuf| {
            ByteBufInputStream::new(buffer).read_int()
        });
        golden.check_mut("readLong()J", |buffer: &mut ByteBuf| {
            ByteBufInputStream::new(buffer).read_long()
        });
        golden.check_mut("readFloat()F", |buffer: &mut ByteBuf| {
            ByteBufInputStream::new(buffer).read_float()
        });
        golden.check_mut("readDouble()D", |buffer: &mut ByteBuf| {
            ByteBufInputStream::new(buffer).read_double()
        });
        golden.check_mut("readFully([B)V", |buffer: &mut ByteBuf, b: &mut Vec<i8>| {
            ByteBufInputStream::new(buffer).read_fully(b)
        });
        golden.check_mut(
            "readFully([BII)V",
            |buffer: &mut ByteBuf, b: &mut Vec<i8>, off: &mut i32, len: &mut i32| {
                ByteBufInputStream::new(buffer).read_fully_range(b, *off, *len)
            },
        );
        golden.check_mut("skipBytes(I)I", |buffer: &mut ByteBuf, n: &mut i32| {
            ByteBufInputStream::new(buffer).skip_bytes(*n)
        });
        golden.check_mut("readUTF()Ljava/lang/String;", |buffer: &mut ByteBuf| {
            ByteBufInputStream::new(buffer).read_utf()
        });
        golden.finish();
    }
}
