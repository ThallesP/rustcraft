//! org/apache/commons/lang3/StringUtils.java (commons-lang3 3.20.0)
//!
//! Only `normalizeSpace` and `isBlank`, which the chat screen runs a message through, written from
//! the jar's bytecode: the whitespace (`Character.isWhitespace`) at the ends gone and each run of
//! it inside made one space, a non-breaking space a space, then `String.trim()`. Strings are
//! UTF-16 units, like Java's.

/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Translated to Rust from commons-lang3's bytecode, and changed to fit the port.
 */

use crate::java::lang::character::Character;

pub struct StringUtils;

impl StringUtils {
    /// `normalizeSpace`.
    pub fn normalize_space(str: &str) -> String {
        if str.is_empty() {
            return String::new();
        }
        let units: Vec<u16> = str.encode_utf16().collect();
        let mut new_chars = Vec::with_capacity(units.len());
        let mut whitespaces_count = 0;
        let mut start_whitespaces = true;
        for &actual_char in &units {
            if Character::is_whitespace(actual_char) {
                if whitespaces_count == 0 && !start_whitespaces {
                    new_chars.push(u16::from(b' '));
                }
                whitespaces_count += 1;
            } else {
                start_whitespaces = false;
                new_chars.push(if actual_char == 160 { 32 } else { actual_char });
                whitespaces_count = 0;
            }
        }
        if start_whitespaces {
            return String::new();
        }
        let end = new_chars.len() - usize::from(whitespaces_count > 0);
        String::from_utf16_lossy(java_trim(&new_chars[..end]))
    }

    /// `isBlank`: empty, or only whitespace.
    pub fn is_blank(str: &str) -> bool {
        str.encode_utf16().all(Character::is_whitespace)
    }
}

/// `String.trim()`: the units up to the space at either end gone.
pub fn java_trim(units: &[u16]) -> &[u16] {
    let start = units
        .iter()
        .position(|&unit| unit > 32)
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|&unit| unit > 32)
        .map_or(start, |end| end + 1);
    &units[start..end]
}
