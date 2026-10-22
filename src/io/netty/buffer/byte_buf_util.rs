//! io/netty/buffer/ByteBufUtil.java
//!
//! Only the UTF-8 helpers `Utf8String` uses. A Rust string can't hold a lone surrogate, which
//! `writeUtf8` would write as `?`, so its UTF-8 is the string's own bytes.

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

pub struct ByteBufUtil;

impl ByteBufUtil {
    /// `CharsetUtil.encoder(UTF_8).maxBytesPerChar()`: a UTF-16 unit takes at most 3 bytes.
    const MAX_BYTES_PER_CHAR_UTF8: i32 = 3;

    /// Wraps for a length over a third of the int range, like Java's.
    pub fn utf8_max_bytes(seq_length: i32) -> i32 {
        seq_length.wrapping_mul(ByteBufUtil::MAX_BYTES_PER_CHAR_UTF8)
    }

    /// `utf8MaxBytes(CharSequence)`, by its length in UTF-16 units.
    pub fn utf8_max_bytes_seq(seq: &str) -> i32 {
        ByteBufUtil::utf8_max_bytes(seq.encode_utf16().count() as i32)
    }

    /// Reserves `utf8MaxBytes` after the writer index first, so the buffer grows by up to three
    /// bytes a unit, then writes the bytes and returns how many.
    pub fn write_utf8(buf: &mut ByteBuf, seq: &str) -> i32 {
        buf.ensure_writable0(ByteBufUtil::utf8_max_bytes_seq(seq));
        let bytes: Vec<i8> = seq.bytes().map(|byte| byte as i8).collect();
        let written = bytes.len() as i32;
        let writer_index = buf.writer_index();
        buf.set_bytes_array(writer_index, &bytes, 0, written);
        buf.set_writer_index(writer_index + written);
        written
    }
}
