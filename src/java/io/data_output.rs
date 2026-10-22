//! java/io/DataOutput.java
//!
//! Writes Java's primitives big-endian, and strings in modified UTF-8 (`writeUTF`). Only the
//! methods NBT calls are here. Java throws `UTFDataFormatException` for a string longer than 65535
//! bytes, which is a panic here.

/*
 * Copyright (c) 1995, 2021, Oracle and/or its affiliates. All rights reserved.
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

pub trait DataOutput {
    /// `write(byte[])`: all of the array.
    fn write(&mut self, b: &[i8]);

    /// `write(byte[], int, int)`: `len` bytes of the array from `off`.
    fn write_range(&mut self, b: &[i8], off: i32, len: i32);

    /// The low 8 bits.
    fn write_byte(&mut self, v: i32);

    /// The low 16 bits.
    fn write_short(&mut self, v: i32);

    fn write_int(&mut self, v: i32);

    fn write_long(&mut self, v: i64);

    fn write_float(&mut self, v: f32);

    fn write_double(&mut self, v: f64);

    /// The string in modified UTF-8, after its length in bytes as an unsigned short.
    fn write_utf(&mut self, s: &str);
}
