//! java/io/DataInput.java
//!
//! Reads Java's primitives big-endian, and strings in modified UTF-8 (`readUTF`). Only the methods
//! NBT calls are here. Java throws `EOFException` when the bytes run out and
//! `UTFDataFormatException` for malformed modified UTF-8, which are panics here.

/*
 * Copyright (c) 1995, 2022, Oracle and/or its affiliates. All rights reserved.
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

pub trait DataInput {
    /// `readFully(byte[])`: fills the array.
    fn read_fully(&mut self, b: &mut [i8]);

    /// `readFully(byte[], int, int)`: `len` bytes into the array from `off`.
    fn read_fully_range(&mut self, b: &mut [i8], off: i32, len: i32);

    /// Skips up to `n` bytes, and returns how many it skipped.
    fn skip_bytes(&mut self, n: i32) -> i32;

    fn read_byte(&mut self) -> i8;

    fn read_unsigned_byte(&mut self) -> i32;

    fn read_short(&mut self) -> i16;

    fn read_unsigned_short(&mut self) -> i32;

    fn read_int(&mut self) -> i32;

    fn read_long(&mut self) -> i64;

    fn read_float(&mut self) -> f32;

    fn read_double(&mut self) -> f64;

    /// A string in modified UTF-8, after its length in bytes as an unsigned short.
    fn read_utf(&mut self) -> String;
}
