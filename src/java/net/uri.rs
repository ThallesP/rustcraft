//! java/net/URI.java (OpenJDK 25): what the game parses links with. A text component's
//! `open_url` click event decodes its string with `ExtraCodecs.UNTRUSTED_URI`, which is
//! `Util.parseAndValidateUntrustedUri`'s `new URI(string)`, and the game's constants are
//! `URI.create`'s.
//!
//! Ported: `URI(String)`, which is `new Parser(str).parse(false)` and everything it reaches, and
//! `create`, which panics where Java throws `IllegalArgumentException` (with the same message);
//! the getters, raw and decoded; `isAbsolute`, `isOpaque`, `toString` (the string it was made
//! from), `equals` (`PartialEq`), `hashCode` and `compareTo`, which compare the hex digits of
//! escapes, the scheme and a server's host without regard to case.
//!
//! The parser reads the input's UTF-16 units, so the indices in its messages are Java's, and the
//! components it cuts out are Rust strings: each starts and ends at an ASCII delimiter or an end
//! of the input, never inside a surrogate pair. Java's URI allows a lone surrogate wherever it
//! allows other non-ASCII characters, which a Rust string can't hold, so no such input comes in.
//! A message shows the input trimmed of spaces (`Exceptions.formatMsg`) and the index, as under
//! the JDK's default `jdk.includeInExceptions`.
//!
//! Java keeps the hash and the decoded components in fields once it has worked them out, and
//! `getRawSchemeSpecificPart` a hierarchical URI's part; the port works them out each time, which
//! gives the same answers, since a URI never changes. A URI here is only ever made from a string,
//! so what Java builds from the components when it has no string (`defineString`,
//! `appendSchemeSpecificPart`) isn't needed.
//!
//! Not ported: the multi-argument constructors and what only they reach (`quote`, `checkPath`,
//! `toString(…)`), `parseServerAuthority` (`parse(true)`, which the parser keeps the flag for),
//! `normalize`, `resolve`, `relativize`, `toURL`, `toASCIIString` (`encode`, which normalizes to
//! NFC) and serialization.

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

use std::borrow::Cow;
use std::fmt;

use crate::java::lang::character::Character;
use crate::java::lang::integer::Integer;
use crate::java::lang::string::{JavaString, from_bytes_utf8};
use crate::java::net::uri_syntax_exception::URISyntaxException;
use crate::jdk::internal::util::exceptions::Exceptions;

#[derive(Clone, Debug)]
pub struct URI {
    // Components of all URIs: [<scheme>:]<scheme-specific-part>[#<fragment>]
    scheme: Option<String>, // None ==> relative URI
    fragment: Option<String>,

    // Hierarchical URI components: [//<authority>]<path>[?<query>]
    authority: Option<String>, // Registry or server

    // Server-based authority: [<userInfo>@]<host>[:<port>]
    user_info: Option<String>,
    host: Option<String>, // None ==> registry-based
    port: i32,            // -1 ==> undefined

    // Remaining components of hierarchical URIs
    path: Option<String>, // None ==> opaque
    query: Option<String>,

    // What the parser sets for an opaque URI. A hierarchical one's is cut from `string` when
    // it's asked for.
    scheme_specific_part: Option<String>,

    // The string form of this URI
    string: String,
}

impl URI {
    /// `URI(String)`: parses the string as RFC 2396 says, with Java's deviations (an empty
    /// authority before a path, query or fragment; empty relative paths; IPv4 parts of at most
    /// three digits and 255; a single-label hostname starting with a digit; IPv6 hosts in
    /// brackets, with a scope id; unescaped non-ASCII characters that aren't spaces or controls
    /// wherever escapes are allowed).
    pub fn new(str: &str) -> Result<URI, URISyntaxException> {
        let mut uri = URI {
            scheme: None,
            fragment: None,
            authority: None,
            user_info: None,
            host: None,
            port: -1,
            path: None,
            query: None,
            scheme_specific_part: None,
            string: String::new(),
        };
        Parser::new(&mut uri, str).parse(false)?;
        Ok(uri)
    }

    /// `URI.create(String)`: panics with the `URISyntaxException`'s message where Java throws an
    /// `IllegalArgumentException` with it.
    pub fn create(str: &str) -> URI {
        URI::new(str).unwrap_or_else(|x| panic!("{x}"))
    }

    // -- Component access methods --

    pub fn get_scheme(&self) -> Option<&str> {
        self.scheme.as_deref()
    }

    pub fn is_absolute(&self) -> bool {
        self.scheme.is_some()
    }

    pub fn is_opaque(&self) -> bool {
        self.path.is_none()
    }

    /// The opaque part the parser cut out, or the string between the scheme's `:` and the
    /// fragment's `#`.
    pub fn get_raw_scheme_specific_part(&self) -> &str {
        if let Some(part) = &self.scheme_specific_part {
            return part;
        }

        // if string is defined, components will have been parsed
        let s = &self.string;
        let mut start = 0;
        let mut end = s.len();
        if let Some(scheme) = &self.scheme {
            start = scheme.len() + 1;
        }
        if let Some(fragment) = &self.fragment {
            end -= fragment.len() + 1;
        }
        &s[start..end]
    }

    pub fn get_scheme_specific_part(&self) -> Cow<'_, str> {
        decode(self.get_raw_scheme_specific_part())
    }

    pub fn get_raw_authority(&self) -> Option<&str> {
        self.authority.as_deref()
    }

    pub fn get_authority(&self) -> Option<Cow<'_, str>> {
        self.authority.as_deref().map(decode)
    }

    pub fn get_raw_user_info(&self) -> Option<&str> {
        self.user_info.as_deref()
    }

    pub fn get_user_info(&self) -> Option<Cow<'_, str>> {
        self.user_info.as_deref().map(decode)
    }

    pub fn get_host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    pub fn get_port(&self) -> i32 {
        self.port
    }

    pub fn get_raw_path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn get_path(&self) -> Option<Cow<'_, str>> {
        self.path.as_deref().map(decode)
    }

    pub fn get_raw_query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn get_query(&self) -> Option<Cow<'_, str>> {
        self.query
            .as_deref()
            .map(|query| decode_ignoring(query, false))
    }

    pub fn get_raw_fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    pub fn get_fragment(&self) -> Option<Cow<'_, str>> {
        self.fragment
            .as_deref()
            .map(|fragment| decode_ignoring(fragment, false))
    }

    // -- Equality, comparison, hash code, toString --

    /// `hashCode()`: the scheme and a server's host without regard to case, and the hex digits
    /// of escapes upper-cased.
    pub fn hash_code(&self) -> i32 {
        let mut h = hash_ignoring_case(0, self.scheme.as_deref());
        h = hash(h, self.fragment.as_deref());
        if self.is_opaque() {
            h = hash(h, self.scheme_specific_part.as_deref());
        } else {
            h = hash(h, self.path.as_deref());
            h = hash(h, self.query.as_deref());
            if self.host.is_some() {
                h = hash(h, self.user_info.as_deref());
                h = hash_ignoring_case(h, self.host.as_deref());
                h = h.wrapping_add(1949i32.wrapping_mul(self.port));
            } else {
                h = hash(h, self.authority.as_deref());
            }
        }
        h
    }

    /// `compareTo(URI)`: by scheme (without regard to case), hierarchical before opaque, then
    /// part by part, an undefined component before a defined one.
    pub fn compare_to(&self, that: &URI) -> i32 {
        let c = compare_ignoring_case(self.scheme.as_deref(), that.scheme.as_deref());
        if c != 0 {
            return c;
        }

        if self.is_opaque() {
            if that.is_opaque() {
                // Both opaque
                let c = compare(
                    self.scheme_specific_part.as_deref(),
                    that.scheme_specific_part.as_deref(),
                );
                if c != 0 {
                    return c;
                }
                return compare(self.fragment.as_deref(), that.fragment.as_deref());
            }
            return 1; // Opaque > hierarchical
        } else if that.is_opaque() {
            return -1; // Hierarchical < opaque
        }

        // Hierarchical
        if self.host.is_some() && that.host.is_some() {
            // Both server-based
            let c = compare(self.user_info.as_deref(), that.user_info.as_deref());
            if c != 0 {
                return c;
            }
            let c = compare_ignoring_case(self.host.as_deref(), that.host.as_deref());
            if c != 0 {
                return c;
            }
            let c = self.port.wrapping_sub(that.port);
            if c != 0 {
                return c;
            }
        } else {
            // If one or both authorities are registry-based then we simply compare them in the
            // usual, case-sensitive way. If one is registry-based and one is server-based then
            // the strings are guaranteed to be unequal, hence the comparison will never return
            // zero and the compareTo and equals methods will remain consistent.
            let c = compare(self.authority.as_deref(), that.authority.as_deref());
            if c != 0 {
                return c;
            }
        }

        let c = compare(self.path.as_deref(), that.path.as_deref());
        if c != 0 {
            return c;
        }
        let c = compare(self.query.as_deref(), that.query.as_deref());
        if c != 0 {
            return c;
        }
        compare(self.fragment.as_deref(), that.fragment.as_deref())
    }
}

/// `equals(Object)`: both opaque or both hierarchical, the schemes equal without regard to
/// case, then the parts, the hex digits of escapes without regard to case. Server-based
/// authorities compare their user info, host (without regard to case) and port, not their text:
/// `http://h:/` equals `http://h/`.
impl PartialEq for URI {
    fn eq(&self, that: &URI) -> bool {
        if self.is_opaque() != that.is_opaque() {
            return false;
        }
        if !equal_ignoring_case(self.scheme.as_deref(), that.scheme.as_deref()) {
            return false;
        }
        if !equal(self.fragment.as_deref(), that.fragment.as_deref()) {
            return false;
        }

        // Opaque
        if self.is_opaque() {
            return equal(
                self.scheme_specific_part.as_deref(),
                that.scheme_specific_part.as_deref(),
            );
        }

        // Hierarchical
        if !equal(self.path.as_deref(), that.path.as_deref()) {
            return false;
        }
        if !equal(self.query.as_deref(), that.query.as_deref()) {
            return false;
        }

        // Authorities. Java compares them by reference first, which two URIs only share when
        // neither has one: each parse cuts its own.
        if self.authority.is_none() && that.authority.is_none() {
            return true;
        }
        if self.host.is_some() {
            // Server-based
            if !equal(self.user_info.as_deref(), that.user_info.as_deref()) {
                return false;
            }
            if !equal_ignoring_case(self.host.as_deref(), that.host.as_deref()) {
                return false;
            }
            if self.port != that.port {
                return false;
            }
        } else if self.authority.is_some() {
            // Registry-based
            if !equal(self.authority.as_deref(), that.authority.as_deref()) {
                return false;
            }
        } else if that.authority.is_some() {
            return false;
        }

        true
    }
}

impl Eq for URI {}

/// `toString()`: the string the URI was parsed from.
impl fmt::Display for URI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.string)
    }
}

// -- Utility methods for string-field comparison and hashing --

// These methods return appropriate values for undefined string arguments, thereby simplifying
// the equals, hashCode, and compareTo methods.
//
// The case-ignoring methods should only be applied to strings whose characters are all known to
// be US-ASCII.

const PERCENT: u16 = b'%' as u16;

// US-ASCII only
fn to_lower(c: u16) -> i32 {
    if (u16::from(b'A')..=u16::from(b'Z')).contains(&c) {
        return i32::from(c) + i32::from(b'a' - b'A');
    }
    i32::from(c)
}

// US-ASCII only
fn to_upper(c: u16) -> i32 {
    if (u16::from(b'a')..=u16::from(b'z')).contains(&c) {
        return i32::from(c) - i32::from(b'a' - b'A');
    }
    i32::from(c)
}

fn equal(s: Option<&str>, t: Option<&str>) -> bool {
    let test_for_equality = true;
    let result = percent_normalized_comparison(s, t, test_for_equality);
    result == 0
}

// US-ASCII only
fn equal_ignoring_case(s: Option<&str>, t: Option<&str>) -> bool {
    match (s, t) {
        (None, None) => true,
        (Some(s), Some(t)) => {
            let s: Vec<u16> = s.encode_utf16().collect();
            let t: Vec<u16> = t.encode_utf16().collect();
            s.len() == t.len() && s.iter().zip(&t).all(|(&c, &d)| to_lower(c) == to_lower(d))
        }
        _ => false,
    }
}

fn hash(hash: i32, s: Option<&str>) -> i32 {
    let Some(s) = s else {
        return hash;
    };
    if s.contains('%') {
        normalized_hash(hash, s)
    } else {
        hash.wrapping_mul(127).wrapping_add(s.hash_code())
    }
}

fn normalized_hash(hash: i32, s: &str) -> i32 {
    let s: Vec<u16> = s.encode_utf16().collect();
    let mut h = 0i32;
    let mut index = 0;
    while index < s.len() {
        let ch = s[index];
        h = h.wrapping_mul(31).wrapping_add(i32::from(ch));
        if ch == PERCENT {
            // Process the next two encoded characters
            for &c in &s[index + 1..index + 3] {
                h = h.wrapping_mul(31).wrapping_add(to_upper(c));
            }
            index += 2;
        }
        index += 1;
    }
    hash.wrapping_mul(127).wrapping_add(h)
}

// US-ASCII only
fn hash_ignoring_case(hash: i32, s: Option<&str>) -> i32 {
    let Some(s) = s else {
        return hash;
    };
    s.encode_utf16()
        .fold(hash, |h, c| h.wrapping_mul(31).wrapping_add(to_lower(c)))
}

fn compare(s: Option<&str>, t: Option<&str>) -> i32 {
    let test_for_equality = false;
    percent_normalized_comparison(s, t, test_for_equality)
}

// The percentNormalizedComparison method does not verify two characters that follow the % sign
// are hexadecimal digits. Reason being:
// 1) percentNormalizedComparison method is not called with 'decoded' strings
// 2) The only place where a percent can be followed by anything other than hexadecimal digits is
// in the authority component (for a IPv6 scope) and the whole authority component is case
// insensitive.
//
// Java returns 0 first for the same string object, which compares equal anyway.
fn percent_normalized_comparison(s: Option<&str>, t: Option<&str>, test_for_equality: bool) -> i32 {
    let (s, t) = match (s, t) {
        (None, None) => return 0,
        (Some(_), None) => return 1,
        (None, Some(_)) => return -1,
        (Some(s), Some(t)) => (s, t),
    };
    if !s.contains('%') {
        return s.compare_to(t);
    }
    let s: Vec<u16> = s.encode_utf16().collect();
    let t: Vec<u16> = t.encode_utf16().collect();
    let sn = s.len() as i32;
    let tn = t.len() as i32;
    if sn != tn && test_for_equality {
        return sn - tn;
    }
    let n = sn.min(tn) as usize;
    let mut i = 0;
    while i < n {
        let c = s[i];
        let d = t[i];
        let val = i32::from(c) - i32::from(d);
        if c != PERCENT {
            if val != 0 {
                return val;
            }
            i += 1;
            continue;
        }
        if d != PERCENT && val != 0 {
            return val;
        }
        i += 1;
        let val = to_lower(s[i]) - to_lower(t[i]);
        if val != 0 {
            return val;
        }
        i += 1;
        let val = to_lower(s[i]) - to_lower(t[i]);
        if val != 0 {
            return val;
        }
        i += 1;
    }
    sn - tn
}

// US-ASCII only
fn compare_ignoring_case(s: Option<&str>, t: Option<&str>) -> i32 {
    match (s, t) {
        (None, None) => 0,
        (Some(_), None) => 1,
        (None, Some(_)) => -1,
        (Some(s), Some(t)) => {
            let s: Vec<u16> = s.encode_utf16().collect();
            let t: Vec<u16> = t.encode_utf16().collect();
            for (&c, &d) in s.iter().zip(&t) {
                let c = to_lower(c) - to_lower(d);
                if c != 0 {
                    return c;
                }
            }
            s.len() as i32 - t.len() as i32
        }
    }
}

// -- Character classes for parsing --

// RFC2396 precisely specifies which characters in the US-ASCII charset are permissible in the
// various components of a URI reference. We here define a set of mask pairs to aid in enforcing
// these restrictions. Each mask pair consists of two longs, a low mask and a high mask. Taken
// together they represent a 128-bit mask, where bit i is set iff the character with value i is
// permitted.

// Tell whether the given character is permitted by the given mask pair
fn r#match(c: u16, low_mask: i64, high_mask: i64) -> bool {
    if c == 0 {
        // 0 doesn't have a slot in the mask. So, it never matches.
        return false;
    }
    if c < 64 {
        return ((1i64 << c) & low_mask) != 0;
    }
    if c < 128 {
        return ((1i64 << (c - 64)) & high_mask) != 0;
    }
    false
}

// Character-class masks, in reverse order from RFC2396 because initializers for static fields
// cannot make forward references.

// digit    = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
const L_DIGIT: i64 = 0x3FF000000000000; // lowMask('0', '9');
const H_DIGIT: i64 = 0;

// upalpha  = "A" | "B" | … | "Z"
const L_UPALPHA: i64 = 0;
const H_UPALPHA: i64 = 0x7FFFFFE; // highMask('A', 'Z');

// lowalpha = "a" | "b" | … | "z"
const L_LOWALPHA: i64 = 0;
const H_LOWALPHA: i64 = 0x7FFFFFE00000000; // highMask('a', 'z');

// alpha         = lowalpha | upalpha
const L_ALPHA: i64 = L_LOWALPHA | L_UPALPHA;
const H_ALPHA: i64 = H_LOWALPHA | H_UPALPHA;

// alphanum      = alpha | digit
const L_ALPHANUM: i64 = L_DIGIT | L_ALPHA;
const H_ALPHANUM: i64 = H_DIGIT | H_ALPHA;

// hex           = digit | "A" | "B" | "C" | "D" | "E" | "F" |
//                         "a" | "b" | "c" | "d" | "e" | "f"
const L_HEX: i64 = L_DIGIT;
const H_HEX: i64 = 0x7E0000007E; // highMask('A', 'F') | highMask('a', 'f');

// mark          = "-" | "_" | "." | "!" | "~" | "*" | "'" | "(" | ")"
const L_MARK: i64 = 0x678200000000; // lowMask("-_.!~*'()");
const H_MARK: i64 = 0x4000000080000000; // highMask("-_.!~*'()");

// unreserved    = alphanum | mark
const L_UNRESERVED: i64 = L_ALPHANUM | L_MARK;
const H_UNRESERVED: i64 = H_ALPHANUM | H_MARK;

// reserved      = ";" | "/" | "?" | ":" | "@" | "&" | "=" | "+" | "$" | "," | "[" | "]"
// Added per RFC2732: "[", "]"
const L_RESERVED: i64 = 0xAC00985000000000_u64 as i64; // lowMask(";/?:@&=+$,[]");
const H_RESERVED: i64 = 0x28000001; // highMask(";/?:@&=+$,[]");

// The zero'th bit is used to indicate that escape pairs and non-US-ASCII characters are allowed;
// this is handled by the scanEscape method below.
const L_ESCAPED: i64 = 1;
const H_ESCAPED: i64 = 0;

// uric          = reserved | unreserved | escaped
const L_URIC: i64 = L_RESERVED | L_UNRESERVED | L_ESCAPED;
const H_URIC: i64 = H_RESERVED | H_UNRESERVED | H_ESCAPED;

// pchar         = unreserved | escaped | ":" | "@" | "&" | "=" | "+" | "$" | ","
const L_PCHAR: i64 = L_UNRESERVED | L_ESCAPED | 0x2400185000000000; // lowMask(":@&=+$,");
const H_PCHAR: i64 = H_UNRESERVED | H_ESCAPED | 0x1; // highMask(":@&=+$,");

// All valid path characters
const L_PATH: i64 = L_PCHAR | 0x800800000000000; // lowMask(";/");
const H_PATH: i64 = H_PCHAR; // highMask(";/") == 0x0L;

// Dash, for use in domainlabel and toplabel
const L_DASH: i64 = 0x200000000000; // lowMask("-");
const H_DASH: i64 = 0x0; // highMask("-");

// Dot, for use in hostnames
const L_DOT: i64 = 0x400000000000; // lowMask(".");
const H_DOT: i64 = 0x0; // highMask(".");

// userinfo      = *( unreserved | escaped | ";" | ":" | "&" | "=" | "+" | "$" | "," )
const L_USERINFO: i64 = L_UNRESERVED | L_ESCAPED | 0x2C00185000000000; // lowMask(";:&=+$,");
const H_USERINFO: i64 = H_UNRESERVED | H_ESCAPED; // | highMask(";:&=+$,") == 0L;

// reg_name      = 1*( unreserved | escaped | "$" | "," | ";" | ":" | "@" | "&" | "=" | "+" )
const L_REG_NAME: i64 = L_UNRESERVED | L_ESCAPED | 0x2C00185000000000; // lowMask("$,;:@&=+");
const H_REG_NAME: i64 = H_UNRESERVED | H_ESCAPED | 0x1; // highMask("$,;:@&=+");

// All valid characters for server-based authorities
const L_SERVER: i64 = L_USERINFO | L_ALPHANUM | L_DASH | 0x400400000000000; // lowMask(".:@[]");
const H_SERVER: i64 = H_USERINFO | H_ALPHANUM | H_DASH | 0x28000001; // highMask(".:@[]");

// Special case of server authority that represents an IPv6 address
// In this case, a % does not signify an escape sequence
const L_SERVER_PERCENT: i64 = L_SERVER | 0x2000000000; // lowMask("%");
const H_SERVER_PERCENT: i64 = H_SERVER; // | highMask("%") == 0L;

// scheme        = alpha *( alpha | digit | "+" | "-" | "." )
const L_SCHEME: i64 = L_ALPHA | L_DIGIT | 0x680000000000; // lowMask("+-.");
const H_SCHEME: i64 = H_ALPHA | H_DIGIT; // | highMask("+-.") == 0L

// scope_id = alpha | digit | "_" | "."
const L_SCOPE_ID: i64 = L_ALPHANUM | 0x400000000000; // lowMask("_.");
const H_SCOPE_ID: i64 = H_ALPHANUM | 0x80000000; // highMask("_.");

// -- Escaping and encoding --

fn decode_char(c: u16) -> i32 {
    match u8::try_from(c) {
        Ok(c @ b'0'..=b'9') => i32::from(c - b'0'),
        Ok(c @ b'a'..=b'f') => i32::from(c - b'a' + 10),
        Ok(c @ b'A'..=b'F') => i32::from(c - b'A' + 10),
        _ => -1,
    }
}

fn decode_pair(c1: u16, c2: u16) -> i8 {
    (((decode_char(c1) & 0xf) << 4) | (decode_char(c2) & 0xf)) as i8
}

// Evaluates all escapes in s, applying UTF-8 decoding if needed. Assumes that escapes are
// well-formed syntactically, i.e., of the form %XX. If a sequence of escaped octets is not valid
// UTF-8 then the erroneous octets are replaced with '\uFFFD'.
// Exception: any "%" found between "[]" is left alone. It is an IPv6 literal with a scope_id
fn decode(s: &str) -> Cow<'_, str> {
    decode_ignoring(s, true)
}

// `decode(String, boolean)`, which decodes the octets with `UTF_8`'s decoder replacing what's
// malformed, which replaces it like `new String(bytes, UTF_8)` (`from_bytes_utf8`).
fn decode_ignoring(s: &str, ignore_percent_in_brackets: bool) -> Cow<'_, str> {
    if s.is_empty() || !s.contains('%') {
        return Cow::Borrowed(s);
    }

    let s: Vec<u16> = s.encode_utf16().collect();
    let n = s.len();
    let mut sb: Vec<u16> = Vec::with_capacity(n);
    let mut bb: Vec<i8> = Vec::with_capacity(n);

    // This is not horribly efficient, but it will do for now
    let mut c = s[0];
    let mut between_brackets = false;

    let mut i = 0;
    while i < n {
        if c == u16::from(b'[') {
            between_brackets = true;
        } else if between_brackets && c == u16::from(b']') {
            between_brackets = false;
        }
        if c != PERCENT || (between_brackets && ignore_percent_in_brackets) {
            sb.push(c);
            i += 1;
            if i >= n {
                break;
            }
            c = s[i];
            continue;
        }
        bb.clear();
        loop {
            bb.push(decode_pair(s[i + 1], s[i + 2]));
            i += 3;
            if i >= n {
                break;
            }
            c = s[i];
            if c != PERCENT {
                break;
            }
        }
        sb.extend(from_bytes_utf8(&bb).encode_utf16());
    }

    // What's copied is whole characters, since escapes are well-formed.
    Cow::Owned(String::from_utf16_lossy(&sb))
}

// -- Parsing --

// For convenience we wrap the input URI string in a new instance of the following internal
// class. This saves always having to pass the input string as an argument to each internal
// scan/parse method.
struct Parser<'a> {
    // Java's enclosing instance, whose components the parser sets.
    uri: &'a mut URI,
    // URI input string
    input: &'a str,
    // Its UTF-16 units, which the parser reads and its indices count.
    chars: Vec<u16>,
    require_server_authority: bool,
    ipv6byte_count: i32,
}

impl<'a> Parser<'a> {
    fn new(uri: &'a mut URI, s: &'a str) -> Parser<'a> {
        uri.string = s.to_owned();
        Parser {
            uri,
            input: s,
            chars: s.encode_utf16().collect(),
            require_server_authority: false,
            ipv6byte_count: 0,
        }
    }

    // -- Methods for throwing URISyntaxException in various ways --

    fn fail<T>(&self, reason: &str) -> Result<T, URISyntaxException> {
        Err(URISyntaxException::new(
            &Exceptions::format_msg(Exceptions::filter_non_socket_info(self.input)),
            reason,
        ))
    }

    fn fail_at<T>(&self, reason: &str, p: i32) -> Result<T, URISyntaxException> {
        let p = if Exceptions::enhanced_non_socket_exceptions() {
            p
        } else {
            -1
        };
        Err(URISyntaxException::with_index(
            &Exceptions::format_msg(Exceptions::filter_non_socket_info(self.input)),
            reason,
            p,
        ))
    }

    fn fail_expecting<T>(&self, expected: &str, p: i32) -> Result<T, URISyntaxException> {
        self.fail_at(&format!("Expected {expected}"), p)
    }

    // -- Simple access to the input string --

    fn length(&self) -> i32 {
        self.chars.len() as i32
    }

    fn char_at(&self, index: i32) -> u16 {
        self.chars[index as usize]
    }

    // The input from start to end, which never cuts a surrogate pair: the parser only cuts at
    // ASCII characters.
    fn substring(&self, start: i32, end: i32) -> String {
        String::from_utf16(&self.chars[start as usize..end as usize])
            .expect("components end at ASCII characters")
    }

    // Tells whether start < end and, if so, whether charAt(start) == c
    fn at(&self, start: i32, end: i32, c: u8) -> bool {
        start < end && self.char_at(start) == u16::from(c)
    }

    // Tells whether start + s.length() < end and, if so, whether the chars at the start position
    // match s exactly
    fn at_str(&self, start: i32, end: i32, s: &str) -> bool {
        let sn = s.len() as i32;
        if sn > end - start {
            return false;
        }
        (start..)
            .zip(s.bytes())
            .all(|(p, c)| self.char_at(p) == u16::from(c))
    }

    // -- Scanning --

    // The various scan and parse methods that follow use a uniform convention of taking the
    // current start position and end index as their first two arguments. The start is inclusive
    // while the end is exclusive, just as in the String class, i.e., a start/end pair denotes the
    // left-open interval [start, end) of the input string.
    //
    // These methods never proceed past the end position. They may return -1 to indicate outright
    // failure, but more often they simply return the position of the first char after the last
    // char scanned.

    // Scan a specific char: If the char at the given start position is equal to c, return the
    // index of the next char; otherwise, return the start position.
    fn scan_char(&self, start: i32, end: i32, c: u8) -> i32 {
        if start < end && self.char_at(start) == u16::from(c) {
            return start + 1;
        }
        start
    }

    // Scan forward from the given start position. Stop at the first char in the err string (in
    // which case -1 is returned), or the first char in the stop string (in which case the index
    // of the preceding char is returned), or the end of the input string (in which case the
    // length of the input string is returned). May return the start position if nothing matches.
    fn scan_err(&self, start: i32, end: i32, err: &str, stop: &str) -> i32 {
        let mut p = start;
        while p < end {
            let c = self.char_at(p);
            if err.bytes().any(|e| u16::from(e) == c) {
                return -1;
            }
            if stop.bytes().any(|s| u16::from(s) == c) {
                break;
            }
            p += 1;
        }
        p
    }

    // Scan forward from the given start position. Stop at the first char in the stop string (in
    // which case the index of the preceding char is returned), or the end of the input string (in
    // which case the length of the input string is returned). May return the start position if
    // nothing matches.
    fn scan(&self, start: i32, end: i32, stop: &str) -> i32 {
        let mut p = start;
        while p < end {
            let c = self.char_at(p);
            if stop.bytes().any(|s| u16::from(s) == c) {
                break;
            }
            p += 1;
        }
        p
    }

    // Scan a potential escape sequence, starting at the given position, with the given first
    // char (i.e., charAt(start) == c).
    //
    // This method assumes that if escapes are allowed then visible non-US-ASCII chars are also
    // allowed.
    fn scan_escape(&self, start: i32, n: i32, first: u16) -> Result<i32, URISyntaxException> {
        let p = start;
        let c = first;
        if c == PERCENT {
            // Process escape pair
            if p + 3 <= n
                && r#match(self.char_at(p + 1), L_HEX, H_HEX)
                && r#match(self.char_at(p + 2), L_HEX, H_HEX)
            {
                return Ok(p + 3);
            }
            return self.fail_at("Malformed escape pair", p);
        } else if c > 128 && !Character::is_space_char(c) && !Character::is_iso_control(c) {
            // Allow unescaped but visible non-US-ASCII chars
            return Ok(p + 1);
        }
        Ok(p)
    }

    // Scan chars that match the given mask pair
    fn scan_mask(
        &self,
        start: i32,
        n: i32,
        low_mask: i64,
        high_mask: i64,
    ) -> Result<i32, URISyntaxException> {
        let mut p = start;
        while p < n {
            let c = self.char_at(p);
            if r#match(c, low_mask, high_mask) {
                p += 1;
                continue;
            }
            if (low_mask & L_ESCAPED) != 0 {
                let q = self.scan_escape(p, n, c)?;
                if q > p {
                    p = q;
                    continue;
                }
            }
            break;
        }
        Ok(p)
    }

    // Check that each of the chars in [start, end) matches the given mask
    fn check_chars(
        &self,
        start: i32,
        end: i32,
        low_mask: i64,
        high_mask: i64,
        what: &str,
    ) -> Result<(), URISyntaxException> {
        let p = self.scan_mask(start, end, low_mask, high_mask)?;
        if p < end {
            return self.fail_at(&format!("Illegal character in {what}"), p);
        }
        Ok(())
    }

    // Check that the char at position p matches the given mask
    fn check_char(
        &self,
        p: i32,
        low_mask: i64,
        high_mask: i64,
        what: &str,
    ) -> Result<(), URISyntaxException> {
        self.check_chars(p, p + 1, low_mask, high_mask, what)
    }

    // -- Parsing --

    // [<scheme>:]<scheme-specific-part>[#<fragment>]
    fn parse(&mut self, rsa: bool) -> Result<(), URISyntaxException> {
        self.require_server_authority = rsa;
        let n = self.length();
        let mut p = self.scan_err(0, n, "/?#", ":");
        if p >= 0 && self.at(p, n, b':') {
            if p == 0 {
                return self.fail_expecting("scheme name", 0);
            }
            self.check_char(0, L_ALPHA, H_ALPHA, "scheme name")?;
            self.check_chars(1, p, L_SCHEME, H_SCHEME, "scheme name")?;
            self.uri.scheme = Some(self.substring(0, p));
            p += 1; // Skip ':'
            if self.at(p, n, b'/') {
                p = self.parse_hierarchical(p, n)?;
            } else {
                // opaque; need to create the schemeSpecificPart
                let q = self.scan(p, n, "#");
                if q <= p {
                    return self.fail_expecting("scheme-specific part", p);
                }
                self.check_chars(p, q, L_URIC, H_URIC, "opaque part")?;
                self.uri.scheme_specific_part = Some(self.substring(p, q));
                p = q;
            }
        } else {
            p = self.parse_hierarchical(0, n)?;
        }
        if self.at(p, n, b'#') {
            self.check_chars(p + 1, n, L_URIC, H_URIC, "fragment")?;
            self.uri.fragment = Some(self.substring(p + 1, n));
            p = n;
        }
        if p < n {
            return self.fail_at("end of URI", p);
        }
        Ok(())
    }

    // [//authority]<path>[?<query>]
    //
    // DEVIATION from RFC2396: We allow an empty authority component as long as it's followed by a
    // non-empty path, query component, or fragment component. This is so that URIs such as
    // "file:///foo/bar" will parse. This seems to be the intent of RFC2396, though the grammar
    // does not permit it. If the authority is empty then the userInfo, host, and port components
    // are undefined.
    //
    // DEVIATION from RFC2396: We allow empty relative paths. This seems to be the intent of
    // RFC2396, but the grammar does not permit it. The primary consequence of this deviation is
    // that "#f" parses as a relative URI with an empty path.
    fn parse_hierarchical(&mut self, start: i32, n: i32) -> Result<i32, URISyntaxException> {
        let mut p = start;
        if self.at(p, n, b'/') && self.at(p + 1, n, b'/') {
            p += 2;
            let q = self.scan(p, n, "/?#");
            if q > p {
                p = self.parse_authority(p, q)?;
            } else if q >= n {
                // An empty authority is only allowed before a non-empty path, a query or a
                // fragment (DEVIATION).
                return self.fail_expecting("authority", p);
            }
        }
        let mut q = self.scan(p, n, "?#"); // DEVIATION: May be empty
        self.check_chars(p, q, L_PATH, H_PATH, "path")?;
        self.uri.path = Some(self.substring(p, q));
        p = q;
        if self.at(p, n, b'?') {
            p += 1;
            q = self.scan(p, n, "#");
            self.check_chars(p, q, L_URIC, H_URIC, "query")?;
            self.uri.query = Some(self.substring(p, q));
            p = q;
        }
        Ok(p)
    }

    // authority     = server | reg_name
    //
    // Ambiguity: An authority that is a registry name rather than a server might have a prefix
    // that parses as a server. We use the fact that the authority component is always followed by
    // '/' or the end of the input string to resolve this: If the complete authority did not parse
    // as a server then we try to parse it as a registry name.
    fn parse_authority(&mut self, start: i32, n: i32) -> Result<i32, URISyntaxException> {
        let p = start;
        let mut q = p;
        let mut ex = None;

        let server_chars = if self.scan(p, n, "]") > p {
            // contains a literal IPv6 address, therefore % is allowed
            self.scan_mask(p, n, L_SERVER_PERCENT, H_SERVER_PERCENT)? == n
        } else {
            self.scan_mask(p, n, L_SERVER, H_SERVER)? == n
        };
        let qreg = self.scan_mask(p, n, L_REG_NAME, H_REG_NAME)?;
        let reg_chars = qreg == n;

        if reg_chars && !server_chars {
            // Must be a registry-based authority
            self.uri.authority = Some(self.substring(p, n));
            return Ok(n);
        }

        // When parsing a URI, skip creating exception objects if the server-based authority is
        // not required and the registry parse is successful.
        let skip_parse_exception = !self.require_server_authority && reg_chars;
        if server_chars {
            // Might be (probably is) a server-based authority, so attempt to parse it as such.
            // If the attempt fails, try to treat it as a registry-based authority. Java's try
            // block, which the failed expectation of its end is thrown in too.
            let attempt = match self.parse_server(p, n, skip_parse_exception) {
                Ok(end) if end < n && skip_parse_exception => {
                    self.uri.user_info = None;
                    self.uri.host = None;
                    self.uri.port = -1;
                    Ok(p)
                }
                Ok(end) if end < n => self.fail_expecting("end of authority", end),
                Ok(end) => {
                    self.uri.authority = Some(self.substring(p, n));
                    Ok(end)
                }
                Err(x) => Err(x),
            };
            match attempt {
                Ok(end) => q = end,
                Err(x) => {
                    // Undo results of failed parse
                    self.uri.user_info = None;
                    self.uri.host = None;
                    self.uri.port = -1;
                    if self.require_server_authority {
                        // If we're insisting upon a server-based authority, then just re-throw
                        // the exception
                        return Err(x);
                    }
                    // Save the exception in case it doesn't parse as a registry either
                    ex = Some(x);
                    q = p;
                }
            }
        }

        if q < n {
            if reg_chars {
                // Registry-based authority
                self.uri.authority = Some(self.substring(p, n));
            } else if let Some(ex) = ex {
                // Re-throw exception; it was probably due to a malformed IPv6 address
                return Err(ex);
            } else {
                return self.fail_at(
                    "Illegal character in authority",
                    if server_chars { q } else { qreg },
                );
            }
        }

        Ok(n)
    }

    // [<userinfo>@]<host>[:<port>]
    fn parse_server(
        &mut self,
        start: i32,
        n: i32,
        skip_parse_exception: bool,
    ) -> Result<i32, URISyntaxException> {
        let mut p = start;

        // userinfo
        let mut q = self.scan_err(p, n, "/?#", "@");
        if q >= p && self.at(q, n, b'@') {
            self.check_chars(p, q, L_USERINFO, H_USERINFO, "user info")?;
            self.uri.user_info = Some(self.substring(p, q));
            p = q + 1; // Skip '@'
        }

        // hostname, IPv4 address, or IPv6 address
        if self.at(p, n, b'[') {
            // DEVIATION from RFC2396: Support IPv6 addresses, per RFC2732
            p += 1;
            q = self.scan_err(p, n, "/?#", "]");
            if q > p && self.at(q, n, b']') {
                // look for a "%" scope id
                let r = self.scan(p, q, "%");
                if r > p {
                    self.parse_ipv6_reference(p, r)?;
                    if r + 1 == q {
                        return self.fail("scope id expected");
                    }
                    self.check_chars(r + 1, q, L_SCOPE_ID, H_SCOPE_ID, "scope id")?;
                } else {
                    self.parse_ipv6_reference(p, q)?;
                }
                self.uri.host = Some(self.substring(p - 1, q + 1));
                p = q + 1;
            } else {
                return self.fail_expecting("closing bracket for IPv6 address", q);
            }
        } else {
            q = self.parse_ipv4_address(p, n);
            if q <= p {
                q = self.parse_hostname(p, n, skip_parse_exception)?;
            }
            p = q;
        }

        // port
        if self.at(p, n, b':') {
            p += 1;
            q = self.scan(p, n, "/");
            if q > p {
                self.check_chars(p, q, L_DIGIT, H_DIGIT, "port number")?;
                match Integer::parse_int(&self.substring(p, q)) {
                    Ok(port) => self.uri.port = port,
                    Err(_) => return self.fail_at("Malformed port number", p),
                }
                p = q;
            }
        } else if p < n && skip_parse_exception {
            return Ok(p);
        }

        if p < n {
            return self.fail_expecting("port number", p);
        }

        Ok(p)
    }

    // Scan a string of decimal digits whose value fits in a byte
    fn scan_byte(&self, start: i32, n: i32) -> Result<i32, URISyntaxException> {
        let p = start;
        let q = self.scan_mask(p, n, L_DIGIT, H_DIGIT)?;
        if q <= p {
            return Ok(q);
        }

        // Handle leading zeros
        let mut i = p;
        loop {
            let j = self.scan_char(i, q, b'0');
            if j <= i {
                break;
            }
            i = j;
        }

        // Calculate the number of significant digits (after leading zeros)
        let significant_digits_num = q - i;

        if significant_digits_num < 3 {
            return Ok(q); // definitely < 255
        }

        // If more than 3 significant digits, it's definitely > 255
        if significant_digits_num > 3 {
            return Ok(p);
        }

        let value = Integer::parse_int(&self.substring(p, q))
            .expect("three significant digits fit in an int");
        if value > 255 {
            return Ok(p);
        }
        Ok(q)
    }

    // Scan an IPv4 address.
    //
    // If the strict argument is true then we require that the given interval contain nothing
    // besides an IPv4 address; if it is false then we only require that it start with an IPv4
    // address.
    //
    // If the interval does not contain or start with (depending upon the strict argument) a legal
    // IPv4 address characters then we return -1 immediately; otherwise we insist that these
    // characters parse as a legal IPv4 address and throw an exception on failure.
    //
    // We assume that any string of decimal digits and dots must be an IPv4 address. It won't
    // parse as a hostname anyway, so making that assumption here allows more meaningful
    // exceptions to be thrown.
    fn scan_ipv4_address(
        &self,
        start: i32,
        n: i32,
        strict: bool,
    ) -> Result<i32, URISyntaxException> {
        let mut p = start;
        let mut q;
        let m = self.scan_mask(p, n, L_DIGIT | L_DOT, H_DIGIT | H_DOT)?;
        if m <= p || (strict && m != n) {
            return Ok(-1);
        }
        // Java's `for (;;)`, which only runs once.
        'address: {
            // Per RFC2732: At most three digits per byte
            // Further constraint: Each element fits in a byte
            q = self.scan_byte(p, m)?;
            if q <= p {
                break 'address;
            }
            p = q;
            q = self.scan_char(p, m, b'.');
            if q <= p {
                break 'address;
            }
            p = q;
            q = self.scan_byte(p, m)?;
            if q <= p {
                break 'address;
            }
            p = q;
            q = self.scan_char(p, m, b'.');
            if q <= p {
                break 'address;
            }
            p = q;
            q = self.scan_byte(p, m)?;
            if q <= p {
                break 'address;
            }
            p = q;
            q = self.scan_char(p, m, b'.');
            if q <= p {
                break 'address;
            }
            p = q;
            q = self.scan_byte(p, m)?;
            if q <= p {
                break 'address;
            }
            if q < m {
                break 'address;
            }
            return Ok(q);
        }
        if strict {
            return self.fail_at("Malformed IPv4 address", q);
        }
        Ok(-1)
    }

    // Take an IPv4 address: Throw an exception if the given interval contains anything except an
    // IPv4 address
    fn take_ipv4_address(
        &self,
        start: i32,
        n: i32,
        expected: &str,
    ) -> Result<i32, URISyntaxException> {
        let p = self.scan_ipv4_address(start, n, true)?;
        if p <= start {
            return self.fail_expecting(expected, start);
        }
        Ok(p)
    }

    // Attempt to parse an IPv4 address, returning -1 on failure but allowing the given interval
    // to contain [:<characters>] after the IPv4 address.
    fn parse_ipv4_address(&mut self, start: i32, n: i32) -> i32 {
        let Ok(p) = self.scan_ipv4_address(start, n, false) else {
            return -1;
        };

        if p == -1 {
            return p;
        }

        if p > start && p < n {
            // IPv4 address is followed by something - check that it's a ":" as this is the only
            // valid character to follow an address.
            if self.char_at(p) != u16::from(b':') {
                return -1;
            }
        }

        if p > start {
            self.uri.host = Some(self.substring(start, p));
        }

        p
    }

    // hostname      = domainlabel [ "." ] | 1*( domainlabel "." ) toplabel [ "." ]
    // domainlabel   = alphanum | alphanum *( alphanum | "-" ) alphanum
    // toplabel      = alpha | alpha *( alphanum | "-" ) alphanum
    fn parse_hostname(
        &mut self,
        start: i32,
        n: i32,
        skip_parse_exception: bool,
    ) -> Result<i32, URISyntaxException> {
        let mut p = start;
        let mut q;
        let mut l = -1; // Start of last parsed label

        loop {
            // domainlabel = alphanum [ *( alphanum | "-" ) alphanum ]
            q = self.scan_mask(p, n, L_ALPHANUM, H_ALPHANUM)?;
            if q <= p {
                break;
            }
            l = p;
            p = q;
            q = self.scan_mask(p, n, L_ALPHANUM | L_DASH, H_ALPHANUM | H_DASH)?;
            if q > p {
                if self.char_at(q - 1) == u16::from(b'-') {
                    return self.fail_at("Illegal character in hostname", q - 1);
                }
                p = q;
            }
            q = self.scan_char(p, n, b'.');
            if q <= p {
                break;
            }
            p = q;
            if p >= n {
                break;
            }
        }

        if p < n && !self.at(p, n, b':') {
            if skip_parse_exception {
                return Ok(p);
            }
            return self.fail_at("Illegal character in hostname", p);
        }
        if l < 0 {
            return self.fail_expecting("hostname", start);
        }

        // for a fully qualified hostname check that the rightmost label starts with an alpha
        // character.
        if l > start && !r#match(self.char_at(l), L_ALPHA, H_ALPHA) {
            return self.fail_at("Illegal character in hostname", l);
        }

        self.uri.host = Some(self.substring(start, p));
        Ok(p)
    }

    // IPv6 address parsing, from RFC2373: IPv6 Addressing Architecture
    //
    // Bug: The grammar in RFC2373 Appendix B does not allow addresses of the form ::12.34.56.78,
    // which are clearly shown in the examples earlier in the document. Here is the original
    // grammar:
    //
    //   IPv6address = hexpart [ ":" IPv4address ]
    //   hexpart     = hexseq | hexseq "::" [ hexseq ] | "::" [ hexseq ]
    //   hexseq      = hex4 *( ":" hex4)
    //   hex4        = 1*4HEXDIG
    //
    // We therefore use the following revised grammar:
    //
    //   IPv6address = hexseq [ ":" IPv4address ]
    //                 | hexseq [ "::" [ hexpost ] ]
    //                 | "::" [ hexpost ]
    //   hexpost     = hexseq | hexseq ":" IPv4address | IPv4address
    //   hexseq      = hex4 *( ":" hex4)
    //   hex4        = 1*4HEXDIG
    //
    // This covers all and only the following cases:
    //
    //   hexseq
    //   hexseq : IPv4address
    //   hexseq ::
    //   hexseq :: hexseq
    //   hexseq :: hexseq : IPv4address
    //   hexseq :: IPv4address
    //   :: hexseq
    //   :: hexseq : IPv4address
    //   :: IPv4address
    //   ::
    //
    // Additionally we constrain the IPv6 address as follows :-
    //
    //  i.  IPv6 addresses without compressed zeros should contain exactly 16 bytes.
    //
    //  ii. IPv6 addresses with compressed zeros should contain less than 16 bytes.
    fn parse_ipv6_reference(&mut self, start: i32, n: i32) -> Result<i32, URISyntaxException> {
        let mut p = start;
        let mut compressed_zeros = false;

        let q = self.scan_hex_seq(p, n)?;

        if q > p {
            p = q;
            if self.at_str(p, n, "::") {
                compressed_zeros = true;
                p = self.scan_hex_post(p + 2, n)?;
            } else if self.at(p, n, b':') {
                p = self.take_ipv4_address(p + 1, n, "IPv4 address")?;
                self.ipv6byte_count += 4;
            }
        } else if self.at_str(p, n, "::") {
            compressed_zeros = true;
            p = self.scan_hex_post(p + 2, n)?;
        }
        if p < n {
            return self.fail_at("Malformed IPv6 address", start);
        }
        if self.ipv6byte_count > 16 {
            return self.fail_at("IPv6 address too long", start);
        }
        if !compressed_zeros && self.ipv6byte_count < 16 {
            return self.fail_at("IPv6 address too short", start);
        }
        if compressed_zeros && self.ipv6byte_count == 16 {
            return self.fail_at("Malformed IPv6 address", start);
        }

        Ok(p)
    }

    fn scan_hex_post(&mut self, start: i32, n: i32) -> Result<i32, URISyntaxException> {
        let mut p = start;

        if p == n {
            return Ok(p);
        }

        let q = self.scan_hex_seq(p, n)?;
        if q > p {
            p = q;
            if self.at(p, n, b':') {
                p += 1;
                p = self.take_ipv4_address(p, n, "hex digits or IPv4 address")?;
                self.ipv6byte_count += 4;
            }
        } else {
            p = self.take_ipv4_address(p, n, "hex digits or IPv4 address")?;
            self.ipv6byte_count += 4;
        }
        Ok(p)
    }

    // Scan a hex sequence; return -1 if one could not be scanned
    fn scan_hex_seq(&mut self, start: i32, n: i32) -> Result<i32, URISyntaxException> {
        let mut p = start;

        let mut q = self.scan_mask(p, n, L_HEX, H_HEX)?;
        if q <= p {
            return Ok(-1);
        }
        if self.at(q, n, b'.') {
            // Beginning of IPv4 address
            return Ok(-1);
        }
        if q > p + 4 {
            return self.fail_at("IPv6 hexadecimal digit sequence too long", p);
        }
        self.ipv6byte_count += 2;
        p = q;
        while p < n {
            if !self.at(p, n, b':') {
                break;
            }
            if self.at(p + 1, n, b':') {
                break; // "::"
            }
            p += 1;
            q = self.scan_mask(p, n, L_HEX, H_HEX)?;
            if q <= p {
                return self.fail_expecting("digits for an IPv6 address", p);
            }
            if self.at(q, n, b'.') {
                // Beginning of IPv4 address
                p -= 1;
                break;
            }
            if q > p + 4 {
                return self.fail_at("IPv6 hexadecimal digit sequence too long", p);
            }
            self.ipv6byte_count += 2;
            p = q;
        }

        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::URI;
    use crate::golden::{self, Golden};
    use crate::net::minecraft::util::util::Util;

    // [scheme, rawSchemeSpecificPart, rawAuthority, rawUserInfo, host, port, rawPath, rawQuery,
    // rawFragment, toString, isAbsolute, isOpaque, [schemeSpecificPart, authority, userInfo, path,
    // query, fragment]], the suite's order.
    fn components(uri: &URI) -> Value {
        json!([
            uri.get_scheme(),
            uri.get_raw_scheme_specific_part(),
            uri.get_raw_authority(),
            uri.get_raw_user_info(),
            uri.get_host(),
            uri.get_port(),
            uri.get_raw_path(),
            uri.get_raw_query(),
            uri.get_raw_fragment(),
            uri.to_string(),
            uri.is_absolute(),
            uri.is_opaque(),
            [
                uri.get_scheme_specific_part(),
                uri.get_authority(),
                uri.get_user_info(),
                uri.get_path(),
                uri.get_query(),
                uri.get_fragment(),
            ],
        ])
    }

    fn parse(s: &str) -> URI {
        URI::new(s).unwrap_or_else(|x| panic!("URISyntaxException: {x}"))
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.net.URI");
        golden.check("parse", |s: String| components(&parse(&s)));
        golden.check("parseError", |s: String| match URI::new(&s) {
            Ok(uri) => panic!("parsed as {uri:?}"),
            Err(x) => x.to_string(),
        });
        // `URI.create` makes the same URIs, and throws an `IllegalArgumentException` with the
        // message where the constructor throws.
        golden.check("parse", |s: String| components(&URI::create(&s)));
        golden.check("parseError", |s: String| {
            match golden::catch(|| URI::create(&s)) {
                Ok(uri) => panic!("created {uri:?}"),
                Err(message) => message,
            }
        });
        golden.check("Util.parseAndValidateUntrustedUri", |s: String| {
            match Util::parse_and_validate_untrusted_uri(&s) {
                Ok(uri) => ("success".to_owned(), uri.to_string()),
                Err(x) => ("error".to_owned(), x.to_string()),
            }
        });
        // Each char but the surrogates, which a Rust string can't hold alone, between the two
        // strings, the ones that parse as ranges.
        golden.check("acceptsCharacter", |prefix: String, suffix: String| {
            let mut ranges: Vec<(i32, i32)> = Vec::new();
            for c in (0..=0xFFFF).filter_map(char::from_u32) {
                if URI::new(&format!("{prefix}{c}{suffix}")).is_err() {
                    continue;
                }
                let c = c as i32;
                match ranges.last_mut() {
                    Some((_, last)) if *last + 1 == c => *last = c,
                    _ => ranges.push((c, c)),
                }
            }
            ranges
        });
        golden.check("hashCode()I", |s: String| parse(&s).hash_code());
        golden.check("equals(Ljava/lang/Object;)Z", |a: String, b: String| {
            parse(&a) == parse(&b)
        });
        golden.check("compareTo(Ljava/net/URI;)I", |a: String, b: String| {
            parse(&a).compare_to(&parse(&b))
        });
        golden.finish();
    }
}
