//! java/lang/Byte.java
//!
//! Only `parseByte(String)`, which `JsonPrimitive.getAsByte` parses a string with:
//! `Integer.parseInt`, then a range check.

use crate::java::lang::integer::Integer;
use crate::java::lang::number_format_exception::NumberFormatException;

pub struct Byte;

impl Byte {
    pub const MIN_VALUE: i8 = i8::MIN;
    pub const MAX_VALUE: i8 = i8::MAX;

    pub fn parse_byte(s: &str) -> Result<i8, NumberFormatException> {
        let i = Integer::parse_int(s)?;
        i8::try_from(i).map_err(|_| {
            NumberFormatException::new(format!("Value out of range. Value:\"{s}\" Radix:10"))
        })
    }
}
