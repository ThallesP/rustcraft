//! com/google/gson/internal/NumberLimits.java (Gson 2.14.0)
//!
//! The limits Gson puts on numbers it parses into a `BigDecimal`, so a huge string or exponent
//! can't make it work for minutes: 10,000 characters (UTF-16 units) and a scale under 10,000 either
//! way. `parseBigInteger` isn't ported: nothing reaches `BigInteger`.

use crate::java::lang::number_format_exception::NumberFormatException;
use crate::java::math::big_decimal::BigDecimal;

pub struct NumberLimits;

const MAX_NUMBER_STRING_LENGTH: usize = 10_000;

impl NumberLimits {
    fn check_number_string_length(s: &str) -> Result<(), NumberFormatException> {
        let units: Vec<u16> = s.encode_utf16().collect();
        if units.len() > MAX_NUMBER_STRING_LENGTH {
            return Err(NumberFormatException::new(format!(
                "Number string too large: {}...",
                String::from_utf16_lossy(&units[..30])
            )));
        }
        Ok(())
    }

    pub fn parse_big_decimal(s: &str) -> Result<BigDecimal, NumberFormatException> {
        NumberLimits::check_number_string_length(s)?;
        let decimal = BigDecimal::new(s)?;

        // Java casts to long to avoid issues with abs when the scale is Integer.MIN_VALUE.
        if i64::from(decimal.scale()).abs() >= 10_000 {
            return Err(NumberFormatException::new(format!(
                "Number has unsupported scale: {s}"
            )));
        }
        Ok(decimal)
    }
}
