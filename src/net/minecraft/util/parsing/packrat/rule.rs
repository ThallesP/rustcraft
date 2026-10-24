//! net/minecraft/util/parsing/packrat/Rule.java
//!
//! What parses a value, `None` where Java's returns `null`. `fromTerm` wraps a term in a frame of
//! its own and runs the action on what it parsed; a `SimpleRuleAction` only reads the scope.

use crate::net::minecraft::util::parsing::packrat::control::Control;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::scope::Scope;
use crate::net::minecraft::util::parsing::packrat::term::Term;

pub trait Rule<S, T>: Send + Sync {
    fn parse(&self, state: &mut dyn ParseState<S>) -> Option<T>;
}

/// A rule held in a static (`SnbtGrammar`'s `NumberRunParseRule`s), which grammars share.
impl<S, T, R: Rule<S, T> + ?Sized> Rule<S, T> for &'static R {
    fn parse(&self, state: &mut dyn ParseState<S>) -> Option<T> {
        (**self).parse(state)
    }
}

/// `RuleAction`: the value from the state once the term has parsed.
pub type RuleAction<S, T> = dyn Fn(&mut dyn ParseState<S>) -> Option<T> + Send + Sync;

/// `fromTerm(Term, RuleAction)`.
pub fn from_term<S, T>(
    child: Box<dyn Term<S>>,
    action: impl Fn(&mut dyn ParseState<S>) -> Option<T> + Send + Sync + 'static,
) -> WrappedTerm<S, T> {
    WrappedTerm {
        action: Box::new(action),
        child,
    }
}

/// `fromTerm(Term, SimpleRuleAction)`: an action that only reads the scope.
pub fn from_term_simple<S, T>(
    child: Box<dyn Term<S>>,
    action: impl Fn(&Scope) -> Option<T> + Send + Sync + 'static,
) -> WrappedTerm<S, T> {
    from_term(child, move |state: &mut dyn ParseState<S>| {
        action(state.scope())
    })
}

/// A term in a frame of its own, and the action making its value.
pub struct WrappedTerm<S, T> {
    action: Box<RuleAction<S, T>>,
    child: Box<dyn Term<S>>,
}

impl<S, T> Rule<S, T> for WrappedTerm<S, T> {
    fn parse(&self, state: &mut dyn ParseState<S>) -> Option<T> {
        state.scope().push_frame();
        let result = if self.child.parse(state, &Control::UNBOUND) {
            (self.action)(state)
        } else {
            None
        };
        state.scope().pop_frame();
        result
    }
}
