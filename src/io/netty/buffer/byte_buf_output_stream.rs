//! io/netty/buffer/ByteBufOutputStream.java
//!
//! A `DataOutput` writing to a buffer, which the game writes a packet's NBT through
//! (`FriendlyByteBuf.writeNbt`). Only what `DataOutput` asks of it and `buffer()` are ported:
//! `writeUTF` goes through a `DataOutputStream` of the stream itself in Java, which writes the
//! string's bytes in one `write`, like `DataOutputStream::write_utf` does here.

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
use crate::java::io::data_output::DataOutput;
use crate::java::io::data_output_stream::DataOutputStream;

/// Writes to the buffer at its writer index, growing it.
pub struct ByteBufOutputStream<'a> {
    buffer: &'a mut ByteBuf,
}

impl<'a> ByteBufOutputStream<'a> {
    pub fn new(buffer: &'a mut ByteBuf) -> ByteBufOutputStream<'a> {
        ByteBufOutputStream { buffer }
    }

    pub fn buffer(&mut self) -> &mut ByteBuf {
        self.buffer
    }
}

impl DataOutput for ByteBufOutputStream<'_> {
    fn write(&mut self, b: &[i8]) {
        self.buffer.write_bytes_array(b);
    }

    fn write_range(&mut self, b: &[i8], off: i32, len: i32) {
        self.buffer.write_bytes_array_range(b, off, len);
    }

    fn write_byte(&mut self, v: i32) {
        self.buffer.write_byte(v);
    }

    fn write_short(&mut self, v: i32) {
        self.buffer.write_short(i32::from(v as i16));
    }

    fn write_int(&mut self, v: i32) {
        self.buffer.write_int(v);
    }

    fn write_long(&mut self, v: i64) {
        self.buffer.write_long(v);
    }

    fn write_float(&mut self, v: f32) {
        self.buffer.write_float(v);
    }

    fn write_double(&mut self, v: f64) {
        self.buffer.write_double(v);
    }

    fn write_utf(&mut self, s: &str) {
        DataOutputStream::write_utf(s, self);
    }
}

#[cfg(test)]
mod tests {
    use super::ByteBufOutputStream;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::java::io::data_output::DataOutput;

    // A stream is the buffer it writes, which each case makes it over.
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("io.netty.buffer.ByteBufOutputStream");
        golden.check_mut("writeByte(I)V", |buffer: &mut ByteBuf, v: &mut i32| {
            ByteBufOutputStream::new(buffer).write_byte(*v)
        });
        golden.check_mut("writeShort(I)V", |buffer: &mut ByteBuf, v: &mut i32| {
            ByteBufOutputStream::new(buffer).write_short(*v)
        });
        golden.check_mut("writeInt(I)V", |buffer: &mut ByteBuf, v: &mut i32| {
            ByteBufOutputStream::new(buffer).write_int(*v)
        });
        golden.check_mut("writeLong(J)V", |buffer: &mut ByteBuf, v: &mut i64| {
            ByteBufOutputStream::new(buffer).write_long(*v)
        });
        golden.check_mut("writeFloat(F)V", |buffer: &mut ByteBuf, v: &mut f32| {
            ByteBufOutputStream::new(buffer).write_float(*v)
        });
        golden.check_mut("writeDouble(D)V", |buffer: &mut ByteBuf, v: &mut f64| {
            ByteBufOutputStream::new(buffer).write_double(*v)
        });
        golden.check_mut("write([B)V", |buffer: &mut ByteBuf, b: &mut Vec<i8>| {
            ByteBufOutputStream::new(buffer).write(b)
        });
        golden.check_mut(
            "write([BII)V",
            |buffer: &mut ByteBuf, b: &mut Vec<i8>, off: &mut i32, len: &mut i32| {
                ByteBufOutputStream::new(buffer).write_range(b, *off, *len)
            },
        );
        golden.check_mut(
            "writeUTF(Ljava/lang/String;)V",
            |buffer: &mut ByteBuf, s: &mut String| ByteBufOutputStream::new(buffer).write_utf(s),
        );
        golden.finish();
    }
}
