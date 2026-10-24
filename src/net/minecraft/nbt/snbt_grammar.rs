//! net/minecraft/nbt/SnbtGrammar.java
//!
//! SNBT's grammar, over the packrat parser (`util::parsing::packrat`): `createParser` makes it for
//! any ops (`TagParser`'s is `NbtOps`'), and `escapeControlCharacters` is what `StringTag`'s
//! quoting writes SNBT with. What it reads, as vanilla does:
//!
//! - Numbers: an optional sign; integers in decimal (no leading zero), hex (`0x`) or binary
//!   (`0b`), with a type suffix (`b`, `s`, `i`, `l`) and a signedness prefix to it (`u`, `s`),
//!   hex and binary being unsigned unless it says `s`; floats with a whole part, a fraction, an
//!   exponent and `f` or `d`, finite only. Underscores may come between digits. Whitespace may
//!   come between any of the parts (every terminal and number run skips it first): `1 . 5 f` is
//!   `1.5f`.
//! - Strings: quoted with either quote, with escapes (`\b \s \t \n \f \r \\ \' \"`, `\x` and two
//!   hex digits, `\u` and four, `\U` and eight, `\N{name}`), or unquoted (Brigadier's characters,
//!   not starting like a number), `true` and `false` in any case being booleans, and a name with
//!   arguments in parentheses an operation (`SnbtOperations`).
//! - Compounds (`{key: value, …}`, a trailing comma allowed, a key quoted or not but not empty,
//!   a repeated key keeping its first place and last value), lists (`[a, b]`) and arrays
//!   (`[B; …]`, `[I; …]`, `[L; …]` of integers of the types each takes).
//!
//! Java's strings are UTF-16, and an escape can make half of a surrogate pair, which the next
//! escape completes (`"😀"` is 😀), so the string rules' values are UTF-16 units
//! (`Vec<u16>`), made into a Rust string when the ops makes a string of one: a half that stays
//! alone is U+FFFD there, where Java's string holds it.
//!
//! Not ported: `\N{name}` escapes' names, which need `Character.codePointOf` and Unicode's whole
//! table of character names. Such an escape is read up to its `}` like Java's, and then fails
//! the way an unknown name does in Java, with a message saying the names aren't ported (Java's
//! `ERROR_INVALID_CHARACTER_NAME` for names it doesn't know). The parser's suggestions aren't
//! ported either (see `ErrorCollector`).

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use crate::com::google::common::primitives::unsigned_bytes::UnsignedBytes;
use crate::com::mojang::brigadier::exceptions::dynamic_command_exception_type::DynamicCommandExceptionType;
use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::com::mojang::brigadier::literal_message::LiteralMessage;
use crate::com::mojang::brigadier::message::Message;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::java_ops::{JavaOps, JavaValue};
use crate::java::lang::byte::Byte;
use crate::java::lang::character::Character;
use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::lang::integer::Integer;
use crate::java::lang::long::Long;
use crate::java::lang::number::Number;
use crate::java::lang::number_format_exception::NumberFormatException;
use crate::java::lang::short::Short;
use crate::java::util::hex_format::HexFormat;
use crate::net::minecraft::nbt::snbt_operations::{self, BuiltinKey};
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::contents::translatable_contents::TranslationArgument;
use crate::net::minecraft::util::parsing::packrat::atom::Atom;
use crate::net::minecraft::util::parsing::packrat::commands::grammar::Grammar;
use crate::net::minecraft::util::parsing::packrat::commands::greedy_pattern_parse_rule::GreedyPatternParseRule;
use crate::net::minecraft::util::parsing::packrat::commands::greedy_predicate_parse_rule::GreedyPredicateParseRule;
use crate::net::minecraft::util::parsing::packrat::commands::number_run_parse_rule::NumberRunParseRule;
use crate::net::minecraft::util::parsing::packrat::commands::string_reader_terms::{
    TerminalCharacters, character, characters,
};
use crate::net::minecraft::util::parsing::packrat::commands::unquoted_string_parse_rule::UnquotedStringParseRule;
use crate::net::minecraft::util::parsing::packrat::delayed_exception::{self, DelayedException};
use crate::net::minecraft::util::parsing::packrat::dictionary::Dictionary;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::term;

// `Component.translatableEscape(key, arg)` as a message.
fn translatable_escape(key: &str, arg: String) -> Arc<dyn Message> {
    Arc::new(Component::from(Component::translatable_with_args(
        key,
        vec![TranslationArgument::String(arg)],
    )))
}

fn simple(key: &str) -> SimpleCommandExceptionType {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(key)))
}

static ERROR_NUMBER_PARSE_FAILURE: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|message| {
        translatable_escape("snbt.parser.number_parse_failure", message)
    });
static ERROR_EXPECTED_HEX_ESCAPE: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|length| {
        translatable_escape("snbt.parser.expected_hex_escape", length)
    });
static ERROR_INVALID_CODEPOINT: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|codepoint| {
        translatable_escape("snbt.parser.invalid_codepoint", codepoint)
    });
static ERROR_NO_SUCH_OPERATION: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|operation| {
        translatable_escape("snbt.parser.no_such_operation", operation)
    });

macro_rules! delayed {
    ($($name:ident = $key:literal;)*) => {$(
        static $name: LazyLock<DelayedException> =
            LazyLock::new(|| delayed_exception::create(simple($key)));
    )*};
}

delayed! {
    ERROR_EXPECTED_INTEGER_TYPE = "snbt.parser.expected_integer_type";
    ERROR_EXPECTED_FLOAT_TYPE = "snbt.parser.expected_float_type";
    ERROR_EXPECTED_NON_NEGATIVE_NUMBER = "snbt.parser.expected_non_negative_number";
    ERROR_INVALID_CHARACTER_NAME = "snbt.parser.invalid_character_name";
    ERROR_INVALID_ARRAY_ELEMENT_TYPE = "snbt.parser.invalid_array_element_type";
    ERROR_INVALID_UNQUOTED_START = "snbt.parser.invalid_unquoted_start";
    ERROR_EXPECTED_UNQUOTED_STRING = "snbt.parser.expected_unquoted_string";
    ERROR_INVALID_STRING_CONTENTS = "snbt.parser.invalid_string_contents";
    ERROR_EXPECTED_BINARY_NUMERAL = "snbt.parser.expected_binary_numeral";
    // Java's key, which `en_us` spells `undescore_not_allowed`: the message is the key.
    ERROR_UNDESCORE_NOT_ALLOWED = "snbt.parser.underscore_not_allowed";
    ERROR_EXPECTED_DECIMAL_NUMERAL = "snbt.parser.expected_decimal_numeral";
    ERROR_EXPECTED_HEX_NUMERAL = "snbt.parser.expected_hex_numeral";
    ERROR_EMPTY_KEY = "snbt.parser.empty_key";
    ERROR_LEADING_ZERO_NOT_ALLOWED = "snbt.parser.leading_zero_not_allowed";
    ERROR_INFINITY_NOT_ALLOWED = "snbt.parser.infinity_not_allowed";
}

/// What a `\N{name}` escape fails with: its names aren't ported.
static ERROR_CHARACTER_NAMES_NOT_PORTED: LazyLock<DelayedException> = LazyLock::new(|| {
    delayed_exception::create(SimpleCommandExceptionType::new(LiteralMessage::new(
        "Unicode character names (\\N{...}) aren't ported: they need Character.codePointOf",
    )))
});

fn is_ascii(c: u16, set: &[u8]) -> bool {
    set.iter().any(|&allowed| c == u16::from(allowed))
}

static BINARY_NUMERAL: LazyLock<NumberRunParseRule> = LazyLock::new(|| {
    NumberRunParseRule::new(
        ERROR_EXPECTED_BINARY_NUMERAL.clone(),
        ERROR_UNDESCORE_NOT_ALLOWED.clone(),
        |c| is_ascii(c, b"01_"),
    )
});
static DECIMAL_NUMERAL: LazyLock<NumberRunParseRule> = LazyLock::new(|| {
    NumberRunParseRule::new(
        ERROR_EXPECTED_DECIMAL_NUMERAL.clone(),
        ERROR_UNDESCORE_NOT_ALLOWED.clone(),
        |c| is_ascii(c, b"0123456789_"),
    )
});
static HEX_NUMERAL: LazyLock<NumberRunParseRule> = LazyLock::new(|| {
    NumberRunParseRule::new(
        ERROR_EXPECTED_HEX_NUMERAL.clone(),
        ERROR_UNDESCORE_NOT_ALLOWED.clone(),
        |c| is_ascii(c, b"0123456789ABCDEF_abcdef"),
    )
});
static PLAIN_STRING_CHUNK: LazyLock<GreedyPredicateParseRule> = LazyLock::new(|| {
    GreedyPredicateParseRule::new(1, ERROR_INVALID_STRING_CONTENTS.clone(), |c| {
        !is_ascii(c, b"\"'\\")
    })
});
static NUMBER_LOOKEAHEAD: LazyLock<TerminalCharacters> =
    LazyLock::new(|| TerminalCharacters::new(&[], can_start_number));

// `UNICODE_NAME`, `[-a-zA-Z0-9 ]+`: its class.
fn is_unicode_name_character(c: u16) -> bool {
    char::from_u32(u32::from(c)).is_some_and(|c| c.is_ascii_alphanumeric() || c == '-' || c == ' ')
}

fn create_number_parse_error(ex: &NumberFormatException) -> DelayedException {
    delayed_exception::create_with_argument(
        &ERROR_NUMBER_PARSE_FAILURE,
        ex.get_message().unwrap_or("null").to_owned(),
    )
}

/// The escape after a backslash for a control character: `b`, `t`, `n`, `f`, `r`, or `x` and its
/// two hex digits in upper case, or none for any other character.
pub fn escape_control_characters(c: char) -> Option<String> {
    match c {
        '\u{8}' => Some("b".to_owned()),
        '\t' => Some("t".to_owned()),
        '\n' => Some("n".to_owned()),
        '\u{c}' => Some("f".to_owned()),
        '\r' => Some("r".to_owned()),
        c if c < ' ' => Some(format!("x{:02X}", c as u32)),
        _ => None,
    }
}

fn is_allowed_to_start_unquoted_string(c: u16) -> bool {
    !can_start_number(c)
}

fn can_start_number(c: u16) -> bool {
    is_ascii(c, b"+-.0123456789")
}

fn needs_underscore_removal(contents: &[u16]) -> bool {
    contents.contains(&u16::from(b'_'))
}

fn clean_and_append(output: &mut Vec<u16>, contents: &[u16]) {
    clean_and_append_if(output, contents, needs_underscore_removal(contents));
}

fn clean_and_append_if(output: &mut Vec<u16>, contents: &[u16], needs_underscore_removal: bool) {
    if needs_underscore_removal {
        output.extend(contents.iter().filter(|&&c| c != u16::from(b'_')));
    } else {
        output.extend_from_slice(contents);
    }
}

fn parse_unsigned_short(string: &str, radix: i32) -> Result<i16, NumberFormatException> {
    let parse = Integer::parse_int_radix(string, radix)?;
    if parse >> 16 == 0 {
        Ok(parse as i16)
    } else {
        Err(NumberFormatException::new(format!("out of range: {parse}")))
    }
}

fn units(string: &str) -> Vec<u16> {
    string.encode_utf16().collect()
}

// A Java string of the grammar as the Rust one the ops takes.
fn string(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

fn store(state: &mut dyn ParseState<StringReader>, reason: &DelayedException) {
    let mark = state.mark();
    state.error_collector().store(mark, reason);
}

fn create_float<T: OpsValue>(
    ops: &dyn DynamicOps<T>,
    sign: Sign,
    whole: Option<Vec<u16>>,
    fraction: Option<Vec<u16>>,
    exponent: Option<Signed<Vec<u16>>>,
    type_suffix: Option<TypeSuffix>,
    state: &mut dyn ParseState<StringReader>,
) -> Option<T> {
    let mut result = Vec::new();
    sign.append(&mut result);
    if let Some(whole) = whole {
        clean_and_append(&mut result, &whole);
    }
    if let Some(fraction) = fraction {
        result.push(u16::from(b'.'));
        clean_and_append(&mut result, &fraction);
    }
    if let Some(exponent) = exponent {
        result.push(u16::from(b'e'));
        exponent.sign.append(&mut result);
        clean_and_append(&mut result, &exponent.value);
    }
    let contents = string(&result);
    let converted = match type_suffix {
        None | Some(TypeSuffix::DOUBLE) => convert_double(ops, state, &contents),
        Some(TypeSuffix::FLOAT) => convert_float(ops, state, &contents),
        Some(_) => {
            store(state, &ERROR_EXPECTED_FLOAT_TYPE);
            return None;
        }
    };
    match converted {
        Ok(value) => value,
        Err(e) => {
            store(state, &create_number_parse_error(&e));
            None
        }
    }
}

fn convert_float<T: OpsValue>(
    ops: &dyn DynamicOps<T>,
    state: &mut dyn ParseState<StringReader>,
    contents: &str,
) -> Result<Option<T>, NumberFormatException> {
    let value = Float::parse_float(contents)?;
    if !value.is_finite() {
        store(state, &ERROR_INFINITY_NOT_ALLOWED);
        return Ok(None);
    }
    Ok(Some(ops.create_float(value)))
}

fn convert_double<T: OpsValue>(
    ops: &dyn DynamicOps<T>,
    state: &mut dyn ParseState<StringReader>,
    contents: &str,
) -> Result<Option<T>, NumberFormatException> {
    let value = Double::parse_double(contents)?;
    if !value.is_finite() {
        store(state, &ERROR_INFINITY_NOT_ALLOWED);
        return Ok(None);
    }
    Ok(Some(ops.create_double(value)))
}

fn join_list(list: Vec<Vec<u16>>) -> Vec<u16> {
    list.concat()
}

/// `createParser`: SNBT's grammar making values of the ops, whose top rule is a literal.
pub fn create_parser<T: OpsValue + Send + Sync>(
    ops: &'static (dyn DynamicOps<T> + Sync),
) -> Grammar<T> {
    let true_value = ops.create_boolean(true);
    let false_value = ops.create_boolean(false);
    let empty_map_value = ops.empty_map();
    let empty_list = ops.empty_list();
    let rules: Dictionary<StringReader> = Dictionary::new();

    let sign: Atom<Sign> = Atom::of("sign");
    rules.put(
        sign,
        term::alternative(vec![
            term::sequence(vec![character('+'), term::marker(sign, Sign::PLUS)]),
            term::sequence(vec![character('-'), term::marker(sign, Sign::MINUS)]),
        ]),
        move |scope| scope.get_or_throw(&sign),
    );

    let integer_suffix: Atom<IntegerSuffix> = Atom::of("integer_suffix");
    let suffix = |v1, v2, signed, r#type| {
        term::sequence(vec![
            characters(v1, v2),
            term::marker(
                integer_suffix,
                IntegerSuffix {
                    signed,
                    r#type: Some(r#type),
                },
            ),
        ])
    };
    let signed_suffixes = |signed| {
        term::alternative(vec![
            suffix('b', 'B', Some(signed), TypeSuffix::BYTE),
            suffix('s', 'S', Some(signed), TypeSuffix::SHORT),
            suffix('i', 'I', Some(signed), TypeSuffix::INT),
            suffix('l', 'L', Some(signed), TypeSuffix::LONG),
        ])
    };
    rules.put(
        integer_suffix,
        term::alternative(vec![
            term::sequence(vec![
                characters('u', 'U'),
                signed_suffixes(SignedPrefix::UNSIGNED),
            ]),
            term::sequence(vec![
                characters('s', 'S'),
                signed_suffixes(SignedPrefix::SIGNED),
            ]),
            suffix('b', 'B', None, TypeSuffix::BYTE),
            suffix('s', 'S', None, TypeSuffix::SHORT),
            suffix('i', 'I', None, TypeSuffix::INT),
            suffix('l', 'L', None, TypeSuffix::LONG),
        ]),
        move |scope| scope.get_or_throw(&integer_suffix),
    );

    let binary_numeral: Atom<Vec<u16>> = Atom::of("binary_numeral");
    rules.put_rule(binary_numeral, &*BINARY_NUMERAL);
    let decimal_numeral: Atom<Vec<u16>> = Atom::of("decimal_numeral");
    rules.put_rule(decimal_numeral, &*DECIMAL_NUMERAL);
    let hex_numeral: Atom<Vec<u16>> = Atom::of("hex_numeral");
    rules.put_rule(hex_numeral, &*HEX_NUMERAL);

    let integer_literal: Atom<IntegerLiteral> = Atom::of("integer_literal");
    let integer_literal_rule = rules.put(
        integer_literal,
        term::sequence(vec![
            term::optional(rules.named(sign)),
            term::alternative(vec![
                term::sequence(vec![
                    character('0'),
                    term::cut(),
                    term::alternative(vec![
                        term::sequence(vec![
                            characters('x', 'X'),
                            term::cut(),
                            rules.named(hex_numeral),
                        ]),
                        term::sequence(vec![characters('b', 'B'), rules.named(binary_numeral)]),
                        term::sequence(vec![
                            rules.named(decimal_numeral),
                            term::cut(),
                            term::fail(ERROR_LEADING_ZERO_NOT_ALLOWED.clone()),
                        ]),
                        term::marker(decimal_numeral, units("0")),
                    ]),
                ]),
                rules.named(decimal_numeral),
            ]),
            term::optional(rules.named(integer_suffix)),
        ]),
        move |scope| {
            let suffix = scope
                .get_or_default(&integer_suffix, IntegerSuffix::EMPTY)
                .expect("a null integer suffix");
            let sign_value = scope
                .get_or_default(&sign, Sign::PLUS)
                .expect("a null sign");
            if let Some(decimal_contents) = scope.get(&decimal_numeral) {
                return Some(IntegerLiteral {
                    sign: sign_value,
                    base: Base::DECIMAL,
                    digits: decimal_contents,
                    suffix,
                });
            }
            if let Some(hex_contents) = scope.get(&hex_numeral) {
                return Some(IntegerLiteral {
                    sign: sign_value,
                    base: Base::HEX,
                    digits: hex_contents,
                    suffix,
                });
            }
            let binary_contents = scope
                .get_or_throw(&binary_numeral)
                .expect("null binary digits");
            Some(IntegerLiteral {
                sign: sign_value,
                base: Base::BINARY,
                digits: binary_contents,
                suffix,
            })
        },
    );

    let float_type_suffix: Atom<TypeSuffix> = Atom::of("float_type_suffix");
    rules.put(
        float_type_suffix,
        term::alternative(vec![
            term::sequence(vec![
                characters('f', 'F'),
                term::marker(float_type_suffix, TypeSuffix::FLOAT),
            ]),
            term::sequence(vec![
                characters('d', 'D'),
                term::marker(float_type_suffix, TypeSuffix::DOUBLE),
            ]),
        ]),
        move |scope| scope.get_or_throw(&float_type_suffix),
    );

    let float_exponent_part: Atom<Signed<Vec<u16>>> = Atom::of("float_exponent_part");
    rules.put(
        float_exponent_part,
        term::sequence(vec![
            characters('e', 'E'),
            term::optional(rules.named(sign)),
            rules.named(decimal_numeral),
        ]),
        move |scope| {
            Some(Signed {
                sign: scope
                    .get_or_default(&sign, Sign::PLUS)
                    .expect("a null sign"),
                value: scope
                    .get_or_throw(&decimal_numeral)
                    .expect("null decimal digits"),
            })
        },
    );

    let float_whole_part: Atom<Vec<u16>> = Atom::of("float_whole_part");
    let float_fraction_part: Atom<Vec<u16>> = Atom::of("float_fraction_part");
    let float_literal: Atom<T> = Atom::of("float_literal");
    rules.put_complex(
        float_literal,
        term::sequence(vec![
            term::optional(rules.named(sign)),
            term::alternative(vec![
                term::sequence(vec![
                    rules.named_with_alias(decimal_numeral, float_whole_part),
                    character('.'),
                    term::cut(),
                    term::optional(rules.named_with_alias(decimal_numeral, float_fraction_part)),
                    term::optional(rules.named(float_exponent_part)),
                    term::optional(rules.named(float_type_suffix)),
                ]),
                term::sequence(vec![
                    character('.'),
                    term::cut(),
                    rules.named_with_alias(decimal_numeral, float_fraction_part),
                    term::optional(rules.named(float_exponent_part)),
                    term::optional(rules.named(float_type_suffix)),
                ]),
                term::sequence(vec![
                    rules.named_with_alias(decimal_numeral, float_whole_part),
                    rules.named(float_exponent_part),
                    term::cut(),
                    term::optional(rules.named(float_type_suffix)),
                ]),
                term::sequence(vec![
                    rules.named_with_alias(decimal_numeral, float_whole_part),
                    term::optional(rules.named(float_exponent_part)),
                    rules.named(float_type_suffix),
                ]),
            ]),
        ]),
        move |state| {
            let scope = state.scope();
            let whole_sign = scope
                .get_or_default(&sign, Sign::PLUS)
                .expect("a null sign");
            let whole = scope.get(&float_whole_part);
            let fraction = scope.get(&float_fraction_part);
            let exponent = scope.get(&float_exponent_part);
            let type_suffix = scope.get(&float_type_suffix);
            create_float(
                ops,
                whole_sign,
                whole,
                fraction,
                exponent,
                type_suffix,
                state,
            )
        },
    );

    let string_hex_2: Atom<Vec<u16>> = Atom::of("string_hex_2");
    rules.put_rule(string_hex_2, simple_hex_literal_parse_rule(2));
    let string_hex_4: Atom<Vec<u16>> = Atom::of("string_hex_4");
    rules.put_rule(string_hex_4, simple_hex_literal_parse_rule(4));
    let string_hex_8: Atom<Vec<u16>> = Atom::of("string_hex_8");
    rules.put_rule(string_hex_8, simple_hex_literal_parse_rule(8));
    let string_unicode_name: Atom<Vec<u16>> = Atom::of("string_unicode_name");
    rules.put_rule(
        string_unicode_name,
        GreedyPatternParseRule::new(
            is_unicode_name_character,
            ERROR_INVALID_CHARACTER_NAME.clone(),
        ),
    );

    let string_escape_sequence: Atom<Vec<u16>> = Atom::of("string_escape_sequence");
    let escape = |c, escaped: &str| {
        term::sequence(vec![
            character(c),
            term::marker(string_escape_sequence, units(escaped)),
        ])
    };
    rules.put_complex(
        string_escape_sequence,
        term::alternative(vec![
            escape('b', "\u{8}"),
            escape('s', " "),
            escape('t', "\t"),
            escape('n', "\n"),
            escape('f', "\u{c}"),
            escape('r', "\r"),
            escape('\\', "\\"),
            escape('\'', "'"),
            escape('"', "\""),
            term::sequence(vec![character('x'), rules.named(string_hex_2)]),
            term::sequence(vec![character('u'), rules.named(string_hex_4)]),
            term::sequence(vec![character('U'), rules.named(string_hex_8)]),
            term::sequence(vec![
                character('N'),
                character('{'),
                rules.named(string_unicode_name),
                character('}'),
            ]),
        ]),
        move |state| {
            let scope = state.scope();
            if let Some(plain_escape) = scope.get_any(&[&string_escape_sequence]) {
                return Some(plain_escape);
            }
            if let Some(hex_escape) = scope.get_any(&[&string_hex_2, &string_hex_4, &string_hex_8])
            {
                let code_point = HexFormat::from_hex_digits(&hex_escape);
                if !Character::is_valid_code_point(code_point) {
                    store(
                        state,
                        &delayed_exception::create_with_argument(
                            &ERROR_INVALID_CODEPOINT,
                            format!("U+{:08X}", code_point as u32),
                        ),
                    );
                    return None;
                }
                return Some(Character::to_string_code_point(code_point));
            }
            // `Character.codePointOf(name)` isn't ported: every name fails, like one Java doesn't
            // know, with a message of its own.
            scope
                .get_or_throw(&string_unicode_name)
                .expect("a null character name");
            store(state, &ERROR_CHARACTER_NAMES_NOT_PORTED);
            None
        },
    );

    let string_plain_contents: Atom<Vec<u16>> = Atom::of("string_plain_contents");
    rules.put_rule(string_plain_contents, &*PLAIN_STRING_CHUNK);
    let string_chunks: Atom<Vec<Vec<u16>>> = Atom::of("string_chunks");
    let string_contents: Atom<Vec<u16>> = Atom::of("string_contents");

    let single_quoted_string_chunk: Atom<Vec<u16>> = Atom::of("single_quoted_string_chunk");
    let single_quoted_string_chunk_rule = rules.put(
        single_quoted_string_chunk,
        term::alternative(vec![
            rules.named_with_alias(string_plain_contents, string_contents),
            term::sequence(vec![
                character('\\'),
                rules.named_with_alias(string_escape_sequence, string_contents),
            ]),
            term::sequence(vec![
                character('"'),
                term::marker(string_contents, units("\"")),
            ]),
        ]),
        move |scope| scope.get_or_throw(&string_contents),
    );
    let single_quoted_string_contents: Atom<Vec<u16>> = Atom::of("single_quoted_string_contents");
    rules.put(
        single_quoted_string_contents,
        term::repeated(single_quoted_string_chunk_rule, string_chunks),
        move |scope| scope.get_or_throw(&string_chunks).map(join_list),
    );

    let double_quoted_string_chunk: Atom<Vec<u16>> = Atom::of("double_quoted_string_chunk");
    let double_quoted_string_chunk_rule = rules.put(
        double_quoted_string_chunk,
        term::alternative(vec![
            rules.named_with_alias(string_plain_contents, string_contents),
            term::sequence(vec![
                character('\\'),
                rules.named_with_alias(string_escape_sequence, string_contents),
            ]),
            term::sequence(vec![
                character('\''),
                term::marker(string_contents, units("'")),
            ]),
        ]),
        move |scope| scope.get_or_throw(&string_contents),
    );
    let double_quoted_string_contents: Atom<Vec<u16>> = Atom::of("double_quoted_string_contents");
    rules.put(
        double_quoted_string_contents,
        term::repeated(double_quoted_string_chunk_rule, string_chunks),
        move |scope| scope.get_or_throw(&string_chunks).map(join_list),
    );

    let quoted_string_literal: Atom<Vec<u16>> = Atom::of("quoted_string_literal");
    rules.put(
        quoted_string_literal,
        term::alternative(vec![
            term::sequence(vec![
                character('"'),
                term::cut(),
                term::optional(
                    rules.named_with_alias(double_quoted_string_contents, string_contents),
                ),
                character('"'),
            ]),
            term::sequence(vec![
                character('\''),
                term::optional(
                    rules.named_with_alias(single_quoted_string_contents, string_contents),
                ),
                character('\''),
            ]),
        ]),
        move |scope| scope.get_or_throw(&string_contents),
    );

    let unquoted_string: Atom<Vec<u16>> = Atom::of("unquoted_string");
    rules.put_rule(
        unquoted_string,
        UnquotedStringParseRule::new(1, ERROR_EXPECTED_UNQUOTED_STRING.clone()),
    );

    let literal: Atom<T> = Atom::of("literal");
    let argument_list: Atom<Vec<T>> = Atom::of("arguments");
    let literal_forward = rules.forward(literal);
    rules.put(
        argument_list,
        term::repeated_with_trailing_separator(literal_forward, argument_list, character(',')),
        move |scope| scope.get_or_throw(&argument_list),
    );

    let unquoted_string_or_built_in: Atom<T> = Atom::of("unquoted_string_or_builtin");
    {
        let true_value = true_value.clone();
        let false_value = false_value.clone();
        rules.put_complex(
            unquoted_string_or_built_in,
            term::sequence(vec![
                rules.named(unquoted_string),
                term::optional(term::sequence(vec![
                    character('('),
                    rules.named(argument_list),
                    character(')'),
                ])),
            ]),
            move |state| {
                let scope = state.scope();
                let contents = scope
                    .get_or_throw(&unquoted_string)
                    .expect("a null unquoted string");
                if contents.is_empty() || !is_allowed_to_start_unquoted_string(contents[0]) {
                    let mark = state.mark();
                    state
                        .error_collector()
                        .store(mark, &ERROR_INVALID_UNQUOTED_START);
                    return None;
                }
                if let Some(arguments) = scope.get(&argument_list) {
                    let key = BuiltinKey::new(string(&contents), arguments.len() as i32);
                    if let Some(operation) = snbt_operations::BUILTIN_OPERATIONS.get(&key) {
                        return operation.run(ops, &arguments, state);
                    }
                    store(
                        state,
                        &delayed_exception::create_with_argument(
                            &ERROR_NO_SUCH_OPERATION,
                            key.to_string(),
                        ),
                    );
                    return None;
                }
                let contents = string(&contents);
                if contents.eq_ignore_ascii_case("true") {
                    Some(true_value.clone())
                } else if contents.eq_ignore_ascii_case("false") {
                    Some(false_value.clone())
                } else {
                    Some(ops.create_string(&contents))
                }
            },
        );
    }

    let map_key: Atom<Vec<u16>> = Atom::of("map_key");
    rules.put(
        map_key,
        term::alternative(vec![
            rules.named(quoted_string_literal),
            rules.named(unquoted_string),
        ]),
        move |scope| scope.get_any_or_throw(&[&quoted_string_literal, &unquoted_string]),
    );

    let map_entry: Atom<(Vec<u16>, T)> = Atom::of("map_entry");
    let map_entry_rule = rules.put_complex(
        map_entry,
        term::sequence(vec![
            rules.named(map_key),
            character(':'),
            rules.named(literal),
        ]),
        move |state| {
            let scope = state.scope();
            let key = scope.get_or_throw(&map_key).expect("a null key");
            if key.is_empty() {
                store(state, &ERROR_EMPTY_KEY);
                return None;
            }
            let value = scope.get_or_throw(&literal).expect("a null value");
            Some((key, value))
        },
    );

    let map_entries: Atom<Vec<(Vec<u16>, T)>> = Atom::of("map_entries");
    rules.put(
        map_entries,
        term::repeated_with_trailing_separator(map_entry_rule, map_entries, character(',')),
        move |scope| scope.get_or_throw(&map_entries),
    );

    let map_literal: Atom<T> = Atom::of("map_literal");
    rules.put(
        map_literal,
        term::sequence(vec![
            character('{'),
            rules.named(map_entries),
            character('}'),
        ]),
        move |scope| {
            let entries = scope.get_or_throw(&map_entries).expect("null entries");
            if entries.is_empty() {
                return Some(empty_map_value.clone());
            }
            // `ImmutableMap.builderWithExpectedSize(…).put(…).buildKeepingLast()`: each key where
            // it first came, with the value it last had.
            let mut builder: Vec<Pair<T, T>> = Vec::with_capacity(entries.len());
            let mut places: HashMap<Vec<u16>, usize> = HashMap::new();
            for (key, value) in entries {
                match places.get(&key) {
                    Some(&place) => {
                        let first = builder[place].get_first().clone();
                        builder[place] = Pair::new(first, value);
                    }
                    None => {
                        places.insert(key.clone(), builder.len());
                        builder.push(Pair::new(ops.create_string(&string(&key)), value));
                    }
                }
            }
            Some(ops.create_map(builder))
        },
    );

    let list_entries: Atom<Vec<T>> = Atom::of("list_entries");
    let literal_forward = rules.forward(literal);
    rules.put(
        list_entries,
        term::repeated_with_trailing_separator(literal_forward, list_entries, character(',')),
        move |scope| scope.get_or_throw(&list_entries),
    );

    let array_prefix: Atom<ArrayPrefix> = Atom::of("array_prefix");
    rules.put(
        array_prefix,
        term::alternative(vec![
            term::sequence(vec![
                character('B'),
                term::marker(array_prefix, ArrayPrefix::BYTE),
            ]),
            term::sequence(vec![
                character('L'),
                term::marker(array_prefix, ArrayPrefix::LONG),
            ]),
            term::sequence(vec![
                character('I'),
                term::marker(array_prefix, ArrayPrefix::INT),
            ]),
        ]),
        move |scope| scope.get_or_throw(&array_prefix),
    );

    let int_array_entries: Atom<Vec<IntegerLiteral>> = Atom::of("int_array_entries");
    rules.put(
        int_array_entries,
        term::repeated_with_trailing_separator(
            integer_literal_rule,
            int_array_entries,
            character(','),
        ),
        move |scope| scope.get_or_throw(&int_array_entries),
    );

    let list_literal: Atom<T> = Atom::of("list_literal");
    rules.put_complex(
        list_literal,
        term::sequence(vec![
            character('['),
            term::alternative(vec![
                term::sequence(vec![
                    rules.named(array_prefix),
                    character(';'),
                    rules.named(int_array_entries),
                ]),
                rules.named(list_entries),
            ]),
            character(']'),
        ]),
        move |state| {
            let scope = state.scope();
            if let Some(array_type) = scope.get(&array_prefix) {
                let entries = scope
                    .get_or_throw(&int_array_entries)
                    .expect("null array entries");
                return if entries.is_empty() {
                    Some(array_type.create_empty(ops))
                } else {
                    array_type.create(ops, &entries, state)
                };
            }
            let entries = scope
                .get_or_throw(&list_entries)
                .expect("null list entries");
            if entries.is_empty() {
                Some(empty_list.clone())
            } else {
                Some(ops.create_list(entries))
            }
        },
    );

    let literal_rule = rules.put_complex(
        literal,
        term::alternative(vec![
            term::sequence(vec![
                term::positive_lookahead(Box::new(&*NUMBER_LOOKEAHEAD)),
                term::alternative(vec![
                    rules.named_with_alias(float_literal, literal),
                    rules.named(integer_literal),
                ]),
            ]),
            term::sequence(vec![
                term::positive_lookahead(characters('"', '\'')),
                term::cut(),
                rules.named(quoted_string_literal),
            ]),
            term::sequence(vec![
                term::positive_lookahead(character('{')),
                term::cut(),
                rules.named_with_alias(map_literal, literal),
            ]),
            term::sequence(vec![
                term::positive_lookahead(character('[')),
                term::cut(),
                rules.named_with_alias(list_literal, literal),
            ]),
            rules.named_with_alias(unquoted_string_or_built_in, literal),
        ]),
        move |state| {
            let scope = state.scope();
            if let Some(quoted_string) = scope.get(&quoted_string_literal) {
                return Some(ops.create_string(&string(&quoted_string)));
            }
            match scope.get(&integer_literal) {
                Some(integer) => integer.create(ops, state),
                None => scope.get_or_throw(&literal),
            }
        },
    );
    Grammar::new(rules, literal_rule)
}

/// An array's type, and the integer types its elements may have.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArrayPrefix {
    BYTE,
    INT,
    LONG,
}

impl ArrayPrefix {
    fn default_type(self) -> TypeSuffix {
        match self {
            ArrayPrefix::BYTE => TypeSuffix::BYTE,
            ArrayPrefix::INT => TypeSuffix::INT,
            ArrayPrefix::LONG => TypeSuffix::LONG,
        }
    }

    fn additional_types(self) -> &'static [TypeSuffix] {
        match self {
            ArrayPrefix::BYTE => &[],
            ArrayPrefix::INT => &[TypeSuffix::BYTE, TypeSuffix::SHORT],
            ArrayPrefix::LONG => &[TypeSuffix::BYTE, TypeSuffix::SHORT, TypeSuffix::INT],
        }
    }

    fn is_allowed(self, r#type: TypeSuffix) -> bool {
        r#type == self.default_type() || self.additional_types().contains(&r#type)
    }

    /// `create(DynamicOps)`: the empty array.
    fn create_empty<T: OpsValue>(self, ops: &dyn DynamicOps<T>) -> T {
        match self {
            ArrayPrefix::BYTE => ops.create_byte_list(&[]),
            ArrayPrefix::INT => ops.create_int_list(&[]),
            ArrayPrefix::LONG => ops.create_long_list(&[]),
        }
    }

    /// The array of the entries, each made a number of its type (`JavaOps`' box) and narrowed or
    /// widened to the array's.
    fn create<T: OpsValue>(
        self,
        ops: &dyn DynamicOps<T>,
        entries: &[IntegerLiteral],
        state: &mut dyn ParseState<StringReader>,
    ) -> Option<T> {
        let mut numbers = Vec::with_capacity(entries.len());
        for entry in entries {
            numbers.push(self.build_number(entry, state)?);
        }
        Some(match self {
            ArrayPrefix::BYTE => {
                let result: Vec<i8> = numbers.iter().map(Number::byte_value).collect();
                ops.create_byte_list(&result)
            }
            ArrayPrefix::INT => {
                let result: Vec<i32> = numbers.iter().map(Number::int_value).collect();
                ops.create_int_list(&result)
            }
            ArrayPrefix::LONG => {
                let result: Vec<i64> = numbers.iter().map(Number::long_value).collect();
                ops.create_long_list(&result)
            }
        })
    }

    fn build_number(
        self,
        entry: &IntegerLiteral,
        state: &mut dyn ParseState<StringReader>,
    ) -> Option<Number> {
        let Some(actual_type) = self.compute_type(&entry.suffix) else {
            store(state, &ERROR_INVALID_ARRAY_ELEMENT_TYPE);
            return None;
        };
        match entry.create_typed(&JavaOps::INSTANCE, actual_type, state)? {
            JavaValue::Number(number) => Some(number),
            value => panic!("java.lang.ClassCastException: {value} isn't a Number"),
        }
    }

    fn compute_type(self, value: &IntegerSuffix) -> Option<TypeSuffix> {
        match value.r#type {
            None => Some(self.default_type()),
            Some(r#type) => self.is_allowed(r#type).then_some(r#type),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Base {
    BINARY,
    DECIMAL,
    HEX,
}

/// An integer as it was written, made a number of a type when its literal or array is.
#[derive(Clone, Debug)]
struct IntegerLiteral {
    sign: Sign,
    base: Base,
    digits: Vec<u16>,
    suffix: IntegerSuffix,
}

impl IntegerLiteral {
    fn signed_or_default(&self) -> SignedPrefix {
        if let Some(signed) = self.suffix.signed {
            return signed;
        }
        match self.base {
            Base::BINARY | Base::HEX => SignedPrefix::UNSIGNED,
            Base::DECIMAL => SignedPrefix::SIGNED,
        }
    }

    fn cleanup_digits(&self, sign: Sign) -> Vec<u16> {
        let needs_underscore_removal = needs_underscore_removal(&self.digits);
        if sign != Sign::MINUS && !needs_underscore_removal {
            return self.digits.clone();
        }
        let mut result = Vec::new();
        sign.append(&mut result);
        clean_and_append_if(&mut result, &self.digits, needs_underscore_removal);
        result
    }

    /// `create(DynamicOps, ParseState)`: of its suffix's type, an int without one.
    fn create<T: OpsValue>(
        &self,
        ops: &dyn DynamicOps<T>,
        state: &mut dyn ParseState<StringReader>,
    ) -> Option<T> {
        self.create_typed(ops, self.suffix.r#type.unwrap_or(TypeSuffix::INT), state)
    }

    /// `create(DynamicOps, TypeSuffix, ParseState)`.
    fn create_typed<T: OpsValue>(
        &self,
        ops: &dyn DynamicOps<T>,
        r#type: TypeSuffix,
        state: &mut dyn ParseState<StringReader>,
    ) -> Option<T> {
        let is_signed = self.signed_or_default() == SignedPrefix::SIGNED;
        if !is_signed && self.sign == Sign::MINUS {
            store(state, &ERROR_EXPECTED_NON_NEGATIVE_NUMBER);
            return None;
        }
        let fixed_digits = string(&self.cleanup_digits(self.sign));
        let radix = match self.base {
            Base::BINARY => 2,
            Base::DECIMAL => 10,
            Base::HEX => 16,
        };
        let result =
            if is_signed {
                match r#type {
                    TypeSuffix::BYTE => Byte::parse_byte_radix(&fixed_digits, radix)
                        .map(|value| ops.create_byte(value)),
                    TypeSuffix::SHORT => Short::parse_short_radix(&fixed_digits, radix)
                        .map(|value| ops.create_short(value)),
                    TypeSuffix::INT => Integer::parse_int_radix(&fixed_digits, radix)
                        .map(|value| ops.create_int(value)),
                    TypeSuffix::LONG => Long::parse_long_radix(&fixed_digits, radix)
                        .map(|value| ops.create_long(value)),
                    TypeSuffix::FLOAT | TypeSuffix::DOUBLE => {
                        store(state, &ERROR_EXPECTED_INTEGER_TYPE);
                        return None;
                    }
                }
            } else {
                match r#type {
                    TypeSuffix::BYTE => UnsignedBytes::parse_unsigned_byte(&fixed_digits, radix)
                        .map(|value| ops.create_byte(value)),
                    TypeSuffix::SHORT => parse_unsigned_short(&fixed_digits, radix)
                        .map(|value| ops.create_short(value)),
                    TypeSuffix::INT => Integer::parse_unsigned_int_radix(&fixed_digits, radix)
                        .map(|value| ops.create_int(value)),
                    TypeSuffix::LONG => Long::parse_unsigned_long_radix(&fixed_digits, radix)
                        .map(|value| ops.create_long(value)),
                    TypeSuffix::FLOAT | TypeSuffix::DOUBLE => {
                        store(state, &ERROR_EXPECTED_INTEGER_TYPE);
                        return None;
                    }
                }
            };
        match result {
            Ok(value) => Some(value),
            Err(e) => {
                store(state, &create_number_parse_error(&e));
                None
            }
        }
    }
}

/// An integer's suffix: its signedness and type, either left out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IntegerSuffix {
    signed: Option<SignedPrefix>,
    r#type: Option<TypeSuffix>,
}

impl IntegerSuffix {
    const EMPTY: IntegerSuffix = IntegerSuffix {
        signed: None,
        r#type: None,
    };
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sign {
    PLUS,
    MINUS,
}

impl Sign {
    fn append(self, output: &mut Vec<u16>) {
        if self == Sign::MINUS {
            output.push(u16::from(b'-'));
        }
    }
}

#[derive(Clone, Debug)]
struct Signed<T> {
    sign: Sign,
    value: T,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SignedPrefix {
    SIGNED,
    UNSIGNED,
}

/// `SimpleHexLiteralParseRule`: exactly `size` hex digits.
fn simple_hex_literal_parse_rule(size: i32) -> GreedyPredicateParseRule {
    GreedyPredicateParseRule::with_max_size(
        size,
        size,
        delayed_exception::create_with_argument(&ERROR_EXPECTED_HEX_ESCAPE, size.to_string()),
        |c| is_ascii(c, b"0123456789ABCDEFabcdef"),
    )
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TypeSuffix {
    FLOAT,
    DOUBLE,
    BYTE,
    SHORT,
    INT,
    LONG,
}
