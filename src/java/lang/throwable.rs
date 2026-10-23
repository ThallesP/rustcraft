//! java/lang/Throwable.java
//!
//! An exception as a value, for ported code that returns what Java throws where callers catch
//! it: Gson's reader, parser and writer, whose `JsonSyntaxException` `ByteBufCodecs.lenientJson`
//! turns into a `DecoderException`. It holds the class's binary name, the message and the
//! exception it wraps. An exception Java makes from a cause alone (`new JsonSyntaxException(e)`)
//! has the cause's `toString()` as its message, which `with_cause` makes too. Code that throws
//! where nothing catches (a getter given the wrong kind of value) panics instead, with this
//! `toString()` as the message.

use std::fmt;

/// `java.io.EOFException`, which a reader throws at the end of its input.
pub const EOF_EXCEPTION: &str = "java.io.EOFException";
/// `java.io.IOException`.
pub const IO_EXCEPTION: &str = "java.io.IOException";
/// `java.lang.IllegalArgumentException`.
pub const ILLEGAL_ARGUMENT_EXCEPTION: &str = "java.lang.IllegalArgumentException";
/// `java.lang.IllegalStateException`.
pub const ILLEGAL_STATE_EXCEPTION: &str = "java.lang.IllegalStateException";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Throwable {
    class_name: &'static str,
    message: Option<String>,
    cause: Option<Box<Throwable>>,
}

impl Throwable {
    /// `new X(message)`.
    pub fn new(class_name: &'static str, message: impl Into<String>) -> Throwable {
        Throwable {
            class_name,
            message: Some(message.into()),
            cause: None,
        }
    }

    /// `new X()`: no message.
    pub fn without_message(class_name: &'static str) -> Throwable {
        Throwable {
            class_name,
            message: None,
            cause: None,
        }
    }

    /// `new X(cause)`: the cause's `toString()` is the message.
    pub fn with_cause(class_name: &'static str, cause: Throwable) -> Throwable {
        Throwable {
            class_name,
            message: Some(cause.to_string()),
            cause: Some(Box::new(cause)),
        }
    }

    /// `getClass().getName()`.
    pub fn class_name(&self) -> &'static str {
        self.class_name
    }

    pub fn get_message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn get_cause(&self) -> Option<&Throwable> {
        self.cause.as_deref()
    }
}

/// `toString()`: the class's name, then the message after a colon when there is one.
impl fmt::Display for Throwable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(message) => write!(f, "{}: {message}", self.class_name),
            None => f.write_str(self.class_name),
        }
    }
}
