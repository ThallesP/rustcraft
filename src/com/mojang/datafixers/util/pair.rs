//! com/mojang/datafixers/util/Pair.java (DataFixerUpper 10.0.21)
//!
//! Two values, which a decoder hands back as what it read and the rest of its input. The
//! applicative machinery (`Pair.Mu`, `Instance`) isn't ported.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pair<F, S> {
    first: F,
    second: S,
}

impl<F, S> Pair<F, S> {
    pub fn new(first: F, second: S) -> Pair<F, S> {
        Pair { first, second }
    }

    pub fn of(first: F, second: S) -> Pair<F, S> {
        Pair::new(first, second)
    }

    pub fn get_first(&self) -> &F {
        &self.first
    }

    pub fn get_second(&self) -> &S {
        &self.second
    }

    /// `getFirst()`, taking it.
    pub fn into_first(self) -> F {
        self.first
    }

    /// `getSecond()`, taking it.
    pub fn into_second(self) -> S {
        self.second
    }

    /// Both values, taken.
    pub fn into_parts(self) -> (F, S) {
        (self.first, self.second)
    }

    pub fn swap(self) -> Pair<S, F> {
        Pair::new(self.second, self.first)
    }

    pub fn map_first<F2>(self, function: impl FnOnce(F) -> F2) -> Pair<F2, S> {
        Pair::new(function(self.first), self.second)
    }

    pub fn map_second<S2>(self, function: impl FnOnce(S) -> S2) -> Pair<F, S2> {
        Pair::new(self.first, function(self.second))
    }
}

/// `(first, second)`.
impl<F: fmt::Display, S: fmt::Display> fmt::Display for Pair<F, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.first, self.second)
    }
}
