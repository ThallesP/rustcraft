//! java/lang/NumberFormatException.java
//!
//! What the JDK's number parsers (`Integer.parseInt`, `Double.parseDouble`, `new BigDecimal`)
//! throw, returned as the `Err` of their ports, with Java's message.

use std::fmt;

use crate::java::lang::throwable::Throwable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberFormatException {
    message: Option<String>,
}

impl NumberFormatException {
    pub const CLASS_NAME: &str = "java.lang.NumberFormatException";

    pub fn new(message: impl Into<String>) -> NumberFormatException {
        NumberFormatException {
            message: Some(message.into()),
        }
    }

    /// `new NumberFormatException()`, which `new BigDecimal` throws for input it reads past.
    pub fn without_message() -> NumberFormatException {
        NumberFormatException { message: None }
    }

    /// `forInputString(s, 10)`.
    pub fn for_input_string(s: &str) -> NumberFormatException {
        NumberFormatException::new(format!("For input string: \"{s}\""))
    }

    /// `forInputString(String, int)`: the radix named past 10.
    pub fn for_input_string_radix(s: &str, radix: i32) -> NumberFormatException {
        if radix == 10 {
            NumberFormatException::for_input_string(s)
        } else {
            NumberFormatException::new(format!("For input string: \"{s}\" under radix {radix}"))
        }
    }

    /// `forCharSequence`: where in the range the bad character is.
    pub fn for_char_sequence(
        s: &[u16],
        begin_index: usize,
        end_index: usize,
        error_index: usize,
    ) -> NumberFormatException {
        NumberFormatException::new(format!(
            "Error at index {} in: \"{}\"",
            error_index - begin_index,
            String::from_utf16_lossy(&s[begin_index..end_index])
        ))
    }

    pub fn get_message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

/// `toString()`: the class's name, then the message after a colon when there is one.
impl fmt::Display for NumberFormatException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(message) => write!(f, "{}: {message}", NumberFormatException::CLASS_NAME),
            None => f.write_str(NumberFormatException::CLASS_NAME),
        }
    }
}

impl From<NumberFormatException> for Throwable {
    fn from(exception: NumberFormatException) -> Throwable {
        match exception.message {
            Some(message) => Throwable::new(NumberFormatException::CLASS_NAME, message),
            None => Throwable::without_message(NumberFormatException::CLASS_NAME),
        }
    }
}
