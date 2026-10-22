//! java.lang.Object's `hashCode`, for generic code that calls it on any value, like Guava's
//! collections. A type only its own callers hash has a `hash_code` method of its own instead.

use crate::java::lang::string::JavaString;

/// `Object.hashCode()`.
pub trait HashCode {
    fn hash_code(&self) -> i32;
}

impl HashCode for String {
    fn hash_code(&self) -> i32 {
        JavaString::hash_code(self.as_str())
    }
}

impl HashCode for str {
    fn hash_code(&self) -> i32 {
        JavaString::hash_code(self)
    }
}
