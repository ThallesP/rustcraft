//! net/minecraft/util/parsing/packrat/commands/StringReaderParserState.java
//!
//! The cached parse state over Brigadier's reader, whose cursor is the mark.

use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::util::parsing::packrat::cached_parse_state::{
    self, CachedParseState, CachedParseStateClass,
};
use crate::net::minecraft::util::parsing::packrat::control::Control;
use crate::net::minecraft::util::parsing::packrat::error_collector::ErrorCollector;
use crate::net::minecraft::util::parsing::packrat::named_rule::ErasedRule;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::scope::Scope;

pub struct StringReaderParserState<'a> {
    parent: CachedParseState<'a>,
    input: &'a mut StringReader,
}

impl<'a> StringReaderParserState<'a> {
    pub fn new(
        error_collector: &'a mut dyn ErrorCollector,
        input: &'a mut StringReader,
    ) -> StringReaderParserState<'a> {
        StringReaderParserState {
            parent: CachedParseState::new(error_collector),
            input,
        }
    }
}

impl<'a> Deref for StringReaderParserState<'a> {
    type Target = CachedParseState<'a>;

    fn deref(&self) -> &CachedParseState<'a> {
        &self.parent
    }
}

impl<'a> DerefMut for StringReaderParserState<'a> {
    fn deref_mut(&mut self) -> &mut CachedParseState<'a> {
        &mut self.parent
    }
}

impl<'a> CachedParseStateClass<'a, StringReader> for StringReaderParserState<'a> {
    fn parent(&mut self) -> &mut CachedParseState<'a> {
        &mut self.parent
    }
}

impl ParseState<StringReader> for StringReaderParserState<'_> {
    fn scope(&mut self) -> &mut Scope {
        self.parent.scope()
    }

    fn error_collector(&mut self) -> &mut dyn ErrorCollector {
        self.parent.error_collector()
    }

    fn parse_erased(&mut self, rule: &dyn ErasedRule<StringReader>) -> Option<Rc<dyn Any>> {
        cached_parse_state::parse(self, rule)
    }

    fn input(&mut self) -> &mut StringReader {
        self.input
    }

    fn mark(&self) -> i32 {
        self.input.get_cursor()
    }

    fn restore(&mut self, mark: i32) {
        self.input.set_cursor(mark);
    }

    fn acquire_control(&mut self) -> Control {
        self.parent.acquire_control()
    }

    fn release_control(&mut self) {
        self.parent.release_control();
    }

    fn silent(&mut self) -> Box<dyn ParseState<StringReader> + '_> {
        cached_parse_state::silent(self)
    }
}
