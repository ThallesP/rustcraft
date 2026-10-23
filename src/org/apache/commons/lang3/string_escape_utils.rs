//! org/apache/commons/lang3/StringEscapeUtils.java (commons-lang3 3.20.0)
//!
//! Only `escapeJava`, which `IdentifierException` runs its message through: `ESCAPE_JAVA`'s
//! translators, the quote and backslash escaped, the control characters Java has escapes for as
//! them, and everything else outside the printable ASCII range as `\uXXXX` (upper-case hex), a
//! supplementary character as its two UTF-16 units. Written from commons-lang's source, which
//! isn't among the game's (it ships the jar).

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
 * Translated to Rust for rustcraft, and changed: only escapeJava.
 */

use std::fmt::Write;

pub struct StringEscapeUtils;

impl StringEscapeUtils {
    pub fn escape_java(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        for c in input.chars() {
            match c {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\u{8}' => result.push_str("\\b"),
                '\n' => result.push_str("\\n"),
                '\t' => result.push_str("\\t"),
                '\u{c}' => result.push_str("\\f"),
                '\r' => result.push_str("\\r"),
                ' '..='\u{7f}' => result.push(c),
                _ => {
                    let mut units = [0u16; 2];
                    for unit in c.encode_utf16(&mut units) {
                        let _ = write!(result, "\\u{:04X}", unit);
                    }
                }
            }
        }
        result
    }
}
