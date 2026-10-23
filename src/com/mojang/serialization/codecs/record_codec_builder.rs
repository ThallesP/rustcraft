//! com/mojang/serialization/codecs/RecordCodecBuilder.java (DataFixerUpper 10.0.21)
//!
//! A record's map codec, made of a map codec for each field and a constructor. Java builds it
//! through its applicative (`Instance`, `App`, `K1`, `Products`); here `group(…).apply(i, ctor)`
//! is `map_codec((f1, f2, …), ctor)` (or `create` for its `codec()`), each field
//! `MapCodecExt::for_getter`'s `RecordCodecBuilder`, of 1 to 16 fields, and what the applicative
//! does is written out:
//!
//! - Encoding writes each field in turn, in the group's order.
//! - Decoding reads every field. A record of one field is `lift1`'s `flatMap`: the field's error,
//!   or the record made from its value. A record of more is `DataResult`'s `ap2`, `ap3` and the
//!   compositions of them `Applicative`'s `apN` make, which all come to the same: the record made
//!   if every field has a value or a partial one, the errors' messages joined in the fields'
//!   reverse order, and the lifecycles added to `point`'s experimental one.
//!
//! `point`, `stable`, `deprecated` and `dependent` aren't ported: nothing the text components
//! reach calls them.

use std::marker::PhantomData;

use crate::com::mojang::serialization::data_result::{self, DataResult};
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::Valued;
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::com::mojang::serialization::map_codec::MapCodecCodec;
use crate::com::mojang::serialization::map_decoder::MapDecoder;
use crate::com::mojang::serialization::map_encoder::MapEncoder;
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;

/// One field of a record: its map codec, and the getter reading it from the record.
#[derive(Clone)]
pub struct RecordCodecBuilder<M, G> {
    getter: G,
    codec: M,
}

impl<M, G> RecordCodecBuilder<M, G> {
    /// `of(Function, MapCodec)`.
    pub fn of(getter: G, codec: M) -> RecordCodecBuilder<M, G> {
        RecordCodecBuilder { getter, codec }
    }
}

/// `build`'s map codec: the fields, and the constructor of the record from their values.
pub struct RecordMapCodec<O, F, C> {
    fields: F,
    constructor: C,
    record: PhantomData<fn() -> O>,
}

impl<O, F: Clone, C: Clone> Clone for RecordMapCodec<O, F, C> {
    fn clone(&self) -> Self {
        RecordMapCodec {
            fields: self.fields.clone(),
            constructor: self.constructor.clone(),
            record: PhantomData,
        }
    }
}

/// `mapCodec(i -> i.group(fields…).apply(i, constructor))`.
pub fn map_codec<O, F, C>(fields: F, constructor: C) -> RecordMapCodec<O, F, C> {
    RecordMapCodec {
        fields,
        constructor,
        record: PhantomData,
    }
}

/// `create(i -> i.group(fields…).apply(i, constructor))`: the map codec's `codec()`.
pub fn create<O, F, C>(fields: F, constructor: C) -> MapCodecCodec<RecordMapCodec<O, F, C>>
where
    RecordMapCodec<O, F, C>: crate::com::mojang::serialization::map_codec::MapCodecExt,
{
    use crate::com::mojang::serialization::map_codec::MapCodecExt;
    map_codec(fields, constructor).codec()
}

// One field: `lift1`, whose `flatMap` keeps a field's error as it is.
impl<O, C, M1: Valued, G1> Valued for RecordMapCodec<O, (RecordCodecBuilder<M1, G1>,), C>
where
    C: Fn(M1::Value) -> O,
{
    type Value = O;
}

impl<T: OpsValue, O, C, M1: Keyable<T>, G1> Keyable<T>
    for RecordMapCodec<O, (RecordCodecBuilder<M1, G1>,), C>
{
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        self.fields.0.codec.keys(ops)
    }
}

impl<T, O, C, M1, G1> MapEncoder<T> for RecordMapCodec<O, (RecordCodecBuilder<M1, G1>,), C>
where
    T: OpsValue,
    C: Fn(M1::Value) -> O,
    M1: MapEncoder<T>,
    G1: Fn(&O) -> M1::Value,
{
    fn encode(&self, input: &O, ops: &dyn DynamicOps<T>, prefix: &mut dyn RecordBuilder<T>) {
        let field = &self.fields.0;
        field.codec.encode(&(field.getter)(input), ops, prefix);
    }
}

impl<T, O, C, M1, G1> MapDecoder<T> for RecordMapCodec<O, (RecordCodecBuilder<M1, G1>,), C>
where
    T: OpsValue,
    C: Fn(M1::Value) -> O,
    M1: MapDecoder<T>,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<O> {
        self.fields
            .0
            .codec
            .decode(ops, input)
            .flat_map(|value| DataResult::success((self.constructor)(value)))
    }
}

macro_rules! record {
    ($($m:ident $g:ident $v:ident $index:tt),+) => {
        impl<O, C, $($m: Valued, $g),+> Valued
            for RecordMapCodec<O, ($(RecordCodecBuilder<$m, $g>,)+), C>
        where
            C: Fn($(<$m as Valued>::Value),+) -> O,
        {
            type Value = O;
        }

        impl<T: OpsValue, O, C, $($m: Keyable<T>, $g),+> Keyable<T>
            for RecordMapCodec<O, ($(RecordCodecBuilder<$m, $g>,)+), C>
        {
            fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
                let mut keys = Vec::new();
                $(keys.extend(self.fields.$index.codec.keys(ops));)+
                keys
            }
        }

        impl<T: OpsValue, O, C, $($m: MapEncoder<T>, $g: Fn(&O) -> <$m as Valued>::Value),+>
            MapEncoder<T> for RecordMapCodec<O, ($(RecordCodecBuilder<$m, $g>,)+), C>
        where
            C: Fn($(<$m as Valued>::Value),+) -> O,
        {
            fn encode(&self, input: &O, ops: &dyn DynamicOps<T>, prefix: &mut dyn RecordBuilder<T>) {
                $(
                    let field = &self.fields.$index;
                    field.codec.encode(&(field.getter)(input), ops, prefix);
                )+
            }
        }

        impl<T: OpsValue, O, C, $($m: MapDecoder<T>, $g),+> MapDecoder<T>
            for RecordMapCodec<O, ($(RecordCodecBuilder<$m, $g>,)+), C>
        where
            C: Fn($(<$m as Valued>::Value),+) -> O,
        {
            fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<O> {
                let mut lifecycle = Lifecycle::experimental();
                let mut messages = Vec::new();
                $(
                    let result = self.fields.$index.codec.decode(ops, input);
                    lifecycle = lifecycle.add(result.lifecycle());
                    let ($v, message) = result.split();
                    messages.extend(message);
                )+
                messages.reverse();
                let value = match ($($v,)+) {
                    ($(Some($v),)+) => Some((self.constructor)($($v),+)),
                    #[allow(unreachable_patterns)]
                    _ => None,
                };
                data_result::joined(messages, value, lifecycle)
            }
        }
    };
}

record!(M1 G1 v1 0, M2 G2 v2 1);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5, M7 G7 v7 6);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5, M7 G7 v7 6,
    M8 G8 v8 7);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5, M7 G7 v7 6,
    M8 G8 v8 7, M9 G9 v9 8);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5, M7 G7 v7 6,
    M8 G8 v8 7, M9 G9 v9 8, M10 G10 v10 9);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5, M7 G7 v7 6,
    M8 G8 v8 7, M9 G9 v9 8, M10 G10 v10 9, M11 G11 v11 10);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5, M7 G7 v7 6,
    M8 G8 v8 7, M9 G9 v9 8, M10 G10 v10 9, M11 G11 v11 10, M12 G12 v12 11);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5, M7 G7 v7 6,
    M8 G8 v8 7, M9 G9 v9 8, M10 G10 v10 9, M11 G11 v11 10, M12 G12 v12 11, M13 G13 v13 12);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5, M7 G7 v7 6,
    M8 G8 v8 7, M9 G9 v9 8, M10 G10 v10 9, M11 G11 v11 10, M12 G12 v12 11, M13 G13 v13 12,
    M14 G14 v14 13);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5, M7 G7 v7 6,
    M8 G8 v8 7, M9 G9 v9 8, M10 G10 v10 9, M11 G11 v11 10, M12 G12 v12 11, M13 G13 v13 12,
    M14 G14 v14 13, M15 G15 v15 14);
record!(M1 G1 v1 0, M2 G2 v2 1, M3 G3 v3 2, M4 G4 v4 3, M5 G5 v5 4, M6 G6 v6 5, M7 G7 v7 6,
    M8 G8 v8 7, M9 G9 v9 8, M10 G10 v10 9, M11 G11 v11 10, M12 G12 v12 11, M13 G13 v13 12,
    M14 G14 v14 13, M15 G15 v15 14, M16 G16 v16 15);
