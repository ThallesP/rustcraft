//! com/google/gson/Strictness.java (Gson 2.14.0)
//!
//! How strictly a `JsonReader` reads and a `JsonWriter` writes: `LENIENT` takes Gson's extensions
//! (comments, unquoted and single-quoted names and values, NaN and infinities, several top-level
//! values…), `LEGACY_STRICT` (a reader's and writer's default) only a few (keywords in any case,
//! the `\'` and escaped-newline escapes, raw control characters in strings), and `STRICT` only RFC
//! 8259's JSON.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strictness {
    LENIENT,
    LEGACY_STRICT,
    STRICT,
}
