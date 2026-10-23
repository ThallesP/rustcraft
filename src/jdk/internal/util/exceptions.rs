//! jdk/internal/util/Exceptions.java (OpenJDK 25): what an exception's message may show of a host
//! or an input, which `URI`'s parser puts in its `URISyntaxException`s. Only the non-socket
//! category is ported (`filterNonSocketInfo`, `enhancedNonSocketExceptions`), with `formatMsg` of
//! one argument and its `trim`.
//!
//! The JDK reads the categories from the `jdk.includeInExceptions` security property, whose
//! default (`conf/security/java.security`) is `hostInfoExclSocket`: non-socket information is
//! shown, the input and the index both. The game doesn't change it, so the port takes the default
//! as a constant. A JVM that isn't booted yet shows neither, which the game never meets.

/*
 * Copyright (c) 2018, 2025, Oracle and/or its affiliates. All rights reserved.
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

pub struct Exceptions;

impl Exceptions {
    /// Whether `jdk.includeInExceptions` has `hostInfoExclSocket` (or `hostInfo`), which it has
    /// by default.
    pub const fn enhanced_non_socket_exceptions() -> bool {
        true
    }

    /// `filterNonSocketInfo(host).output()`: the host when the category is enhanced, otherwise
    /// its replacement, which is empty unless a caller sets one (URI's doesn't).
    pub fn filter_non_socket_info(host: &str) -> &str {
        if Exceptions::enhanced_non_socket_exceptions() {
            host
        } else {
            ""
        }
    }

    /// `formatMsg("%s", info)`: the info's output, trimmed.
    pub fn format_msg(info: &str) -> String {
        Exceptions::trim(info)
    }

    /// Removes leading, trailing and duplicated spaces (U+0020 only).
    fn trim(s: &str) -> String {
        let mut sb = String::with_capacity(s.len());
        // initial value deals with leading spaces
        let mut in_space = true;
        for c in s.chars() {
            if c == ' ' {
                if in_space {
                    continue;
                }
                in_space = true;
            } else {
                in_space = false;
            }
            sb.push(c);
        }
        // last char could be a space
        if sb.ends_with(' ') {
            sb.pop();
        }
        sb
    }
}

#[cfg(test)]
mod tests {
    use super::Exceptions;

    #[test]
    fn trims_spaces_like_java() {
        // What `Exceptions.trim` gives, run on the JDK 25. Only U+0020 counts: a tab stays.
        assert_eq!(Exceptions::format_msg(""), "");
        assert_eq!(Exceptions::format_msg("   "), "");
        assert_eq!(Exceptions::format_msg("  a  b  "), "a b");
        assert_eq!(Exceptions::format_msg("a \t b"), "a \t b");
        assert_eq!(Exceptions::format_msg(" é  😀 "), "é 😀");
    }
}
