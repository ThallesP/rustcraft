//! com/mojang/serialization/Lifecycle.java (DataFixerUpper 10.0.21)
//!
//! Java's stable and experimental lifecycles are two instances it compares by reference, and a
//! deprecated one an instance of a subclass with the version it was deprecated in, which doesn't
//! override `equals`. Here they're an enum's variants, compared by value: two deprecated
//! lifecycles of the same version are equal, where Java's are two objects, which only `setLifecycle`
//! tells apart (it keeps the result it has when the lifecycle is equal), with the same outcome. A
//! deprecated lifecycle's `toString` is `Object`'s, an identity hash, so it isn't ported.

use std::fmt;

/// How settled a value read or written by a codec is: stable, experimental, or deprecated since a
/// version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Stable,
    Experimental,
    Deprecated(i32),
}

impl Lifecycle {
    pub fn experimental() -> Lifecycle {
        Lifecycle::Experimental
    }

    pub fn stable() -> Lifecycle {
        Lifecycle::Stable
    }

    pub fn deprecated(since: i32) -> Lifecycle {
        Lifecycle::Deprecated(since)
    }

    /// The less settled of the two: experimental over everything, then the one deprecated
    /// earliest, then stable. Java's name, which isn't `Add`'s addition.
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, other: Lifecycle) -> Lifecycle {
        match (self, other) {
            (Lifecycle::Experimental, _) | (_, Lifecycle::Experimental) => Lifecycle::Experimental,
            (Lifecycle::Deprecated(since), Lifecycle::Deprecated(other_since)) => {
                if other_since < since { other } else { self }
            }
            (Lifecycle::Deprecated(_), _) => self,
            (_, Lifecycle::Deprecated(_)) => other,
            _ => Lifecycle::Stable,
        }
    }

    /// `Deprecated.since()`, for a deprecated lifecycle.
    pub fn since(self) -> Option<i32> {
        match self {
            Lifecycle::Deprecated(since) => Some(since),
            _ => None,
        }
    }
}

/// `Stable` and `Experimental`; a deprecated lifecycle prints Java's identity hash, which isn't
/// ported, so it prints its version.
impl fmt::Display for Lifecycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lifecycle::Stable => f.write_str("Stable"),
            Lifecycle::Experimental => f.write_str("Experimental"),
            Lifecycle::Deprecated(since) => write!(f, "Deprecated[{since}]"),
        }
    }
}
