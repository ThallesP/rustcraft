//! net/minecraft/advancements/predicates/MinMaxBounds.java
//!
//! Not ported yet: the codecs (`Bounds.createCodec`, `createStreamCodec`, `validateContainedInRange`
//! and each kind's `CODEC` and `STREAM_CODEC`), `asRange` and `asPoint`, which only they read, and
//! `map`. What's here is what a selector's options parse (`fromReader`) and test.
//!
//! Java's interface is generic over a boxed number; here `Bounds<T>` is generic over the number,
//! which `JavaNumber` compares like its box's `compareTo`, and each kind is a struct of its own.

use std::cmp::Ordering;
use std::sync::LazyLock;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::exceptions::dynamic_command_exception_type::DynamicCommandExceptionType;
use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::lang::integer::Integer;
use crate::java::lang::number_format_exception::NumberFormatException;
use crate::net::minecraft::commands::brigadier_exceptions::BrigadierExceptions;
use crate::net::minecraft::network::chat::component::Component;

static ERROR_EMPTY: LazyLock<SimpleCommandExceptionType> = LazyLock::new(|| {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(
        "argument.range.empty",
    )))
});
static ERROR_SWAPPED: LazyLock<SimpleCommandExceptionType> = LazyLock::new(|| {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(
        "argument.range.swapped",
    )))
});

/// A number a range can hold, compared like its Java box (`Integer`, `Float`, `Double`).
pub trait JavaNumber: Copy {
    fn compare_to(&self, other: &Self) -> Ordering;
}

impl JavaNumber for i32 {
    fn compare_to(&self, other: &i32) -> Ordering {
        self.cmp(other)
    }
}

impl JavaNumber for f32 {
    fn compare_to(&self, other: &f32) -> Ordering {
        Float::compare(*self, *other)
    }
}

impl JavaNumber for f64 {
    fn compare_to(&self, other: &f64) -> Ordering {
        Double::compare(*self, *other)
    }
}

/// `MinMaxBounds.Bounds`: a range, open at either end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}

impl<T: JavaNumber> Bounds<T> {
    pub fn is_any(&self) -> bool {
        self.min.is_none() && self.max.is_none()
    }

    pub fn are_swapped(&self) -> bool {
        match (&self.min, &self.max) {
            (Some(min), Some(max)) => min.compare_to(max) == Ordering::Greater,
            _ => false,
        }
    }

    pub fn any() -> Bounds<T> {
        Bounds {
            min: None,
            max: None,
        }
    }

    /// `N`, `N..`, `..N` or `N..M`, read with `converter`; an error rewinds to where the range
    /// started and says so.
    pub fn from_reader(
        reader: &mut StringReader,
        converter: fn(&str) -> Result<T, NumberFormatException>,
        parse_exc: &DynamicCommandExceptionType<String>,
    ) -> Result<Bounds<T>, CommandSyntaxException> {
        if !reader.can_read() {
            return Err(ERROR_EMPTY.create_with_context(reader));
        }
        let start = reader.get_cursor();
        let read = (|| {
            let min = Bounds::read_number(reader, converter, parse_exc)?;
            let max = if reader.can_read_length(2)
                && reader.peek() == u16::from(b'.')
                && reader.peek_offset(1) == u16::from(b'.')
            {
                reader.skip();
                reader.skip();
                Bounds::read_number(reader, converter, parse_exc)?
            } else {
                min
            };
            if min.is_none() && max.is_none() {
                Err(ERROR_EMPTY.create_with_context(reader))
            } else {
                Ok(Bounds { min, max })
            }
        })();
        read.map_err(|exception| {
            reader.set_cursor(start);
            CommandSyntaxException::with_raw(
                exception.get_raw_message().clone(),
                exception.get_input().map(str::to_owned),
                start,
            )
        })
    }

    fn read_number(
        reader: &mut StringReader,
        converter: fn(&str) -> Result<T, NumberFormatException>,
        parse_exc: &DynamicCommandExceptionType<String>,
    ) -> Result<Option<T>, CommandSyntaxException> {
        let start = reader.get_cursor();
        while reader.can_read() && Bounds::<T>::is_allowed_input_char(reader) {
            reader.skip();
        }
        let number = reader.substring(start, reader.get_cursor());
        if number.is_empty() {
            return Ok(None);
        }
        converter(&number)
            .map(Some)
            .map_err(|_| parse_exc.create_with_context(reader, number))
    }

    // A digit or `-`, or a `.` that doesn't start `..`.
    fn is_allowed_input_char(reader: &StringReader) -> bool {
        let c = reader.peek();
        if (c < u16::from(b'0') || c > u16::from(b'9')) && c != u16::from(b'-') {
            c == u16::from(b'.')
                && (!reader.can_read_length(2) || reader.peek_offset(1) != u16::from(b'.'))
        } else {
            true
        }
    }
}

/// `MinMaxBounds.Doubles`, with the squares of its bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Doubles {
    pub bounds: Bounds<f64>,
    pub bounds_sqr: Bounds<f64>,
}

impl Doubles {
    pub const ANY: Doubles = Doubles {
        bounds: Bounds {
            min: None,
            max: None,
        },
        bounds_sqr: Bounds {
            min: None,
            max: None,
        },
    };

    fn new(bounds: Bounds<f64>) -> Doubles {
        Doubles {
            bounds,
            bounds_sqr: Bounds {
                min: bounds.min.map(|min| min * min),
                max: bounds.max.map(|max| max * max),
            },
        }
    }

    pub fn min(&self) -> Option<f64> {
        self.bounds.min
    }

    pub fn max(&self) -> Option<f64> {
        self.bounds.max
    }

    pub fn matches(&self, value: f64) -> bool {
        if self.bounds.min.is_some_and(|min| min > value) {
            false
        } else {
            // Java's `!(max < value)`, true for a NaN.
            self.bounds
                .max
                .is_none_or(|max| max.partial_cmp(&value) != Some(Ordering::Less))
        }
    }

    pub fn from_reader(reader: &mut StringReader) -> Result<Doubles, CommandSyntaxException> {
        let start = reader.get_cursor();
        let bounds = Bounds::from_reader(
            reader,
            Double::parse_double,
            BrigadierExceptions::reader_invalid_double(),
        )?;
        if bounds.are_swapped() {
            reader.set_cursor(start);
            return Err(ERROR_SWAPPED.create_with_context(reader));
        }
        Ok(Doubles::new(bounds))
    }
}

/// `MinMaxBounds.FloatDegrees`: a range of angles, which may wrap (its bounds aren't checked).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatDegrees {
    pub bounds: Bounds<f32>,
}

impl FloatDegrees {
    pub fn min(&self) -> Option<f32> {
        self.bounds.min
    }

    pub fn max(&self) -> Option<f32> {
        self.bounds.max
    }

    pub fn from_reader(reader: &mut StringReader) -> Result<FloatDegrees, CommandSyntaxException> {
        let bounds = Bounds::from_reader(
            reader,
            Float::parse_float,
            BrigadierExceptions::reader_invalid_float(),
        )?;
        Ok(FloatDegrees { bounds })
    }
}

/// `MinMaxBounds.Ints`, with the squares of its bounds as longs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ints {
    pub bounds: Bounds<i32>,
    pub bounds_sqr: Bounds<i64>,
}

impl Ints {
    fn new(bounds: Bounds<i32>) -> Ints {
        Ints {
            bounds,
            bounds_sqr: Bounds {
                min: bounds.min.map(|min| i64::from(min) * i64::from(min)),
                max: bounds.max.map(|max| i64::from(max) * i64::from(max)),
            },
        }
    }

    pub fn min(&self) -> Option<i32> {
        self.bounds.min
    }

    pub fn max(&self) -> Option<i32> {
        self.bounds.max
    }

    pub fn matches(&self, value: i32) -> bool {
        if self.bounds.min.is_some_and(|min| min > value) {
            false
        } else {
            self.bounds.max.is_none_or(|max| max >= value)
        }
    }

    pub fn from_reader(reader: &mut StringReader) -> Result<Ints, CommandSyntaxException> {
        let start = reader.get_cursor();
        let bounds = Bounds::from_reader(
            reader,
            Integer::parse_int,
            BrigadierExceptions::reader_invalid_int(),
        )?;
        if bounds.are_swapped() {
            reader.set_cursor(start);
            return Err(ERROR_SWAPPED.create_with_context(reader));
        }
        Ok(Ints::new(bounds))
    }
}
