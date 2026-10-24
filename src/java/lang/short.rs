//! java/lang/Short.java
//!
//! Only `parseShort(String)`, which `JsonPrimitive.getAsShort` parses a string with:
//! `Integer.parseInt`, then a range check. And `parseShort(String, int)`, which SNBT's numbers parse
//! with in radixes 2, 10 and 16.

use crate::java::lang::integer::Integer;
use crate::java::lang::number_format_exception::NumberFormatException;

pub struct Short;

impl Short {
    pub const MIN_VALUE: i16 = i16::MIN;
    pub const MAX_VALUE: i16 = i16::MAX;

    pub fn parse_short(s: &str) -> Result<i16, NumberFormatException> {
        Short::parse_short_radix(s, 10)
    }

    /// `parseShort(String, int)`: `Integer.parseInt` in the radix, then a range check.
    pub fn parse_short_radix(s: &str, radix: i32) -> Result<i16, NumberFormatException> {
        let i = Integer::parse_int_radix(s, radix)?;
        i16::try_from(i).map_err(|_| {
            NumberFormatException::new(format!("Value out of range. Value:\"{s}\" Radix:{radix}"))
        })
    }
}
