//! java/io/DataOutputStream.java
//!
//! Only `writeUTF(String, DataOutput)`, which every `DataOutput`'s `writeUTF` runs (Netty's
//! `ByteBufOutputStream.writeUTF` through a `DataOutputStream` of itself), with
//! `jdk.internal.util.ModifiedUtf`'s `utfLen` and `putChar`. The stream itself isn't ported: the
//! game writes NBT to Netty's buffers.

/*
 * Copyright (c) 1994, 2024, Oracle and/or its affiliates. All rights reserved.
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

/*
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

use crate::java::io::data_output::DataOutput;

pub struct DataOutputStream;

impl DataOutputStream {
    /// The string in modified UTF-8 after its length as an unsigned short, in one `write` of
    /// them all. Each UTF-16 unit is one byte from U+0001 to U+007F, two up to U+07FF and for
    /// U+0000, and three above, a supplementary character's two surrogates three each. Longer
    /// than 65535 bytes panics before anything's written, where Java throws
    /// `UTFDataFormatException`. Returns how many bytes it wrote.
    pub fn write_utf(s: &str, out: &mut dyn DataOutput) -> i32 {
        let utflen = DataOutputStream::utf_len(s);
        assert!(utflen <= 65535, "encoded string too long: {utflen} bytes");

        let mut bytearr: Vec<i8> = Vec::with_capacity(utflen as usize + 2);
        bytearr.extend((utflen as u16).to_be_bytes().map(|byte| byte as i8));
        for c in s.encode_utf16() {
            if c != 0 && c < 0x80 {
                bytearr.push(c as i8);
            } else if c >= 0x800 {
                bytearr.push((0xE0 | ((c >> 12) & 0x0F)) as i8);
                bytearr.push((0x80 | ((c >> 6) & 0x3F)) as i8);
                bytearr.push((0x80 | (c & 0x3F)) as i8);
            } else {
                bytearr.push((0xC0 | ((c >> 6) & 0x1F)) as i8);
                bytearr.push((0x80 | (c & 0x3F)) as i8);
            }
        }
        out.write_range(&bytearr, 0, utflen + 2);
        utflen + 2
    }

    /// `ModifiedUtf.utfLen`: how many bytes `write_utf` writes of the string, without its length.
    pub fn utf_len(s: &str) -> i32 {
        s.encode_utf16().fold(0i32, |utflen, c| {
            utflen.wrapping_add(if c != 0 && c < 0x80 {
                1
            } else if c >= 0x800 {
                3
            } else {
                2
            })
        })
    }
}
