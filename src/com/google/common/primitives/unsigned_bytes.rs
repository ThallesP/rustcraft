//! com/google/common/primitives/UnsignedBytes.java
//!
//! Only `parseUnsignedByte(String, int)`, which SNBT's unsigned bytes parse with. Checked through
//! `net.minecraft.nbt.TagParser`'s goldens.

/*
 * Copyright (C) 2009 The Guava Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
 * in compliance with the License. You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software distributed under the License
 * is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
 * or implied. See the License for the specific language governing permissions and limitations under
 * the License.
 *
 * Translated to Rust and changed for rustcraft.
 */

use crate::java::lang::integer::Integer;
use crate::java::lang::number_format_exception::NumberFormatException;

pub struct UnsignedBytes;

impl UnsignedBytes {
    /// `parseUnsignedByte(String, int)`: `Integer.parseInt` in the radix, which must come out
    /// from 0 to 255, as the byte with those bits.
    pub fn parse_unsigned_byte(string: &str, radix: i32) -> Result<i8, NumberFormatException> {
        let parse = Integer::parse_int_radix(string, radix)?;
        if parse >> 8 == 0 {
            Ok(parse as i8)
        } else {
            Err(NumberFormatException::new(format!("out of range: {parse}")))
        }
    }
}
