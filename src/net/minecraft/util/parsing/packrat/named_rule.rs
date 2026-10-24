//! net/minecraft/util/parsing/packrat/NamedRule.java
//!
//! A rule with the name the parse state caches its results by. Java's one class of it is
//! `Dictionary.Entry`, which the dictionary binds a rule to, so this is a handle to an entry: the
//! dictionary owns its entries, and the terms and grammars naming one hold it weakly, so that a
//! grammar's rules, which name each other in cycles, go with its dictionary.

use std::any::Any;
use std::rc::Rc;
use std::sync::{Arc, Weak};

use crate::net::minecraft::util::parsing::packrat::atom::Atom;
use crate::net::minecraft::util::parsing::packrat::dictionary::Entry;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::rule::Rule;

/// A dictionary's entry for a name.
pub struct NamedRule<S, T> {
    name: Atom<T>,
    entry: Weak<Entry<S, T>>,
}

impl<S, T> NamedRule<S, T> {
    pub(super) fn new(entry: &Arc<Entry<S, T>>) -> NamedRule<S, T> {
        NamedRule {
            name: entry.name(),
            entry: Arc::downgrade(entry),
        }
    }

    pub fn name(&self) -> Atom<T> {
        self.name
    }

    /// The rule bound to the name; panics while there's none, where Java throws
    /// `NullPointerException`, and once the dictionary is gone.
    pub fn value(&self) -> Arc<dyn Rule<S, T>> {
        self.entry
            .upgrade()
            .unwrap_or_else(|| panic!("{}'s dictionary is gone", self.name))
            .value()
    }
}

impl<S, T> Clone for NamedRule<S, T> {
    fn clone(&self) -> Self {
        NamedRule {
            name: self.name,
            entry: self.entry.clone(),
        }
    }
}

/// A named rule with its value's type erased, which `ParseState::parse_erased` takes (Java's
/// parse state is generic in it, which a trait object's method can't be).
pub trait ErasedRule<S> {
    /// The id of the rule's name.
    fn name_id(&self) -> usize;

    /// The rule's result, boxed.
    fn parse_value(&self, state: &mut dyn ParseState<S>) -> Option<Rc<dyn Any>>;
}

impl<S, T: 'static> ErasedRule<S> for NamedRule<S, T> {
    fn name_id(&self) -> usize {
        self.name.id()
    }

    fn parse_value(&self, state: &mut dyn ParseState<S>) -> Option<Rc<dyn Any>> {
        self.value()
            .parse(state)
            .map(|value| Rc::new(value) as Rc<dyn Any>)
    }
}
