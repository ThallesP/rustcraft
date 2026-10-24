//! net/minecraft/commands/BrigadierExceptions.java
//!
//! Brigadier's own errors, as the game words them: translations, which `CommandSyntaxException`
//! shows through the language (`en_us`). Java's instance is `CommandSyntaxException.BUILT_IN_EXCEPTIONS`
//! once `SharedConstants` sets it at startup; here its getters are associated functions, which
//! Brigadier's reader calls.

use std::sync::{Arc, LazyLock};

use crate::com::mojang::brigadier::exceptions::dynamic_2_command_exception_type::Dynamic2CommandExceptionType;
use crate::com::mojang::brigadier::exceptions::dynamic_command_exception_type::DynamicCommandExceptionType;
use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::com::mojang::brigadier::message::Message;
use crate::java::lang::number::Number;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::contents::translatable_contents::TranslationArgument;

// `Component.translatableEscape(key, args)` as a message.
fn translatable(key: &str, args: Vec<TranslationArgument>) -> Arc<dyn Message> {
    Arc::new(Component::from(Component::translatable_with_args(key, args)))
}

fn simple(key: &str) -> SimpleCommandExceptionType {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(key)))
}

fn string(value: String) -> TranslationArgument {
    TranslationArgument::String(value)
}

macro_rules! range {
    ($name:ident, $kind:ident, $type:ty, $key:literal) => {
        static $name: Dynamic2CommandExceptionType<$type, $type> =
            Dynamic2CommandExceptionType::new(|found, bound| {
                translatable(
                    $key,
                    vec![
                        TranslationArgument::Number(Number::$kind(bound)),
                        TranslationArgument::Number(Number::$kind(found)),
                    ],
                )
            });
    };
}

range!(DOUBLE_TOO_SMALL, Double, f64, "argument.double.low");
range!(DOUBLE_TOO_BIG, Double, f64, "argument.double.big");
range!(FLOAT_TOO_SMALL, Float, f32, "argument.float.low");
range!(FLOAT_TOO_BIG, Float, f32, "argument.float.big");
range!(INTEGER_TOO_SMALL, Integer, i32, "argument.integer.low");
range!(INTEGER_TOO_BIG, Integer, i32, "argument.integer.big");
range!(LONG_TOO_SMALL, Long, i64, "argument.long.low");
range!(LONG_TOO_BIG, Long, i64, "argument.long.big");

static LITERAL_INCORRECT: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|expected| {
        translatable("argument.literal.incorrect", vec![string(expected)])
    });
static READER_EXPECTED_START_OF_QUOTE: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("parsing.quote.expected.start"));
static READER_EXPECTED_END_OF_QUOTE: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("parsing.quote.expected.end"));
static READER_INVALID_ESCAPE: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|character| {
        translatable("parsing.quote.escape", vec![string(character)])
    });
static READER_INVALID_BOOL: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|value| translatable("parsing.bool.invalid", vec![string(value)]));
static READER_INVALID_INT: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|value| translatable("parsing.int.invalid", vec![string(value)]));
static READER_EXPECTED_INT: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("parsing.int.expected"));
static READER_INVALID_LONG: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|value| translatable("parsing.long.invalid", vec![string(value)]));
static READER_EXPECTED_LONG: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("parsing.long.expected"));
static READER_INVALID_DOUBLE: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|value| {
        translatable("parsing.double.invalid", vec![string(value)])
    });
static READER_EXPECTED_DOUBLE: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("parsing.double.expected"));
static READER_INVALID_FLOAT: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|value| {
        translatable("parsing.float.invalid", vec![string(value)])
    });
static READER_EXPECTED_FLOAT: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("parsing.float.expected"));
static READER_EXPECTED_BOOL: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("parsing.bool.expected"));
static READER_EXPECTED_SYMBOL: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|symbol| translatable("parsing.expected", vec![string(symbol)]));
static DISPATCHER_UNKNOWN_COMMAND: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("command.unknown.command"));
static DISPATCHER_UNKNOWN_ARGUMENT: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("command.unknown.argument"));
static DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("command.expected.separator"));
static DISPATCHER_PARSE_EXCEPTION: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|message| {
        translatable("command.exception", vec![string(message)])
    });

/// The game's wording of Brigadier's errors.
pub struct BrigadierExceptions;

impl BrigadierExceptions {
    pub fn double_too_low() -> &'static Dynamic2CommandExceptionType<f64, f64> {
        &DOUBLE_TOO_SMALL
    }

    pub fn double_too_high() -> &'static Dynamic2CommandExceptionType<f64, f64> {
        &DOUBLE_TOO_BIG
    }

    pub fn float_too_low() -> &'static Dynamic2CommandExceptionType<f32, f32> {
        &FLOAT_TOO_SMALL
    }

    pub fn float_too_high() -> &'static Dynamic2CommandExceptionType<f32, f32> {
        &FLOAT_TOO_BIG
    }

    pub fn integer_too_low() -> &'static Dynamic2CommandExceptionType<i32, i32> {
        &INTEGER_TOO_SMALL
    }

    pub fn integer_too_high() -> &'static Dynamic2CommandExceptionType<i32, i32> {
        &INTEGER_TOO_BIG
    }

    pub fn long_too_low() -> &'static Dynamic2CommandExceptionType<i64, i64> {
        &LONG_TOO_SMALL
    }

    pub fn long_too_high() -> &'static Dynamic2CommandExceptionType<i64, i64> {
        &LONG_TOO_BIG
    }

    pub fn literal_incorrect() -> &'static DynamicCommandExceptionType<String> {
        &LITERAL_INCORRECT
    }

    pub fn reader_expected_start_of_quote() -> &'static SimpleCommandExceptionType {
        &READER_EXPECTED_START_OF_QUOTE
    }

    pub fn reader_expected_end_of_quote() -> &'static SimpleCommandExceptionType {
        &READER_EXPECTED_END_OF_QUOTE
    }

    pub fn reader_invalid_escape() -> &'static DynamicCommandExceptionType<String> {
        &READER_INVALID_ESCAPE
    }

    pub fn reader_invalid_bool() -> &'static DynamicCommandExceptionType<String> {
        &READER_INVALID_BOOL
    }

    pub fn reader_invalid_int() -> &'static DynamicCommandExceptionType<String> {
        &READER_INVALID_INT
    }

    pub fn reader_expected_int() -> &'static SimpleCommandExceptionType {
        &READER_EXPECTED_INT
    }

    pub fn reader_invalid_long() -> &'static DynamicCommandExceptionType<String> {
        &READER_INVALID_LONG
    }

    pub fn reader_expected_long() -> &'static SimpleCommandExceptionType {
        &READER_EXPECTED_LONG
    }

    pub fn reader_invalid_double() -> &'static DynamicCommandExceptionType<String> {
        &READER_INVALID_DOUBLE
    }

    pub fn reader_expected_double() -> &'static SimpleCommandExceptionType {
        &READER_EXPECTED_DOUBLE
    }

    pub fn reader_invalid_float() -> &'static DynamicCommandExceptionType<String> {
        &READER_INVALID_FLOAT
    }

    pub fn reader_expected_float() -> &'static SimpleCommandExceptionType {
        &READER_EXPECTED_FLOAT
    }

    pub fn reader_expected_bool() -> &'static SimpleCommandExceptionType {
        &READER_EXPECTED_BOOL
    }

    pub fn reader_expected_symbol() -> &'static DynamicCommandExceptionType<String> {
        &READER_EXPECTED_SYMBOL
    }

    pub fn dispatcher_unknown_command() -> &'static SimpleCommandExceptionType {
        &DISPATCHER_UNKNOWN_COMMAND
    }

    pub fn dispatcher_unknown_argument() -> &'static SimpleCommandExceptionType {
        &DISPATCHER_UNKNOWN_ARGUMENT
    }

    pub fn dispatcher_expected_argument_separator() -> &'static SimpleCommandExceptionType {
        &DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR
    }

    pub fn dispatcher_parse_exception() -> &'static DynamicCommandExceptionType<String> {
        &DISPATCHER_PARSE_EXCEPTION
    }
}
