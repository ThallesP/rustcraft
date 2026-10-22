//! Checks ported code against vanilla's answers in `vanilla/<version>/golden/`, which
//! `tools/golden.py` writes. See TESTING.md.
//!
//! A test loads one class's file, checks every method and constant in it, marks the rest
//! pending with a reason, then calls [`Golden::finish`]. Anything left unaccounted fails, so a
//! ported class is either fully verified or says what's missing.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::Once;

use serde_json::Value;

use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;

/// IntStreams can be endless, so vanilla records only their first elements.
pub const STREAM_LIMIT: usize = 512;

const MAX_REPORTED_MISMATCHES: usize = 5;

pub trait Decode: Sized {
    fn decode(value: &Value) -> Self;
}

pub trait Encode {
    fn encode(&self) -> Value;

    fn matches(&self, expected: &Value) -> bool {
        self.encode() == *expected
    }
}

macro_rules! integers {
    ($($type:ty),*) => {$(
        impl Decode for $type {
            fn decode(value: &Value) -> Self {
                let number = value.as_i64().unwrap_or_else(|| panic!("expected an integer, got {value}"));
                <$type>::try_from(number).unwrap_or_else(|_| panic!("{number} doesn't fit in {}", stringify!($type)))
            }
        }

        impl Encode for $type {
            fn encode(&self) -> Value {
                Value::from(*self)
            }
        }
    )*};
}

integers!(i8, i16, i32, i64, u16);

// Floats compare by bits, so -0.0 != 0.0, but any NaN matches any NaN: payloads never change game behaviour.
macro_rules! floats {
    ($($type:ty, $bits:ty, $digits:literal);*) => {$(
        impl Decode for $type {
            fn decode(value: &Value) -> Self {
                let hex = value.as_str().unwrap_or_else(|| panic!("expected float bits, got {value}"));
                <$type>::from_bits(<$bits>::from_str_radix(hex, 16).unwrap_or_else(|_| panic!("bad float bits {hex}")))
            }
        }

        impl Encode for $type {
            fn encode(&self) -> Value {
                Value::from(format!("{:0width$x}", self.to_bits(), width = $digits))
            }

            fn matches(&self, expected: &Value) -> bool {
                let expected = <$type>::decode(expected);
                (self.is_nan() && expected.is_nan()) || self.to_bits() == expected.to_bits()
            }
        }
    )*};
}

floats!(f32, u32, 8; f64, u64, 16);

impl Decode for bool {
    fn decode(value: &Value) -> Self {
        value
            .as_bool()
            .unwrap_or_else(|| panic!("expected a boolean, got {value}"))
    }
}

impl Encode for bool {
    fn encode(&self) -> Value {
        Value::from(*self)
    }
}

impl Decode for String {
    fn decode(value: &Value) -> Self {
        value
            .as_str()
            .unwrap_or_else(|| panic!("expected a string, got {value}"))
            .to_owned()
    }
}

impl Encode for String {
    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self) -> Value {
        self.iter().map(Encode::encode).collect()
    }

    fn matches(&self, expected: &Value) -> bool {
        expected.as_array().is_some_and(|expected| {
            expected.len() == self.len()
                && self
                    .iter()
                    .zip(expected)
                    .all(|(actual, expected)| actual.matches(expected))
        })
    }
}

/// Raw JSON, for hand-written suites like RandomSource's programs. A `{"throws": …}` object
/// matches any other, since Rust panics carry no Java exception class.
impl Decode for Value {
    fn decode(value: &Value) -> Self {
        value.clone()
    }
}

impl Encode for Value {
    fn encode(&self) -> Value {
        self.clone()
    }

    fn matches(&self, expected: &Value) -> bool {
        match (expected, self) {
            (Value::Object(expected), Value::Object(actual)) if expected.contains_key("throws") => {
                actual.contains_key("throws")
            }
            (Value::Array(expected), Value::Array(actual)) => {
                expected.len() == actual.len()
                    && actual
                        .iter()
                        .zip(expected)
                        .all(|(actual, expected)| actual.matches(expected))
            }
            _ => expected == self,
        }
    }
}

/// A function the golden args can be applied to, e.g. `Mth::clamp_i32` for `clamp(III)I`.
pub trait Call<Args> {
    fn call(&self, args: &[Value]) -> Box<dyn Encode>;
}

/// A function taking a `RandomSource` first. Vanilla passes a `LegacyRandomSource` seeded with
/// the first arg and records its next `nextLong()` after the call, which must match too.
pub trait CallWithRandom<Args> {
    fn call(&self, random: &mut LegacyRandomSource, args: &[Value]) -> Box<dyn Encode>;
}

macro_rules! calls {
    ($($type:ident $value:ident),*) => {
        impl<Function, Return, $($type),*> Call<($($type,)*)> for Function
        where
            Function: Fn($($type),*) -> Return,
            Return: Encode + 'static,
            $($type: Decode,)*
        {
            fn call(&self, args: &[Value]) -> Box<dyn Encode> {
                let [$($value),*] = args else {
                    let names: &[&str] = &[$(stringify!($value)),*];
                    panic!("expected {} arguments, got {}", names.len(), args.len());
                };
                Box::new(self($($type::decode($value)),*))
            }
        }

        impl<Function, Return, $($type),*> CallWithRandom<($($type,)*)> for Function
        where
            Function: Fn(&mut dyn RandomSource, $($type),*) -> Return,
            Return: Encode + 'static,
            $($type: Decode,)*
        {
            fn call(&self, random: &mut LegacyRandomSource, args: &[Value]) -> Box<dyn Encode> {
                let [$($value),*] = args else {
                    let names: &[&str] = &[$(stringify!($value)),*];
                    panic!("expected {} arguments, got {}", names.len(), args.len());
                };
                Box::new(self(random, $($type::decode($value)),*))
            }
        }
    };
}

calls!();
calls!(A a);
calls!(A a, B b);
calls!(A a, B b, C c);
calls!(A a, B b, C c, D d);
calls!(A a, B b, C c, D d, E e);
calls!(A a, B b, C c, D d, E e, F f);
calls!(A a, B b, C c, D d, E e, F f, G g);
calls!(A a, B b, C c, D d, E e, F f, G g, H h);
calls!(A a, B b, C c, D d, E e, F f, G g, H h, I i);
calls!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j);
calls!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k);

pub struct Golden {
    class: String,
    entries: BTreeMap<String, Entry>,
    accounted: RefCell<BTreeSet<String>>,
    failures: RefCell<Vec<String>>,
}

enum Entry {
    Field(Value),
    Cases(Vec<Case>),
    Skipped(String),
}

struct Case {
    args: Vec<Value>,
    expected: Expected,
    after: Option<Vec<Value>>,
}

enum Expected {
    Returns(Value),
    Throws(String),
}

impl Golden {
    pub fn load(class: &str) -> Golden {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vanilla")
            .join(include_str!("../MINECRAFT_VERSION").trim())
            .join("golden")
            .join(format!("{class}.jsonl"));
        let text = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "can't read {}: {error}. Run `python3 tools/golden.py` first.",
                path.display()
            )
        });

        let mut entries = BTreeMap::new();
        for line in text.lines() {
            let mut object: serde_json::Map<String, Value> =
                serde_json::from_str(line).expect("golden lines are JSON objects");
            if let Some(Value::String(field)) = object.remove("field") {
                entries.insert(
                    field,
                    Entry::Field(object.remove("value").expect("fields have a value")),
                );
                continue;
            }

            let Some(Value::String(method)) = object.remove("method") else {
                panic!("golden line has no method: {line}");
            };
            if let Some(Value::String(reason)) = object.remove("skipped") {
                entries.insert(method, Entry::Skipped(reason));
                continue;
            }

            let expected = match object.remove("throws") {
                Some(Value::String(exception)) => Expected::Throws(exception),
                _ => Expected::Returns(
                    object
                        .remove("result")
                        .expect("cases have a result or throw"),
                ),
            };
            let case = Case {
                args: into_array(object.remove("args")),
                expected,
                after: object.remove("after").map(|after| into_array(Some(after))),
            };
            match entries
                .entry(method)
                .or_insert_with(|| Entry::Cases(Vec::new()))
            {
                Entry::Cases(cases) => cases.push(case),
                _ => panic!("golden file mixes cases with a field or skip: {line}"),
            }
        }

        Golden {
            class: class.to_owned(),
            entries,
            accounted: RefCell::default(),
            failures: RefCell::default(),
        }
    }

    pub fn field(&self, name: &str, actual: impl Encode) {
        let Some(Entry::Field(expected)) = self.entry(name) else {
            return self.fail(format!("{name}: no golden value for this field"));
        };
        if !actual.matches(expected) {
            self.fail(format!(
                "{name}: expected {}, got {}",
                show(expected),
                show(&actual.encode())
            ));
        }
    }

    pub fn check<Args>(&self, method: &str, function: impl Call<Args>) {
        self.check_cases(method, |case| (catch(|| function.call(&case.args)), None));
    }

    pub fn check_random<Args>(&self, method: &str, function: impl CallWithRandom<Args>) {
        self.check_cases(method, |case| {
            let mut random = LegacyRandomSource::new(i64::decode(&case.args[0]));
            let outcome = catch(|| function.call(&mut random, &case.args[1..]));
            (outcome, Some(random.next_long()))
        });
    }

    /// Accounts for a method or constant that isn't verified yet, with the reason.
    pub fn pending(&self, name: &str, reason: &str) {
        if self.entry(name).is_none() {
            self.fail(format!(
                "{name} is marked pending ({reason}) but vanilla has no such entry"
            ));
        }
    }

    /// Accounts for everything not checked or pending so far, for classes that are only partly ported.
    pub fn pending_rest(&self, _reason: &str) {
        self.accounted
            .borrow_mut()
            .extend(self.entries.keys().cloned());
    }

    pub fn finish(self) {
        let accounted = self.accounted.borrow();
        for (name, entry) in &self.entries {
            if !accounted.contains(name) {
                let note = match entry {
                    Entry::Skipped(reason) => format!(" (vanilla skipped it: {reason})"),
                    _ => String::new(),
                };
                self.fail(format!("{name} is neither checked nor pending{note}"));
            }
        }

        let failures = self.failures.borrow();
        assert!(
            failures.is_empty(),
            "{} doesn't match vanilla:\n\n{}",
            self.class,
            failures.join("\n\n")
        );
    }

    fn entry(&self, name: &str) -> Option<&Entry> {
        self.accounted.borrow_mut().insert(name.to_owned());
        self.entries.get(name)
    }

    fn check_cases(
        &self,
        method: &str,
        mut run: impl FnMut(&Case) -> (Result<Box<dyn Encode>, String>, Option<i64>),
    ) {
        let cases = match self.entry(method) {
            Some(Entry::Cases(cases)) => cases,
            Some(Entry::Skipped(reason)) => {
                return self.fail(format!(
                    "{method}: vanilla skipped it ({reason}), so mark it pending"
                ));
            }
            _ => {
                return self.fail(format!(
                    "{method}: no golden cases (a typo, or vanilla changed)"
                ));
            }
        };

        let mut mismatches = Vec::new();
        for case in cases {
            let (outcome, after) = run(case);
            let mut problems = Vec::new();
            match (&case.expected, outcome) {
                (Expected::Returns(expected), Ok(actual)) if !actual.matches(expected) => {
                    problems.push(format!(
                        "expected {}, got {}",
                        show(expected),
                        show(&actual.encode())
                    ));
                }
                (Expected::Returns(expected), Err(message)) => {
                    problems.push(format!("expected {}, panicked: {message}", show(expected)))
                }
                (Expected::Throws(exception), Ok(actual)) => problems.push(format!(
                    "expected {exception}, got {}",
                    show(&actual.encode())
                )),
                _ => {}
            }
            if let (Some(after), Some(expected)) =
                (after, case.after.as_ref().and_then(|after| after.first()))
                && !after.matches(expected)
            {
                problems.push(format!(
                    "random afterwards: expected nextLong {}, got {after}",
                    show(expected)
                ));
            }
            if !problems.is_empty() {
                mismatches.push(format!(
                    "{} → {}",
                    show(&Value::from(case.args.clone())),
                    problems.join("; ")
                ));
            }
        }

        if !mismatches.is_empty() {
            let examples: Vec<_> = mismatches
                .iter()
                .take(MAX_REPORTED_MISMATCHES)
                .map(|mismatch| format!("  {mismatch}"))
                .collect();
            self.fail(format!(
                "{method}: {} of {} cases differ, e.g.\n{}",
                mismatches.len(),
                cases.len(),
                examples.join("\n")
            ));
        }
    }

    fn fail(&self, message: String) {
        self.failures.borrow_mut().push(message);
    }
}

fn into_array(value: Option<Value>) -> Vec<Value> {
    match value {
        Some(Value::Array(values)) => values,
        other => panic!("expected an array, got {other:?}"),
    }
}

// Float bits are unreadable, so shows the decimal next to them.
fn show(value: &Value) -> String {
    match value {
        Value::Array(values) => format!(
            "[{}]",
            values.iter().map(show).collect::<Vec<_>>().join(", ")
        ),
        Value::String(hex)
            if hex.len() == 8 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
        {
            format!("{hex}={:?}", f32::decode(value))
        }
        Value::String(hex)
            if hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
        {
            format!("{hex}={:?}", f64::decode(value))
        }
        _ => value.to_string(),
    }
}

thread_local! {
    static QUIET_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Runs `call` and catches its panic without printing it: when vanilla throws, the port must panic.
pub fn catch<R>(call: impl FnOnce() -> R) -> Result<R, String> {
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        let default = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if QUIET_DEPTH.get() == 0 {
                default(info);
            }
        }));
    });

    QUIET_DEPTH.set(QUIET_DEPTH.get() + 1);
    let result = panic::catch_unwind(AssertUnwindSafe(call));
    QUIET_DEPTH.set(QUIET_DEPTH.get() - 1);
    result.map_err(|payload| {
        payload
            .downcast_ref::<&str>()
            .map(|message| message.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "a non-string panic".to_owned())
    })
}
