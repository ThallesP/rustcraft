//! net/minecraft/util/parsing/packrat/Atom.java
//!
//! The name of what a rule makes, and of a value a term stores in the scope. Java's is a record of
//! its name, but everything that looks one up (`Dictionary`'s `IdentityHashMap`, `Scope`, the
//! parse state's cache) compares references, so each `of` here makes an atom of its own, told
//! apart by an id that copies keep.

use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// A name for values of type `T`.
pub struct Atom<T> {
    name: &'static str,
    id: usize,
    value: PhantomData<fn() -> T>,
}

impl<T> Atom<T> {
    pub fn of(name: &'static str) -> Atom<T> {
        Atom {
            name,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            value: PhantomData,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Which atom this is: Java's reference.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<T> Clone for Atom<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Atom<T> {}

impl<T> fmt::Debug for Atom<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// `<name>`.
impl<T> fmt::Display for Atom<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.name)
    }
}
