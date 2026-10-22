//! java/lang/Enum.java
//!
//! Only what generic code asks of any enum: `ordinal()`, and its class's constants
//! (`Class.getEnumConstants()`), which `FriendlyByteBuf.readEnum` and `writeEnum` use. Each enum
//! keeps its own `values()`, `valueOf` and name (PORTING.md).

/// `Enum<E>`, for an enum ported as a Rust enum without data, whose constants are declared in
/// Java's order, so that `as i32` is the ordinal.
pub trait Enum: Copy + 'static {
    /// `getDeclaringClass().getEnumConstants()`: every constant, in the order they're declared.
    const ENUM_CONSTANTS: &'static [Self];

    /// Its place among the constants, from 0.
    fn ordinal(&self) -> i32;
}
