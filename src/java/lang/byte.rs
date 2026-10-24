//! java/lang/Byte.java
//!
//! Only `parseByte(String)`, which `JsonPrimitive.getAsByte` parses a string with:
//! `Integer.parseInt`, then a range check. And `parseByte(String, int)`, which SNBT's numbers parse
//! with in radixes 2, 10 and 16.

use crate::java::lang::integer::Integer;
use crate::java::lang::number_format_exception::NumberFormatException;

pub struct Byte;

impl Byte {
    pub const MIN_VALUE: i8 = i8::MIN;
    pub const MAX_VALUE: i8 = i8::MAX;

    pub fn parse_byte(s: &str) -> Result<i8, NumberFormatException> {
        Byte::parse_byte_radix(s, 10)
    }

    /// `parseByte(String, int)`: `Integer.parseInt` in the radix, then a range check.
    pub fn parse_byte_radix(s: &str, radix: i32) -> Result<i8, NumberFormatException> {
        let i = Integer::parse_int_radix(s, radix)?;
        i8::try_from(i).map_err(|_| {
            NumberFormatException::new(format!("Value out of range. Value:\"{s}\" Radix:{radix}"))
        })
    }
}
