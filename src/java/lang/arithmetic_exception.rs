//! java/lang/ArithmeticException.java
//!
//! What `BigDecimal.longValueExact` throws for a value with a fraction or past a long's range,
//! which `JsonOps.convertTo` catches: the `Err` of its port, with Java's message.

use std::fmt;

use crate::java::lang::throwable::Throwable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArithmeticException {
    message: String,
}

impl ArithmeticException {
    pub const CLASS_NAME: &str = "java.lang.ArithmeticException";

    pub fn new(message: impl Into<String>) -> ArithmeticException {
        ArithmeticException {
            message: message.into(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

/// `toString()`: the class's name and the message.
impl fmt::Display for ArithmeticException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", ArithmeticException::CLASS_NAME, self.message)
    }
}

impl From<ArithmeticException> for Throwable {
    fn from(exception: ArithmeticException) -> Throwable {
        Throwable::new(ArithmeticException::CLASS_NAME, exception.message)
    }
}
