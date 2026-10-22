//! net/minecraft/network/protocol/common/custom/CustomPacketPayload.java
//!
//! Java's `equals`, `hashCode` and `toString` come from `Object`, and each payload (a record)
//! overrides them, so they're the trait's `equals`, `hash_code` and `Display`, like
//! `CustomQueryPayload`'s. A payload a packet holds is an `Arc<dyn CustomPacketPayload>`, Java's
//! shared reference, which a getter hands out and a codec's `map` gets back.
//!
//! The static methods are this module's functions (PORTING.md). `codec(FallbackProvider, List)`
//! is `codec_with_fallback`. `TypeAndCodec`'s type argument `T` only tells the compiler what the
//! codec reads, so it holds the codec as one of any payload, like `ProtocolCodecBuilder` boxing
//! each packet, and its buffer is the dispatch codec's (Java's `? super B`).

use std::any::Any;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::net::minecraft::network::codec::stream_codec::{self, StreamCodec};
use crate::net::minecraft::network::codec::stream_decoder::StreamDecoder;
use crate::net::minecraft::network::codec::stream_member_encoder::StreamMemberEncoder;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::resources::identifier::Identifier;

/// What a `custom_payload` packet carries after its identifier, which is its type's: a client's
/// brand, a mod's channel, or the scenario's start marker. Vanilla knows `BrandPayload`, and
/// reads any other as a `DiscardedPayload`.
pub trait CustomPacketPayload: Any + Debug + Display + Send + Sync {
    fn r#type(&self) -> Type;

    /// `equals(Object)`, with another payload.
    fn equals(&self, other: &dyn CustomPacketPayload) -> bool;

    fn hash_code(&self) -> i32;
}

impl PartialEq for dyn CustomPacketPayload {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

/// `codec(StreamMemberEncoder, StreamDecoder)`: a payload's codec, from its `write` method and its
/// constructor reading a buffer.
pub fn codec<B, T: CustomPacketPayload>(
    writer: impl StreamMemberEncoder<B, T>,
    reader: impl StreamDecoder<B, T>,
) -> impl StreamCodec<Buffer = B, Value = T> {
    stream_codec::of_member(writer, reader)
}

/// A type in the default namespace. Panics on a path `Identifier` refuses, where Java throws
/// `IdentifierException`.
pub fn create_type(id: &str) -> Type {
    Type::new(Identifier::with_default_namespace(id))
}

/// `codec(FallbackProvider, List)`: the payload's identifier, then the payload by the codec of
/// the type with that identifier, or the fallback's for any other. Panics on two types with one
/// identifier, where Java's `toUnmodifiableMap` throws `IllegalStateException`, and when asked
/// to write a payload the codec it picks doesn't read, where Java throws `ClassCastException`
/// after the identifier (a `DiscardedPayload` with the brand's identifier).
pub fn codec_with_fallback<B, F>(
    fallback: F,
    types: Vec<TypeAndCodec<B>>,
) -> impl StreamCodec<Buffer = B, Value = Arc<dyn CustomPacketPayload>>
where
    B: AsMut<FriendlyByteBuf>,
    F: FallbackProvider<B>,
{
    struct Dispatch<B, F> {
        fallback: F,
        id_to_type: HashMap<Identifier, PayloadCodec<B>>,
    }

    impl<B: AsMut<FriendlyByteBuf>, F: FallbackProvider<B>> StreamCodec for Dispatch<B, F> {
        type Buffer = B;
        type Value = Arc<dyn CustomPacketPayload>;

        fn decode(&self, input: &mut B) -> Arc<dyn CustomPacketPayload> {
            let identifier = input.as_mut().read_identifier();
            match self.id_to_type.get(&identifier) {
                Some(codec) => codec.decode(input),
                None => Arc::new(self.fallback.create(identifier).decode(input)),
            }
        }

        fn encode(&self, output: &mut B, value: &Arc<dyn CustomPacketPayload>) {
            let r#type = value.r#type();
            output.as_mut().write_identifier(r#type.id());
            match self.id_to_type.get(r#type.id()) {
                Some(codec) => codec.encode(output, value),
                None => {
                    let codec = self.fallback.create(r#type.id().clone());
                    let payload: &dyn Any = value.as_ref();
                    let payload = payload
                        .downcast_ref::<F::Payload>()
                        .unwrap_or_else(|| panic!("ClassCastException: {value:?}"));
                    codec.encode(output, payload);
                }
            }
        }
    }

    let mut id_to_type = HashMap::new();
    for TypeAndCodec { r#type, codec } in types {
        let id = r#type.id;
        assert!(
            !id_to_type.contains_key(&id),
            "IllegalStateException: Duplicate key {id}"
        );
        id_to_type.insert(id, codec);
    }
    Dispatch {
        fallback,
        id_to_type,
    }
}

/// `CustomPacketPayload.FallbackProvider`: the codec for a payload whose identifier no type has.
/// A functional interface, so any `Fn(Identifier) -> C` is one, for a codec `C` of a payload.
pub trait FallbackProvider<B> {
    type Payload: CustomPacketPayload;
    type Codec: StreamCodec<Buffer = B, Value = Self::Payload>;

    fn create(&self, type_id: Identifier) -> Self::Codec;
}

impl<B, C, F> FallbackProvider<B> for F
where
    F: Fn(Identifier) -> C,
    C: StreamCodec<Buffer = B, Value: CustomPacketPayload>,
{
    type Payload = C::Value;
    type Codec = C;

    fn create(&self, type_id: Identifier) -> C {
        self(type_id)
    }
}

/// `CustomPacketPayload.Type`: a payload's identifier, which a packet writes before it.
#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    id: Identifier,
}

impl Type {
    pub fn new(id: Identifier) -> Type {
        Type { id }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    /// A record's of one component: its hash.
    pub fn hash_code(&self) -> i32 {
        self.id.hash_code()
    }
}

/// A record's: `Type[id=minecraft:brand]`.
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type[id={}]", self.id)
    }
}

/// A payload's codec where a codec of any payload is wanted.
type PayloadCodec<B> =
    Box<dyn StreamCodec<Buffer = B, Value = Arc<dyn CustomPacketPayload>> + Send + Sync>;

/// `CustomPacketPayload.TypeAndCodec`: a payload's type and its codec, which
/// `codec_with_fallback` picks by the type's identifier.
pub struct TypeAndCodec<B> {
    r#type: Type,
    codec: PayloadCodec<B>,
}

impl<B: 'static> TypeAndCodec<B> {
    pub fn new<C>(r#type: Type, codec: C) -> TypeAndCodec<B>
    where
        C: StreamCodec<Buffer = B, Value: CustomPacketPayload> + Send + Sync + 'static,
    {
        TypeAndCodec {
            r#type,
            codec: Box::new(Upcast {
                codec,
                buffer: PhantomData,
            }),
        }
    }

    pub fn r#type(&self) -> &Type {
        &self.r#type
    }

    /// The codec, as one of any payload.
    pub fn codec(&self) -> &PayloadCodec<B> {
        &self.codec
    }
}

/// A payload's codec where a codec of any payload is wanted, which Java's generics allow without
/// a cast: it shares what it reads, and casts what it's given to write back to its payload,
/// panicking where Java throws `ClassCastException`.
struct Upcast<C, B> {
    codec: C,
    buffer: PhantomData<fn(&mut B)>,
}

impl<C, B> StreamCodec for Upcast<C, B>
where
    C: StreamCodec<Buffer = B, Value: CustomPacketPayload>,
{
    type Buffer = B;
    type Value = Arc<dyn CustomPacketPayload>;

    fn decode(&self, input: &mut B) -> Arc<dyn CustomPacketPayload> {
        Arc::new(self.codec.decode(input))
    }

    fn encode(&self, output: &mut B, value: &Arc<dyn CustomPacketPayload>) {
        let payload: &dyn Any = value.as_ref();
        let payload = payload
            .downcast_ref::<C::Value>()
            .unwrap_or_else(|| panic!("ClassCastException: {value:?}"));
        self.codec.encode(output, payload);
    }
}

#[cfg(test)]
pub mod tests {
    use std::any::Any;
    use std::sync::Arc;

    use serde_json::Value;

    use super::{CustomPacketPayload, Type, TypeAndCodec};
    use crate::golden::{Decode, Encode, Golden, elements, encode_fields};
    use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
    use crate::net::minecraft::network::protocol::common::custom::brand_payload::BrandPayload;
    use crate::net::minecraft::network::protocol::common::custom::discarded_payload::DiscardedPayload;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components.
    encode_fields!(Type { id: Identifier });

    /// A payload declared as the interface, which can be any of several classes: `[class,
    /// fields]`, by the class's simple name.
    impl Decode for Arc<dyn CustomPacketPayload> {
        fn decode(value: &Value) -> Self {
            let [class, fields] = elements(value);
            match class.as_str() {
                Some("BrandPayload") => Arc::new(BrandPayload::decode(fields)),
                Some("DiscardedPayload") => Arc::new(DiscardedPayload::decode(fields)),
                _ => panic!("expected one of vanilla's payloads, got {value}"),
            }
        }
    }

    impl Encode for dyn CustomPacketPayload {
        fn encode(&self) -> Value {
            let payload: &dyn Any = self;
            if let Some(brand) = payload.downcast_ref::<BrandPayload>() {
                return Value::from(vec![Value::from("BrandPayload"), brand.encode()]);
            }
            if let Some(discarded) = payload.downcast_ref::<DiscardedPayload>() {
                return Value::from(vec![Value::from("DiscardedPayload"), discarded.encode()]);
            }
            panic!("{self:?} isn't one of vanilla's payloads");
        }
    }

    #[test]
    fn custom_packet_payload_matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.custom.CustomPacketPayload");
        golden.pending(
            "type()Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload$Type;",
            "each payload's own, which its suite checks",
        );
        golden.check(
            "createType(Ljava/lang/String;)Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload$Type;",
            |id: String| super::create_type(&id),
        );
        golden.check_codec(
            "codec(Lnet/minecraft/network/codec/StreamMemberEncoder;Lnet/minecraft/network/codec/StreamDecoder;)Lnet/minecraft/network/codec/StreamCodec;[writeUtf, readUtf]",
            || {
                super::codec(
                    |payload: &BrandPayload, output: &mut FriendlyByteBuf| {
                        output.write_utf(payload.brand());
                    },
                    |input: &mut FriendlyByteBuf| BrandPayload::new(input.read_utf()),
                )
            },
        );
        golden.check_codec(
            "codec(Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload$FallbackProvider;Ljava/util/List;)Lnet/minecraft/network/codec/StreamCodec;[DiscardedPayload.codec(id, 32767), [BrandPayload]]",
            || {
                super::codec_with_fallback(
                    |id| DiscardedPayload::codec(id, 32767),
                    vec![TypeAndCodec::new(
                        (*BrandPayload::TYPE).clone(),
                        BrandPayload::STREAM_CODEC,
                    )],
                )
            },
        );
        golden.check_codec(
            "codec(Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload$FallbackProvider;Ljava/util/List;)Lnet/minecraft/network/codec/StreamCodec;[DiscardedPayload.codec(id, 0), []]",
            || {
                super::codec_with_fallback::<FriendlyByteBuf, _>(
                    |id| DiscardedPayload::codec(id, 0),
                    Vec::new(),
                )
            },
        );
        golden.finish();
    }

    #[test]
    fn type_matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.custom.CustomPacketPayload$Type");
        golden.check("<init>(Lnet/minecraft/resources/Identifier;)V", Type::new);
        golden.check_method(
            "id()Lnet/minecraft/resources/Identifier;",
            |this: &mut Type| this.id().clone(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Type, other: Type| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut Type| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Type| {
            this.to_string()
        });
        golden.finish();
    }
}
