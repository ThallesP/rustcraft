//! java/lang/Short.java
//!
//! Only `parseShort(String)`, which `JsonPrimitive.getAsShort` parses a string with:
//! `Integer.parseInt`, then a range check.

use crate::java::lang::integer::Integer;
use crate::java::lang::number_format_exception::NumberFormatException;

pub struct Short;

impl Short {
    pub const MIN_VALUE: i16 = i16::MIN;
    pub const MAX_VALUE: i16 = i16::MAX;

    pub fn parse_short(s: &str) -> Result<i16, NumberFormatException> {
        let i = Integer::parse_int(s)?;
        i16::try_from(i).map_err(|_| {
            NumberFormatException::new(format!("Value out of range. Value:\"{s}\" Radix:10"))
        })
    }
}
