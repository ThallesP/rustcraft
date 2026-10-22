//! java/io/DataInputStream.java
//!
//! Only `readUTF(DataInput)`, which every `DataInput`'s `readUTF` runs (Netty's
//! `ByteBufInputStream.readUTF` included). The stream itself isn't ported: the game reads NBT from
//! Netty's buffers.

/*
 * Copyright (c) 1994, 2025, Oracle and/or its affiliates. All rights reserved.
 * Copyright (c) 2024, Alibaba Group Holding Limited. All Rights Reserved.
 * DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
 *
 * This code is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License version 2 only, as
 * published by the Free Software Foundation.  Oracle designates this
 * particular file as subject to the "Classpath" exception as provided
 * by Oracle in the LICENSE file that accompanied this code.
 *
 * This code is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
 * version 2 for more details (a copy is included in the LICENSE file that
 * accompanied this code).
 *
 * You should have received a copy of the GNU General Public License version
 * 2 along with this work; if not, write to the Free Software Foundation,
 * Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * Please contact Oracle, 500 Oracle Parkway, Redwood Shores, CA 94065 USA
 * or visit www.oracle.com if you need additional information or have any
 * questions.
 */

use crate::java::io::data_input::DataInput;

pub struct DataInputStream;

impl DataInputStream {
    /// Modified UTF-8 after its length as an unsigned short: all the bytes are read first, then
    /// decoded. A character is one byte below 0x80 (U+0000 too, which modified UTF-8 otherwise
    /// writes as two), two or three bytes like UTF-8's, overlong ones included, and a
    /// supplementary character is its two surrogates, three bytes each. Anything else (a lone
    /// continuation byte, a lead byte of four, a continuation that isn't one, a character cut
    /// short by the length) panics where Java throws `UTFDataFormatException`, after the bytes
    /// are read.
    ///
    /// Java's strings can hold a surrogate that isn't in a pair, which a Rust string can't: it
    /// reads as U+FFFD. The golden harness can't hold one either, so that isn't checked.
    pub fn read_utf(input: &mut dyn DataInput) -> String {
        let utflen = input.read_unsigned_short();
        let mut bytearr = vec![0i8; utflen as usize];
        input.read_fully_range(&mut bytearr, 0, utflen);

        let mut chararr: Vec<u16> = Vec::with_capacity(utflen as usize);
        let mut count = 0;
        while count < utflen {
            let c = i32::from(bytearr[count as usize]) & 0xff;
            match c >> 4 {
                0..=7 => {
                    count += 1;
                    chararr.push(c as u16);
                }
                12 | 13 => {
                    count += 2;
                    assert!(count <= utflen, "malformed input: partial character at end");
                    let char2 = i32::from(bytearr[(count - 1) as usize]);
                    assert!(char2 & 0xC0 == 0x80, "malformed input around byte {count}");
                    chararr.push((((c & 0x1F) << 6) | (char2 & 0x3F)) as u16);
                }
                14 => {
                    count += 3;
                    assert!(count <= utflen, "malformed input: partial character at end");
                    let char2 = i32::from(bytearr[(count - 2) as usize]);
                    let char3 = i32::from(bytearr[(count - 1) as usize]);
                    assert!(
                        char2 & 0xC0 == 0x80 && char3 & 0xC0 == 0x80,
                        "malformed input around byte {}",
                        count - 1
                    );
                    chararr
                        .push((((c & 0x0F) << 12) | ((char2 & 0x3F) << 6) | (char3 & 0x3F)) as u16);
                }
                _ => panic!("malformed input around byte {count}"),
            }
        }
        String::from_utf16_lossy(&chararr)
    }
}
