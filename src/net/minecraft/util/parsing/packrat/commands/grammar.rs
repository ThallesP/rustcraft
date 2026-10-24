//! net/minecraft/util/parsing/packrat/commands/Grammar.java
//!
//! A dictionary of rules and the rule to parse, over Brigadier's reader. A failed parse throws the
//! first error stored at the furthest cursor any was (`ErrorCollector.LongestOnly`), made from the
//! reader's string; the reader's cursor is wherever the parse left it.
//!
//! Not ported: `parseForSuggestions` (see `CommandArgumentParser`).

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::util::parsing::packrat::commands::command_argument_parser::CommandArgumentParser;
use crate::net::minecraft::util::parsing::packrat::commands::string_reader_parser_state::StringReaderParserState;
use crate::net::minecraft::util::parsing::packrat::dictionary::Dictionary;
use crate::net::minecraft::util::parsing::packrat::error_collector::LongestOnly;
use crate::net::minecraft::util::parsing::packrat::named_rule::NamedRule;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;

pub struct Grammar<T> {
    rules: Dictionary<StringReader>,
    top: NamedRule<StringReader, T>,
}

impl<T: Clone + 'static> Grammar<T> {
    /// Panics if a rule the dictionary names isn't put, where Java throws
    /// `IllegalStateException`.
    pub fn new(rules: Dictionary<StringReader>, top: NamedRule<StringReader, T>) -> Grammar<T> {
        rules.check_all_bound();
        Grammar { rules, top }
    }

    pub fn rules(&self) -> &Dictionary<StringReader> {
        &self.rules
    }

    pub fn top(&self) -> &NamedRule<StringReader, T> {
        &self.top
    }

    pub fn parse(&self, state: &mut dyn ParseState<StringReader>) -> Option<T> {
        state.parse_top_rule(&self.top)
    }
}

impl<T: Clone + 'static> CommandArgumentParser<T> for Grammar<T> {
    fn parse_for_commands(&self, reader: &mut StringReader) -> Result<T, CommandSyntaxException> {
        let mut error_collector = LongestOnly::new();
        let result = {
            let mut state = StringReaderParserState::new(&mut error_collector, reader);
            self.parse(&mut state)
        };
        if let Some(result) = result {
            return Ok(result);
        }
        // Java makes every entry's exception, then throws the first `CommandSyntaxException`,
        // which every reason ported makes: the first entry's.
        let error_entries = error_collector.entries();
        match error_entries.first() {
            Some(entry) => Err(entry.reason().create(reader.get_string(), entry.cursor())),
            None => panic!("java.lang.IllegalStateException: Failed to parse: "),
        }
    }
}
