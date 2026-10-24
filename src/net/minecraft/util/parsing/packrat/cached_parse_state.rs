//! net/minecraft/util/parsing/packrat/CachedParseState.java
//!
//! The packrat part: each rule's result at each position is cached the first time it's parsed
//! there (or that it failed there), and a rule asked for again at the position hands it back and
//! moves the input past it without parsing again, so it stores no errors again either. The
//! abstract class is a struct a subclass holds (`StringReaderParserState`), whose `ParseState`
//! forwards to it, and its methods that need the subclass as Java's `this` (`parse`, which runs
//! the rule on the whole state, and `silent`) are this module's functions of it.
//!
//! The controls an alternative acquires are a pool of flags reused in stack order, like Java's
//! `SimpleControl`s.

use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

use crate::net::minecraft::util::parsing::packrat::control::Control;
use crate::net::minecraft::util::parsing::packrat::error_collector::{ErrorCollector, Nop};
use crate::net::minecraft::util::parsing::packrat::named_rule::ErasedRule;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::scope::Scope;

/// A rule's result at a position: its value and where it ended, or `None` for a failure
/// (`CacheEntry.NEGATIVE`).
struct CacheEntry {
    value: Option<(Rc<dyn Any>, i32)>,
}

/// The rules parsed at one position, in the order they were first asked for, each with its
/// result once it has one.
#[derive(Default)]
struct PositionCache {
    atom_cache: Vec<(usize, Option<CacheEntry>)>,
}

impl PositionCache {
    fn find_key_index(&self, key: usize) -> Option<usize> {
        self.atom_cache.iter().position(|(atom, _)| *atom == key)
    }

    fn allocate_new_entry(&mut self, key: usize) -> usize {
        self.atom_cache.push((key, None));
        self.atom_cache.len() - 1
    }
}

/// What every parse state of a cached kind holds.
pub struct CachedParseState<'a> {
    position_cache: Vec<Option<PositionCache>>,
    error_collector: &'a mut dyn ErrorCollector,
    scope: Scope,
    control_cache: Vec<Rc<Cell<bool>>>,
    next_control_to_return: usize,
}

impl<'a> CachedParseState<'a> {
    pub fn new(error_collector: &'a mut dyn ErrorCollector) -> CachedParseState<'a> {
        CachedParseState {
            position_cache: Vec::new(),
            error_collector,
            scope: Scope::new(),
            control_cache: Vec::new(),
            next_control_to_return: 0,
        }
    }

    pub fn scope(&mut self) -> &mut Scope {
        &mut self.scope
    }

    pub fn error_collector(&mut self) -> &mut dyn ErrorCollector {
        &mut *self.error_collector
    }

    fn get_cache_for_position(&mut self, index: i32) -> &mut PositionCache {
        let index = usize::try_from(index).expect("a negative mark");
        if index >= self.position_cache.len() {
            self.position_cache.resize_with(index + 1, || None);
        }
        self.position_cache[index].get_or_insert_with(PositionCache::default)
    }

    pub fn acquire_control(&mut self) -> Control {
        let control_index = self.next_control_to_return;
        self.next_control_to_return += 1;
        match self.control_cache.get(control_index) {
            Some(entry) => {
                entry.set(false);
                Control::simple(entry.clone())
            }
            None => {
                let entry = Rc::new(Cell::new(false));
                self.control_cache.push(entry.clone());
                Control::simple(entry)
            }
        }
    }

    pub fn release_control(&mut self) {
        self.next_control_to_return -= 1;
    }
}

/// A parse state holding a `CachedParseState`, which its `ParseState` methods forward to.
pub trait CachedParseStateClass<'a, S>: ParseState<S> {
    fn parent(&mut self) -> &mut CachedParseState<'a>;
}

/// `parse(NamedRule)`: the cached result at the mark, else the rule's, cached.
pub fn parse<'a, S>(
    this: &mut impl CachedParseStateClass<'a, S>,
    rule: &dyn ErasedRule<S>,
) -> Option<Rc<dyn Any>> {
    let mark_before_parse = this.mark();
    let position_cache = this.parent().get_cache_for_position(mark_before_parse);
    let entry_index = match position_cache.find_key_index(rule.name_id()) {
        Some(entry_index) => {
            if let Some(value) = &position_cache.atom_cache[entry_index].1 {
                let Some((value, mark_after_parse)) = &value.value else {
                    return None;
                };
                let (value, mark_after_parse) = (value.clone(), *mark_after_parse);
                this.restore(mark_after_parse);
                return Some(value);
            }
            entry_index
        }
        None => position_cache.allocate_new_entry(rule.name_id()),
    };
    let result = rule.parse_value(this);
    let entry = CacheEntry {
        value: result.clone().map(|value| (value, this.mark())),
    };
    this.parent()
        .get_cache_for_position(mark_before_parse)
        .atom_cache[entry_index]
        .1 = Some(entry);
    result
}

/// `silent()`: a view of the state whose errors go nowhere.
pub fn silent<'s, S: 's>(this: &'s mut dyn ParseState<S>) -> Box<dyn ParseState<S> + 's> {
    Box::new(Silent {
        outer: this,
        silent_collector: Nop,
    })
}

/// The state with a `Nop` collector. Its `parse` is the state's own, which parses the rule with
/// the state, errors and all, like Java's.
struct Silent<'s, S> {
    outer: &'s mut dyn ParseState<S>,
    silent_collector: Nop,
}

impl<S> ParseState<S> for Silent<'_, S> {
    fn scope(&mut self) -> &mut Scope {
        self.outer.scope()
    }

    fn error_collector(&mut self) -> &mut dyn ErrorCollector {
        &mut self.silent_collector
    }

    fn parse_erased(&mut self, rule: &dyn ErasedRule<S>) -> Option<Rc<dyn Any>> {
        self.outer.parse_erased(rule)
    }

    fn input(&mut self) -> &mut S {
        self.outer.input()
    }

    fn mark(&self) -> i32 {
        self.outer.mark()
    }

    fn restore(&mut self, mark: i32) {
        self.outer.restore(mark);
    }

    fn acquire_control(&mut self) -> Control {
        self.outer.acquire_control()
    }

    fn release_control(&mut self) {
        self.outer.release_control();
    }

    fn silent(&mut self) -> Box<dyn ParseState<S> + '_> {
        silent(self)
    }
}
