//! Checks ported code against vanilla's answers in `vanilla/<version>/golden/`, which
//! `tools/golden.py` writes. See TESTING.md.
//!
//! A test loads one class's file, checks every method and constant in it, marks the rest
//! pending with a reason, then calls [`Golden::finish`]. Anything left unaccounted fails, so a
//! ported class is either fully verified or says what's missing.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::{Arc, Once};

use serde_json::Value;

use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
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

    /// Never panics: a value of another shape just doesn't match.
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
                let bits = expected.as_str().filter(|hex| hex.len() == $digits);
                let Some(expected) = bits.and_then(|hex| <$bits>::from_str_radix(hex, 16).ok()) else {
                    return false;
                };
                let expected = <$type>::from_bits(expected);
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

/// Any JSON array, like an `EnumSet`'s constants.
impl<T: Decode> Decode for Vec<T> {
    fn decode(value: &Value) -> Self {
        value
            .as_array()
            .unwrap_or_else(|| panic!("expected an array, got {value}"))
            .iter()
            .map(T::decode)
            .collect()
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

/// Fixed-size arrays are JSON arrays too, like `FrustumIntersection`'s six `planes`.
impl<T: Decode, const N: usize> Decode for [T; N] {
    fn decode(value: &Value) -> Self {
        elements::<N>(value).each_ref().map(T::decode)
    }
}

impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode(&self) -> Value {
        self.iter().map(Encode::encode).collect()
    }

    fn matches(&self, expected: &Value) -> bool {
        expected.as_array().is_some_and(|expected| {
            expected.len() == N
                && self
                    .iter()
                    .zip(expected)
                    .all(|(actual, expected)| actual.matches(expected))
        })
    }
}

/// A null, for arguments a method lets be null (`Transformation`'s parts) and results it can
/// return null for (`Transformation.inverse`).
impl<T: Decode> Decode for Option<T> {
    fn decode(value: &Value) -> Self {
        (!value.is_null()).then(|| T::decode(value))
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self) -> Value {
        self.as_ref().map_or(Value::Null, Encode::encode)
    }

    fn matches(&self, expected: &Value) -> bool {
        match self {
            Some(value) => value.matches(expected),
            None => expected.is_null(),
        }
    }
}

/// What a method returning `void` returns.
impl Encode for () {
    fn encode(&self) -> Value {
        Value::Null
    }
}

/// Lets hand-written suites mix outputs of different types, e.g. one step per op of a program.
impl<T: Encode + ?Sized> Encode for Box<T> {
    fn encode(&self) -> Value {
        (**self).encode()
    }

    fn matches(&self, expected: &Value) -> bool {
        (**self).matches(expected)
    }
}

/// A shared value, like a record's array component: a fresh one for each argument.
impl<T: Decode> Decode for Arc<T> {
    fn decode(value: &Value) -> Self {
        Arc::new(T::decode(value))
    }
}

impl<T: Encode + ?Sized> Encode for Arc<T> {
    fn encode(&self) -> Value {
        (**self).encode()
    }

    fn matches(&self, expected: &Value) -> bool {
        (**self).matches(expected)
    }
}

/// Tuples are JSON arrays, like the `[output, pose, normal, trustedNormals]` steps of a program,
/// or a composite codec's fields.
macro_rules! tuples {
    ($($type:ident $index:tt),*) => {
        impl<$($type: Encode),*> Encode for ($($type,)*) {
            fn encode(&self) -> Value {
                Value::from(vec![$(self.$index.encode()),*])
            }

            #[allow(non_snake_case)]
            fn matches(&self, expected: &Value) -> bool {
                match expected.as_array().map(Vec::as_slice) {
                    Some([$($type),*]) => true $(&& self.$index.matches($type))*,
                    _ => false,
                }
            }
        }

        impl<$($type: Decode),*> Decode for ($($type,)*) {
            #[allow(non_snake_case)]
            fn decode(value: &Value) -> Self {
                match value.as_array().map(Vec::as_slice) {
                    Some([$($type),*]) => ($($type::decode($type),)*),
                    _ => panic!("expected a tuple, got {value}"),
                }
            }
        }
    };
}

tuples!(A 0);
tuples!(A 0, B 1);
tuples!(A 0, B 1, C 2);
tuples!(A 0, B 1, C 2, D 3);
tuples!(A 0, B 1, C 2, D 3, E 4);
tuples!(A 0, B 1, C 2, D 3, E 4, F 5);
tuples!(A 0, B 1, C 2, D 3, E 4, F 5, G 6);
tuples!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7);
tuples!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8);
tuples!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9);
tuples!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10);
tuples!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11);
tuples!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12);
tuples!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13);

/// An op of a hand-written suite's program that panicked, which ends the program. It matches
/// any `{"throws": …}`, since Rust panics carry no Java exception class.
pub struct Thrown;

impl Encode for Thrown {
    fn encode(&self) -> Value {
        serde_json::json!({ "throws": "panic" })
    }

    fn matches(&self, expected: &Value) -> bool {
        expected.get("throws").is_some()
    }
}

/// The elements of a fixed-size JSON array, e.g. an object encoded as its fields.
pub fn elements<const N: usize>(value: &Value) -> &[Value; N] {
    value
        .as_array()
        .and_then(|array| <&[Value; N]>::try_from(array.as_slice()).ok())
        .unwrap_or_else(|| panic!("expected an array of {N}, got {value}"))
}

/// Encodes a struct as the JSON array of the listed fields, the way vanilla encodes JOML's
/// objects: `encode_fields!(Vector3f { x: f32, y: f32, z: f32 });`. Use it in the struct's own
/// test module, which can see its private fields.
macro_rules! encode_fields {
    ($type:ty { $($field:ident: $field_type:ty),* $(,)? }) => {
        impl $crate::golden::Decode for $type {
            fn decode(value: &serde_json::Value) -> Self {
                let [$($field),*] = $crate::golden::elements(value);
                Self { $($field: <$field_type as $crate::golden::Decode>::decode($field)),* }
            }
        }

        impl $crate::golden::Encode for $type {
            fn encode(&self) -> serde_json::Value {
                serde_json::Value::from(vec![$($crate::golden::Encode::encode(&self.$field)),*])
            }

            fn matches(&self, expected: &serde_json::Value) -> bool {
                match expected.as_array().map(Vec::as_slice) {
                    Some([$($field),*]) => true $(&& $crate::golden::Encode::matches(&self.$field, $field))*,
                    _ => false,
                }
            }
        }
    };
}

pub(crate) use encode_fields;

/// Encodes an enum as its constants' names, the way vanilla encodes them:
/// `encode_names!(Direction { DOWN, UP, NORTH, SOUTH, WEST, EAST });`.
macro_rules! encode_names {
    ($type:ident { $($constant:ident),* $(,)? }) => {
        impl $crate::golden::Decode for $type {
            fn decode(value: &serde_json::Value) -> Self {
                match value.as_str() {
                    $(Some(stringify!($constant)) => $type::$constant,)*
                    _ => panic!("expected a {} constant, got {value}", stringify!($type)),
                }
            }
        }

        impl $crate::golden::Encode for $type {
            fn encode(&self) -> serde_json::Value {
                serde_json::Value::from(match self {
                    $($type::$constant => stringify!($constant),)*
                })
            }
        }
    };
}

pub(crate) use encode_names;

/// Encodes the packets of a protocol for the listener, values declared as a `Packet` (a
/// protocol codec's), the way vanilla does: `[type, fields]`, each packet by its own class's
/// encoding. It lists each of the protocol's types with its class:
/// `encode_packets!(dyn ServerHandshakePacketListener { HandshakePacketTypes::CLIENT_INTENTION => ClientIntentionPacket });`.
macro_rules! encode_packets {
    ($listener:ty { $($type:expr => $packet:ty),* $(,)? }) => {
        impl $crate::golden::Decode
            for Box<dyn $crate::net::minecraft::network::protocol::packet::Packet<$listener>>
        {
            fn decode(value: &serde_json::Value) -> Self {
                let [r#type, fields] = $crate::golden::elements(value);
                let r#type = <$crate::net::minecraft::network::protocol::packet_type::PacketType
                    as $crate::golden::Decode>::decode(r#type);
                $(if r#type == **$type {
                    return Box::new(<$packet as $crate::golden::Decode>::decode(fields));
                })*
                panic!("expected one of the protocol's packets, got {value}");
            }
        }

        impl $crate::golden::Encode
            for Box<dyn $crate::net::minecraft::network::protocol::packet::Packet<$listener>>
        {
            fn encode(&self) -> serde_json::Value {
                let r#type = $crate::golden::Encode::encode(self.r#type());
                let packet: &dyn $crate::net::minecraft::network::protocol::packet::Packet<$listener> = &**self;
                let packet: &dyn std::any::Any = packet;
                $(if let Some(packet) = packet.downcast_ref::<$packet>() {
                    return serde_json::Value::from(vec![r#type, $crate::golden::Encode::encode(packet)]);
                })*
                panic!("{self:?} isn't one of the protocol's packets");
            }

            fn matches(&self, expected: &serde_json::Value) -> bool {
                let Some([r#type, fields]) = expected.as_array().map(Vec::as_slice) else {
                    return false;
                };
                if !$crate::golden::Encode::matches(self.r#type(), r#type) {
                    return false;
                }
                let packet: &dyn $crate::net::minecraft::network::protocol::packet::Packet<$listener> = &**self;
                let packet: &dyn std::any::Any = packet;
                $(if let Some(packet) = packet.downcast_ref::<$packet>() {
                    return $crate::golden::Encode::matches(packet, fields);
                })*
                false
            }
        }
    };
}

pub(crate) use encode_packets;

/// A `RandomSource` argument of a method checked with `check_mut`, like `Vec3.offsetRandom`'s:
/// vanilla passes a `LegacyRandomSource` seeded with the arg, and records its next `nextLong()`
/// after the call.
impl Decode for LegacyRandomSource {
    fn decode(value: &Value) -> Self {
        LegacyRandomSource::new(i64::decode(value))
    }
}

impl Encode for LegacyRandomSource {
    fn encode(&self) -> Value {
        Value::from(self.clone().next_long())
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

/// An instance method with its receiver first, e.g. `|m: &mut Matrix4f, x: f32, y: f32, z: f32|
/// *m.translate(x, y, z)` for `translate(FFF)Lorg/joml/Matrix4f;`.
pub trait CallMethod<This, Args> {
    fn call(&self, this: &mut This, args: &[Value]) -> Box<dyn Encode>;
}

/// A function that may write to any of its arguments, which it takes as `&mut`, e.g.
/// `MatrixUtil::svd_decompose` writing the parts of its matrix into four of them.
pub trait CallMut<Args> {
    fn call(&self, args: &[Value]) -> Outcome;
}

/// What a call returned, or its panic, and what it can see of its arguments afterwards.
pub type Outcome = (Result<Box<dyn Encode>, String>, Vec<Box<dyn Encode>>);

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

        impl<Function, This, Return, $($type),*> CallMethod<This, ($($type,)*)> for Function
        where
            Function: Fn(&mut This, $($type),*) -> Return,
            Return: Encode + 'static,
            $($type: Decode,)*
        {
            fn call(&self, this: &mut This, args: &[Value]) -> Box<dyn Encode> {
                let [$($value),*] = args else {
                    let names: &[&str] = &[$(stringify!($value)),*];
                    panic!("expected {} arguments, got {}", names.len(), args.len());
                };
                Box::new(self(this, $($type::decode($value)),*))
            }
        }

        impl<Function, Return, $($type),*> CallMut<($($type,)*)> for Function
        where
            Function: Fn($(&mut $type),*) -> Return,
            Return: Encode + 'static,
            $($type: Decode + Encode + 'static,)*
        {
            #[allow(unused_mut)]
            fn call(&self, args: &[Value]) -> Outcome {
                let [$($value),*] = args else {
                    let names: &[&str] = &[$(stringify!($value)),*];
                    panic!("expected {} arguments, got {}", names.len(), args.len());
                };
                $(let mut $value = $type::decode($value);)*
                let outcome = catch(|| Box::new(self($(&mut $value),*)) as Box<dyn Encode>);
                (outcome, vec![$(Box::new($value) as Box<dyn Encode>),*])
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
calls!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k, L l);
calls!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k, L l, M m);
calls!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k, L l, M m, N n);
calls!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k, L l, M m, N n, O o);
calls!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k, L l, M m, N n, O o, P p);

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
        let file = File::open(&path).unwrap_or_else(|error| {
            panic!(
                "can't read {}: {error}. Run `python3 tools/golden.py` first.",
                path.display()
            )
        });

        // Line by line, so the file's text and its parsed cases are never in memory together.
        let mut entries = BTreeMap::new();
        for line in BufReader::new(file).lines() {
            let line =
                line.unwrap_or_else(|error| panic!("can't read {}: {error}", path.display()));
            let mut object: serde_json::Map<String, Value> =
                serde_json::from_str(&line).expect("golden lines are JSON objects");
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
        self.check_cases(method, |case| {
            (catch(|| function.call(&case.args)), Vec::new())
        });
    }

    pub fn check_random<Args>(&self, method: &str, function: impl CallWithRandom<Args>) {
        self.check_cases(method, |case| {
            let mut random = LegacyRandomSource::new(i64::decode(&case.args[0]));
            let outcome = catch(|| function.call(&mut random, &case.args[1..]));
            (
                outcome,
                vec![Box::new(random.next_long()) as Box<dyn Encode>],
            )
        });
    }

    /// Checks an instance method on the receiver in `args[0]`, which must also end up in
    /// vanilla's state afterwards (`after[0]`). Also fits a static method that may write to its
    /// first argument, like `MatrixUtil.checkProperty`.
    pub fn check_method<This, Args>(&self, method: &str, function: impl CallMethod<This, Args>)
    where
        This: Decode + Encode + 'static,
    {
        self.check_cases(method, |case| {
            let mut this = This::decode(&case.args[0]);
            let outcome = catch(|| function.call(&mut this, &case.args[1..]));
            (outcome, vec![Box::new(this) as Box<dyn Encode>])
        });
    }

    /// Checks a function that may write to any of its arguments: each must end up in vanilla's
    /// state afterwards, or as it was where vanilla left its slot empty. Fits a static method
    /// with out-arguments (`MatrixUtil.svdDecompose`), or an instance method that also changes
    /// an argument (`Transformation.slerp` decomposes the other transformation).
    pub fn check_mut<Args>(&self, method: &str, function: impl CallMut<Args>) {
        self.check_cases(method, |case| function.call(&case.args));
    }

    /// Checks a stream codec, which `make` makes inside each checked call, so a panic making it
    /// counts as the call's. `{name}.decode` cases read the buffer in `args[0]`, and
    /// `{name}.encode` cases write `args[1]` into it, the buffer afterwards in `after[0]` either way.
    pub fn check_codec<C: StreamCodec>(&self, name: &str, make: impl Fn() -> C)
    where
        C::Buffer: Decode + Encode + 'static,
        C::Value: Decode + Encode + 'static,
    {
        self.check_mut(&format!("{name}.decode"), |input: &mut C::Buffer| {
            make().decode(input)
        });
        self.check_mut(
            &format!("{name}.encode"),
            |output: &mut C::Buffer, value: &mut C::Value| make().encode(output, value),
        );
    }

    /// Like `check_codec`, for a codec made from an argument (`ByteBufCodecs.stringUtf8`'s
    /// maximum), which comes first in each case.
    pub fn check_codec_from<A, C: StreamCodec>(&self, name: &str, make: impl Fn(A) -> C)
    where
        A: Decode + Encode + Clone + 'static,
        C::Buffer: Decode + Encode + 'static,
        C::Value: Decode + Encode + 'static,
    {
        self.check_mut(
            &format!("{name}.decode"),
            |argument: &mut A, input: &mut C::Buffer| make(argument.clone()).decode(input),
        );
        self.check_mut(
            &format!("{name}.encode"),
            |argument: &mut A, output: &mut C::Buffer, value: &mut C::Value| {
                make(argument.clone()).encode(output, value)
            },
        );
    }

    /// Like `check_codec_from`, for a codec made from two arguments (`DiscardedPayload.codec`'s
    /// identifier and maximum).
    pub fn check_codec_from2<A1, A2, C: StreamCodec>(&self, name: &str, make: impl Fn(A1, A2) -> C)
    where
        A1: Decode + Encode + Clone + 'static,
        A2: Decode + Encode + Clone + 'static,
        C::Buffer: Decode + Encode + 'static,
        C::Value: Decode + Encode + 'static,
    {
        self.check_mut(
            &format!("{name}.decode"),
            |first: &mut A1, second: &mut A2, input: &mut C::Buffer| {
                make(first.clone(), second.clone()).decode(input)
            },
        );
        self.check_mut(
            &format!("{name}.encode"),
            |first: &mut A1, second: &mut A2, output: &mut C::Buffer, value: &mut C::Value| {
                make(first.clone(), second.clone()).encode(output, value)
            },
        );
    }

    /// Checks an `IntFunction`, which `make` makes inside each checked call: `{name}.apply`
    /// cases apply it to the id in `args[0]`.
    pub fn check_function<T, F>(&self, name: &str, make: impl Fn() -> F)
    where
        T: Encode + 'static,
        F: Fn(i32) -> T,
    {
        self.check(&format!("{name}.apply"), |id: i32| make()(id));
    }

    /// Like `check_function`, for a function made from an argument, which comes first in each
    /// case.
    pub fn check_function_from<A, T, F>(&self, name: &str, make: impl Fn(A) -> F)
    where
        A: Decode,
        T: Encode + 'static,
        F: Fn(i32) -> T,
    {
        self.check(&format!("{name}.apply"), |argument: A, id: i32| {
            make(argument)(id)
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

    /// `run` returns the outcome and what it can see of the arguments afterwards, from the first
    /// on, to compare with vanilla's `after`. Vanilla recording more than that fails the case.
    fn check_cases(&self, method: &str, mut run: impl FnMut(&Case) -> Outcome) {
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
            let expected_after = case.after.as_deref().unwrap_or_default();
            for (index, actual) in after.iter().enumerate() {
                match expected_after
                    .get(index)
                    .filter(|expected| !expected.is_null())
                {
                    Some(expected) if actual.matches(expected) => {}
                    Some(expected) => problems.push(format!(
                        "after[{index}]: expected {}, got {}",
                        show(expected),
                        show(&actual.encode())
                    )),
                    // Vanilla leaves a slot empty when the argument didn't change, or when it's the
                    // returned object, which the result already shows.
                    None => {
                        let unchanged = case.args.get(index).is_some_and(|arg| actual.matches(arg));
                        let returned = matches!(&case.expected, Expected::Returns(result) if actual.matches(result));
                        if !unchanged && !returned {
                            problems.push(format!(
                                "after[{index}]: vanilla left it as it was, got {}",
                                show(&actual.encode())
                            ));
                        }
                    }
                }
            }
            for (index, expected) in expected_after.iter().enumerate().skip(after.len()) {
                if !expected.is_null() {
                    problems.push(format!(
                        "after[{index}]: vanilla recorded {}, which this check can't see",
                        show(expected)
                    ));
                }
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
