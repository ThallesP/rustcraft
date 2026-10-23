//! java/net/URISyntaxException.java: a string that doesn't parse as a URI, why, and where. Its
//! message (`getMessage`, which `Display` is) is the reason, then ` at index ` and the index when
//! it's known, then `: ` and the input. Indices count UTF-16 units, like Java's. `Display` leaves
//! out the class name `Throwable.toString` would put first, like the port's other exceptions.

/*
 * Copyright (c) 2000, 2025, Oracle and/or its affiliates. All rights reserved.
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

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct URISyntaxException {
    input: String,
    /// `Throwable`'s message, which `getReason` returns.
    reason: String,
    index: i32,
}

impl URISyntaxException {
    /// `URISyntaxException(input, reason)`, whose index isn't known: -1.
    pub fn new(input: &str, reason: &str) -> URISyntaxException {
        URISyntaxException::with_index(input, reason, -1)
    }

    /// `URISyntaxException(input, reason, index)`. Panics on an index below -1, where Java throws
    /// `IllegalArgumentException`.
    pub fn with_index(input: &str, reason: &str, index: i32) -> URISyntaxException {
        assert!(index >= -1, "IllegalArgumentException: index {index}");
        URISyntaxException {
            input: input.to_owned(),
            reason: reason.to_owned(),
            index,
        }
    }

    pub fn get_input(&self) -> &str {
        &self.input
    }

    pub fn get_reason(&self) -> &str {
        &self.reason
    }

    pub fn get_index(&self) -> i32 {
        self.index
    }
}

/// `getMessage()`.
impl fmt::Display for URISyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_reason())?;
        if self.index > -1 {
            write!(f, " at index {}", self.index)?;
        }
        write!(f, ": {}", self.input)
    }
}

impl std::error::Error for URISyntaxException {}

#[cfg(test)]
mod tests {
    use super::URISyntaxException;

    #[test]
    fn prints_its_message_like_java() {
        // `java.net.URI`'s goldens check the parser's messages; these are the constructors'.
        let at = URISyntaxException::with_index("a b", "Illegal character in path", 1);
        assert_eq!(at.to_string(), "Illegal character in path at index 1: a b");
        assert_eq!(
            (at.get_input(), at.get_reason(), at.get_index()),
            ("a b", "Illegal character in path", 1)
        );
        let unknown = URISyntaxException::new("ftp://h", "Unsupported protocol in URI: ftp://h");
        assert_eq!(
            unknown.to_string(),
            "Unsupported protocol in URI: ftp://h: ftp://h"
        );
        assert_eq!(unknown.get_index(), -1);
    }
}
