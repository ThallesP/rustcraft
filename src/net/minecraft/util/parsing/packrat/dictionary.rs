//! net/minecraft/util/parsing/packrat/Dictionary.java
//!
//! A grammar's rules by name. A name can be referred to (`named`, `forward`) before its rule is
//! put, which is how rules refer to each other in cycles; `checkAllBound` makes sure each got one.
//! The dictionary owns its entries, and what refers to one holds it weakly (`NamedRule`). Its map
//! is Java's `IdentityHashMap` of the atoms, here by their ids, in the order the atoms were made,
//! which only `checkAllBound`'s message shows. It's behind a lock, so that terms naming rules
//! can be made while a rule is put (`rules.put(name, term::optional(rules.named(sign)), …)`, as
//! Java writes it), and the grammar holding it can be a static.

use std::any::Any;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, OnceLock};

use crate::net::minecraft::util::parsing::packrat::atom::Atom;
use crate::net::minecraft::util::parsing::packrat::control::Control;
use crate::net::minecraft::util::parsing::packrat::named_rule::NamedRule;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::rule::{self, Rule};
use crate::net::minecraft::util::parsing::packrat::scope::Scope;
use crate::net::minecraft::util::parsing::packrat::term::Term;

/// A name's slot for its rule.
pub struct Entry<S, T> {
    name: Atom<T>,
    value: OnceLock<Arc<dyn Rule<S, T>>>,
}

impl<S, T> Entry<S, T> {
    pub fn name(&self) -> Atom<T> {
        self.name
    }

    /// Panics while there's no rule, where Java throws `NullPointerException`.
    pub fn value(&self) -> Arc<dyn Rule<S, T>> {
        self.value
            .get()
            .unwrap_or_else(|| panic!("java.lang.NullPointerException: Unbound rule {}", self.name))
            .clone()
    }
}

// An entry with its types erased, as the map holds it.
trait AnyEntry: Send + Sync {
    fn is_bound(&self) -> bool;

    fn atom_name(&self) -> String;

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl<S: 'static, T: 'static> AnyEntry for Entry<S, T> {
    fn is_bound(&self) -> bool {
        self.value.get().is_some()
    }

    fn atom_name(&self) -> String {
        self.name.to_string()
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

/// The rules of a grammar over input `S`.
pub struct Dictionary<S> {
    terms: Mutex<BTreeMap<usize, Arc<dyn AnyEntry>>>,
    input: PhantomData<fn(S)>,
}

impl<S: 'static> Dictionary<S> {
    pub fn new() -> Dictionary<S> {
        Dictionary {
            terms: Mutex::new(BTreeMap::new()),
            input: PhantomData,
        }
    }

    /// `put(Atom, Rule)`: binds the rule to the name; panics if it has one, where Java throws
    /// `IllegalArgumentException`.
    pub fn put_rule<T: 'static>(
        &self,
        name: Atom<T>,
        entry: impl Rule<S, T> + 'static,
    ) -> NamedRule<S, T> {
        let holder = self.get_or_create_entry(name);
        if holder.value.set(Arc::new(entry)).is_err() {
            panic!("java.lang.IllegalArgumentException: Trying to override rule: {name}");
        }
        NamedRule::new(&holder)
    }

    /// `putComplex`: the term, then an action of the whole state.
    pub fn put_complex<T: 'static>(
        &self,
        name: Atom<T>,
        term: Box<dyn Term<S>>,
        action: impl Fn(&mut dyn ParseState<S>) -> Option<T> + Send + Sync + 'static,
    ) -> NamedRule<S, T> {
        self.put_rule(name, rule::from_term(term, action))
    }

    /// `put(Atom, Term, SimpleRuleAction)`: the term, then an action of the scope.
    pub fn put<T: 'static>(
        &self,
        name: Atom<T>,
        term: Box<dyn Term<S>>,
        action: impl Fn(&Scope) -> Option<T> + Send + Sync + 'static,
    ) -> NamedRule<S, T> {
        self.put_rule(name, rule::from_term_simple(term, action))
    }

    /// Panics, naming them, if any name has no rule, where Java throws `IllegalStateException`.
    pub fn check_all_bound(&self) {
        let unbound_names: Vec<String> = self
            .terms()
            .values()
            .filter(|entry| !entry.is_bound())
            .map(|entry| entry.atom_name())
            .collect();
        if !unbound_names.is_empty() {
            panic!(
                "java.lang.IllegalStateException: Unbound names: [{}]",
                unbound_names.join(", ")
            );
        }
    }

    /// Panics if the name was never referred to, where Java throws `NullPointerException`.
    pub fn get_or_throw<T: 'static>(&self, name: Atom<T>) -> NamedRule<S, T> {
        let terms = self.terms();
        let entry = terms
            .get(&name.id())
            .unwrap_or_else(|| panic!("java.lang.NullPointerException: No rule called {name}"));
        NamedRule::new(&Self::downcast(entry))
    }

    /// The name's rule, which may be put later.
    pub fn forward<T: 'static>(&self, name: Atom<T>) -> NamedRule<S, T> {
        NamedRule::new(&self.get_or_create_entry(name))
    }

    fn terms(&self) -> std::sync::MutexGuard<'_, BTreeMap<usize, Arc<dyn AnyEntry>>> {
        self.terms
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get_or_create_entry<T: 'static>(&self, name: Atom<T>) -> Arc<Entry<S, T>> {
        let mut terms = self.terms();
        let entry = terms.entry(name.id()).or_insert_with(|| {
            Arc::new(Entry::<S, T> {
                name,
                value: OnceLock::new(),
            })
        });
        Self::downcast(entry)
    }

    fn downcast<T: 'static>(entry: &Arc<dyn AnyEntry>) -> Arc<Entry<S, T>> {
        entry
            .clone()
            .into_any()
            .downcast::<Entry<S, T>>()
            .unwrap_or_else(|_| panic!("java.lang.ClassCastException: another type's atom"))
    }

    /// A term parsing the name's rule, storing its value under the name.
    pub fn named<T: Clone + 'static>(&self, name: Atom<T>) -> Box<dyn Term<S>> {
        Box::new(Reference {
            rule_to_parse: NamedRule::new(&self.get_or_create_entry(name)),
            name_to_store: name,
        })
    }

    /// A term parsing the first name's rule, storing its value under the second.
    pub fn named_with_alias<T: Clone + 'static>(
        &self,
        name_to_parse: Atom<T>,
        name_to_store: Atom<T>,
    ) -> Box<dyn Term<S>> {
        Box::new(Reference {
            rule_to_parse: NamedRule::new(&self.get_or_create_entry(name_to_parse)),
            name_to_store,
        })
    }
}

impl<S: 'static> Default for Dictionary<S> {
    fn default() -> Self {
        Dictionary::new()
    }
}

/// Parses a rule and stores its value.
struct Reference<S, T> {
    rule_to_parse: NamedRule<S, T>,
    name_to_store: Atom<T>,
}

impl<S, T: Clone + 'static> Term<S> for Reference<S, T> {
    fn parse(&self, state: &mut dyn ParseState<S>, _control: &Control) -> bool {
        let Some(result) = state.parse(&self.rule_to_parse) else {
            return false;
        };
        state.scope().put(&self.name_to_store, result);
        true
    }
}
