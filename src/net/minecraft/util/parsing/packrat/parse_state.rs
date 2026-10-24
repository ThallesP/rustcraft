//! net/minecraft/util/parsing/packrat/ParseState.java
//!
//! What rules and terms parse with: the input, the scope, the errors, the cut controls, and the
//! cache of rules' results. Java's `parse(NamedRule)` is generic in the rule's value, which a
//! trait object's method can't be, so the trait's is `parse_erased`, of a rule with its value
//! boxed, and `parse` (on `dyn ParseState`) unboxes it. `silent()` hands out a view of the state
//! whose errors go nowhere, borrowing it, where Java's is a field of the state.

use std::any::Any;
use std::rc::Rc;

use crate::net::minecraft::util::parsing::packrat::control::Control;
use crate::net::minecraft::util::parsing::packrat::error_collector::ErrorCollector;
use crate::net::minecraft::util::parsing::packrat::named_rule::{ErasedRule, NamedRule};
use crate::net::minecraft::util::parsing::packrat::scope::Scope;

pub trait ParseState<S> {
    fn scope(&mut self) -> &mut Scope;

    fn error_collector(&mut self) -> &mut dyn ErrorCollector;

    /// `parse(NamedRule)`, the value boxed.
    fn parse_erased(&mut self, rule: &dyn ErasedRule<S>) -> Option<Rc<dyn Any>>;

    fn input(&mut self) -> &mut S;

    fn mark(&self) -> i32;

    fn restore(&mut self, mark: i32);

    fn acquire_control(&mut self) -> Control;

    fn release_control(&mut self);

    fn silent(&mut self) -> Box<dyn ParseState<S> + '_>;
}

impl<S> dyn ParseState<S> + '_ {
    /// `parse(NamedRule)`: the rule's value here, `None` where Java's is `null`.
    pub fn parse<T: Clone + 'static>(&mut self, rule: &NamedRule<S, T>) -> Option<T> {
        self.parse_erased(rule).map(|value| {
            value
                .downcast_ref::<T>()
                .unwrap_or_else(|| panic!("java.lang.ClassCastException: {}'s value", rule.name()))
                .clone()
        })
    }

    /// `parseTopRule`: the rule, and the end of the input it parsed for the errors; panics when
    /// a frame's left in the scope, where Java throws `IllegalStateException`.
    pub fn parse_top_rule<T: Clone + 'static>(&mut self, rule: &NamedRule<S, T>) -> Option<T> {
        let result = self.parse(rule);
        if result.is_some() {
            let mark = self.mark();
            self.error_collector().finish(mark);
        }
        if !self.scope().has_only_single_frame() {
            panic!(
                "java.lang.IllegalStateException: Malformed scope: {}",
                self.scope()
            );
        }
        result
    }
}
