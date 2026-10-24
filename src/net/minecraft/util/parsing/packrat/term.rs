//! net/minecraft/util/parsing/packrat/Term.java
//!
//! A piece of a rule: it parses or not, storing what it parsed in the scope. Java passes the
//! state's scope next to the state; here a term reaches it through the state (`state.scope()`),
//! which is the scope Java passes, a lookahead's silent state's included. The interface's static
//! methods making terms are this module's functions, and its records the structs they make.

use crate::net::minecraft::util::parsing::packrat::atom::Atom;
use crate::net::minecraft::util::parsing::packrat::control::Control;
use crate::net::minecraft::util::parsing::packrat::delayed_exception::DelayedException;
use crate::net::minecraft::util::parsing::packrat::named_rule::NamedRule;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;

pub trait Term<S>: Send + Sync {
    fn parse(&self, state: &mut dyn ParseState<S>, control: &Control) -> bool;
}

/// A term held in a static (`SnbtGrammar`'s number lookahead), which grammars share.
impl<S, T: Term<S> + ?Sized> Term<S> for &'static T {
    fn parse(&self, state: &mut dyn ParseState<S>, control: &Control) -> bool {
        (**self).parse(state, control)
    }
}

pub fn marker<S: 'static, T: Clone + Send + Sync + 'static>(
    name: Atom<T>,
    value: T,
) -> Box<dyn Term<S>> {
    Box::new(Marker { name, value })
}

pub fn sequence<S: 'static>(terms: Vec<Box<dyn Term<S>>>) -> Box<dyn Term<S>> {
    Box::new(Sequence { elements: terms })
}

pub fn alternative<S: 'static>(terms: Vec<Box<dyn Term<S>>>) -> Box<dyn Term<S>> {
    Box::new(Alternative { elements: terms })
}

pub fn optional<S: 'static>(term: Box<dyn Term<S>>) -> Box<dyn Term<S>> {
    Box::new(Maybe { term })
}

pub fn repeated<S: 'static, T: Clone + 'static>(
    element: NamedRule<S, T>,
    list_name: Atom<Vec<T>>,
) -> Box<dyn Term<S>> {
    repeated_min(element, list_name, 0)
}

/// `repeated(NamedRule, Atom, int)`.
pub fn repeated_min<S: 'static, T: Clone + 'static>(
    element: NamedRule<S, T>,
    list_name: Atom<Vec<T>>,
    min_repetitions: usize,
) -> Box<dyn Term<S>> {
    Box::new(Repeated {
        element,
        list_name,
        min_repetitions,
    })
}

pub fn repeated_with_trailing_separator<S: 'static, T: Clone + 'static>(
    element: NamedRule<S, T>,
    list_name: Atom<Vec<T>>,
    separator: Box<dyn Term<S>>,
) -> Box<dyn Term<S>> {
    repeated_with_trailing_separator_min(element, list_name, separator, 0)
}

/// `repeatedWithTrailingSeparator(NamedRule, Atom, Term, int)`.
pub fn repeated_with_trailing_separator_min<S: 'static, T: Clone + 'static>(
    element: NamedRule<S, T>,
    list_name: Atom<Vec<T>>,
    separator: Box<dyn Term<S>>,
    min_repetitions: usize,
) -> Box<dyn Term<S>> {
    Box::new(RepeatedWithSeparator {
        element,
        list_name,
        separator,
        min_repetitions,
        allow_trailing_separator: true,
    })
}

pub fn repeated_without_trailing_separator<S: 'static, T: Clone + 'static>(
    element: NamedRule<S, T>,
    list_name: Atom<Vec<T>>,
    separator: Box<dyn Term<S>>,
) -> Box<dyn Term<S>> {
    repeated_without_trailing_separator_min(element, list_name, separator, 0)
}

/// `repeatedWithoutTrailingSeparator(NamedRule, Atom, Term, int)`.
pub fn repeated_without_trailing_separator_min<S: 'static, T: Clone + 'static>(
    element: NamedRule<S, T>,
    list_name: Atom<Vec<T>>,
    separator: Box<dyn Term<S>>,
    min_repetitions: usize,
) -> Box<dyn Term<S>> {
    Box::new(RepeatedWithSeparator {
        element,
        list_name,
        separator,
        min_repetitions,
        allow_trailing_separator: false,
    })
}

pub fn positive_lookahead<S: 'static>(term: Box<dyn Term<S>>) -> Box<dyn Term<S>> {
    Box::new(LookAhead {
        term,
        positive: true,
    })
}

pub fn negative_lookahead<S: 'static>(term: Box<dyn Term<S>>) -> Box<dyn Term<S>> {
    Box::new(LookAhead {
        term,
        positive: false,
    })
}

/// `↑`: the alternative it's in tries no more alternatives.
pub fn cut<S: 'static>() -> Box<dyn Term<S>> {
    struct Cut;
    impl<S> Term<S> for Cut {
        fn parse(&self, _state: &mut dyn ParseState<S>, control: &Control) -> bool {
            control.cut();
            true
        }
    }
    Box::new(Cut)
}

/// `ε`.
pub fn empty<S: 'static>() -> Box<dyn Term<S>> {
    struct Empty;
    impl<S> Term<S> for Empty {
        fn parse(&self, _state: &mut dyn ParseState<S>, _control: &Control) -> bool {
            true
        }
    }
    Box::new(Empty)
}

/// Stores the message at the cursor and fails.
pub fn fail<S: 'static>(message: DelayedException) -> Box<dyn Term<S>> {
    struct Fail(DelayedException);
    impl<S> Term<S> for Fail {
        fn parse(&self, state: &mut dyn ParseState<S>, _control: &Control) -> bool {
            let mark = state.mark();
            state.error_collector().store(mark, &self.0);
            false
        }
    }
    Box::new(Fail(message))
}

/// The first of its terms that parses, unless one before cut.
pub struct Alternative<S> {
    elements: Vec<Box<dyn Term<S>>>,
}

impl<S> Term<S> for Alternative<S> {
    fn parse(&self, state: &mut dyn ParseState<S>, _control: &Control) -> bool {
        let control_for_this = state.acquire_control();
        let mark = state.mark();
        state.scope().split_frame();
        for element in &self.elements {
            if element.parse(state, &control_for_this) {
                state.scope().merge_frame();
                state.release_control();
                return true;
            }
            state.scope().clear_frame_values();
            state.restore(mark);
            if control_for_this.has_cut() {
                break;
            }
        }
        state.scope().pop_frame();
        state.release_control();
        false
    }
}

/// Whether the term parses here (or doesn't), without its errors or its input.
pub struct LookAhead<S> {
    term: Box<dyn Term<S>>,
    positive: bool,
}

impl<S> Term<S> for LookAhead<S> {
    fn parse(&self, state: &mut dyn ParseState<S>, control: &Control) -> bool {
        let mark = state.mark();
        let result = self.term.parse(&mut *state.silent(), control);
        state.restore(mark);
        self.positive == result
    }
}

/// Stores the value.
pub struct Marker<T> {
    name: Atom<T>,
    value: T,
}

impl<S, T: Clone + Send + Sync + 'static> Term<S> for Marker<T> {
    fn parse(&self, state: &mut dyn ParseState<S>, _control: &Control) -> bool {
        state.scope().put(&self.name, self.value.clone());
        true
    }
}

/// `optional`: the term, or nothing.
pub struct Maybe<S> {
    term: Box<dyn Term<S>>,
}

impl<S> Term<S> for Maybe<S> {
    fn parse(&self, state: &mut dyn ParseState<S>, control: &Control) -> bool {
        let mark = state.mark();
        if !self.term.parse(state, control) {
            state.restore(mark);
        }
        true
    }
}

/// The rule as many times as it parses, at least the minimum, as a list.
pub struct Repeated<S, T> {
    element: NamedRule<S, T>,
    list_name: Atom<Vec<T>>,
    min_repetitions: usize,
}

impl<S, T: Clone + 'static> Term<S> for Repeated<S, T> {
    fn parse(&self, state: &mut dyn ParseState<S>, _control: &Control) -> bool {
        let mark = state.mark();
        let mut elements = Vec::with_capacity(self.min_repetitions);
        loop {
            let entry_mark = state.mark();
            let Some(parsed_element) = state.parse(&self.element) else {
                state.restore(entry_mark);
                if elements.len() < self.min_repetitions {
                    state.restore(mark);
                    return false;
                }
                state.scope().put(&self.list_name, elements);
                return true;
            };
            elements.push(parsed_element);
        }
    }
}

/// The rule as many times as it parses with the separator between, at least the minimum, as a
/// list: a separator after the last is left alone if it may trail, else it fails the term.
pub struct RepeatedWithSeparator<S, T> {
    element: NamedRule<S, T>,
    list_name: Atom<Vec<T>>,
    separator: Box<dyn Term<S>>,
    min_repetitions: usize,
    allow_trailing_separator: bool,
}

impl<S, T: Clone + 'static> Term<S> for RepeatedWithSeparator<S, T> {
    fn parse(&self, state: &mut dyn ParseState<S>, control: &Control) -> bool {
        let list_mark = state.mark();
        let mut elements = Vec::with_capacity(self.min_repetitions);
        let mut first = true;
        loop {
            let mark_before_separator = state.mark();
            if !first && !self.separator.parse(state, control) {
                state.restore(mark_before_separator);
                break;
            }
            let mark_after_separator = state.mark();
            let Some(parsed_element) = state.parse(&self.element) else {
                if first {
                    state.restore(mark_after_separator);
                } else {
                    if !self.allow_trailing_separator {
                        state.restore(list_mark);
                        return false;
                    }
                    state.restore(mark_after_separator);
                }
                break;
            };
            elements.push(parsed_element);
            first = false;
        }
        if elements.len() < self.min_repetitions {
            state.restore(list_mark);
            return false;
        }
        state.scope().put(&self.list_name, elements);
        true
    }
}

/// Its terms one after another.
pub struct Sequence<S> {
    elements: Vec<Box<dyn Term<S>>>,
}

impl<S> Term<S> for Sequence<S> {
    fn parse(&self, state: &mut dyn ParseState<S>, control: &Control) -> bool {
        let mark = state.mark();
        for element in &self.elements {
            if !element.parse(state, control) {
                state.restore(mark);
                return false;
            }
        }
        true
    }
}
