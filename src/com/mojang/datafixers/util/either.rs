//! com/mojang/datafixers/util/Either.java (DataFixerUpper 10.0.21)
//!
//! One of two values. Java's abstract class with its `Left` and `Right` subclasses is an enum of
//! the two; the applicative machinery (`Either.Mu`, `Instance`) isn't ported.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn left_of(value: L) -> Either<L, R> {
        Either::Left(value)
    }

    pub fn right_of(value: R) -> Either<L, R> {
        Either::Right(value)
    }

    pub fn map_both<C, D>(self, f1: impl FnOnce(L) -> C, f2: impl FnOnce(R) -> D) -> Either<C, D> {
        match self {
            Either::Left(value) => Either::Left(f1(value)),
            Either::Right(value) => Either::Right(f2(value)),
        }
    }

    pub fn map<T>(self, l: impl FnOnce(L) -> T, r: impl FnOnce(R) -> T) -> T {
        match self {
            Either::Left(value) => l(value),
            Either::Right(value) => r(value),
        }
    }

    pub fn left(&self) -> Option<&L> {
        match self {
            Either::Left(value) => Some(value),
            Either::Right(_) => None,
        }
    }

    pub fn right(&self) -> Option<&R> {
        match self {
            Either::Left(_) => None,
            Either::Right(value) => Some(value),
        }
    }

    pub fn map_left<T>(self, l: impl FnOnce(L) -> T) -> Either<T, R> {
        self.map_both(l, |value| value)
    }

    pub fn map_right<T>(self, r: impl FnOnce(R) -> T) -> Either<L, T> {
        self.map_both(|value| value, r)
    }

    pub fn swap(self) -> Either<R, L> {
        match self {
            Either::Left(value) => Either::Right(value),
            Either::Right(value) => Either::Left(value),
        }
    }
}

impl<U> Either<U, U> {
    /// `unwrap`: whichever value it holds.
    pub fn unwrap(self) -> U {
        match self {
            Either::Left(value) | Either::Right(value) => value,
        }
    }
}

/// `Left[value]` and `Right[value]`.
impl<L: fmt::Display, R: fmt::Display> fmt::Display for Either<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Either::Left(value) => write!(f, "Left[{value}]"),
            Either::Right(value) => write!(f, "Right[{value}]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::Either;
    use crate::golden::{Decode, Encode, elements};

    /// `["left", value]` or `["right", value]`, as the codec suites record one.
    impl<L: Encode, R: Encode> Encode for Either<L, R> {
        fn encode(&self) -> Value {
            match self {
                Either::Left(value) => Value::from(vec![Value::from("left"), value.encode()]),
                Either::Right(value) => Value::from(vec![Value::from("right"), value.encode()]),
            }
        }

        fn matches(&self, expected: &Value) -> bool {
            match (self, expected.as_array().map(Vec::as_slice)) {
                (Either::Left(value), Some([side, expected])) => {
                    side == "left" && value.matches(expected)
                }
                (Either::Right(value), Some([side, expected])) => {
                    side == "right" && value.matches(expected)
                }
                _ => false,
            }
        }
    }

    impl<L: Decode, R: Decode> Decode for Either<L, R> {
        fn decode(value: &Value) -> Self {
            let [side, value] = elements(value);
            if side == "left" {
                Either::Left(L::decode(value))
            } else {
                Either::Right(R::decode(value))
            }
        }
    }
}
