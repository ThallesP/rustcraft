//! net/minecraft/network/codec/StreamCodec.java
//!
//! Not ported yet: `dispatch`, which picks the codec by the value, and comes with its first caller
//! (particles, stats); and `recursive`, which comes with text components.
//!
//! Java's `StreamCodec<B, V>` is `StreamCodec<Buffer = B, Value = V>`: a codec reads and writes
//! one kind of buffer. Java passes a codec of a buffer's parent class where a codec of the buffer
//! is wanted (`StreamCodec<? super B, T>`), which is a codec whose `Buffer` the buffer is
//! `AsMut` of: a `FriendlyByteBuf` is `AsMut<ByteBuf>`. See PORTING.md.

use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_decoder::StreamDecoder;
use crate::net::minecraft::network::codec::stream_encoder::StreamEncoder;
use crate::net::minecraft::network::codec::stream_member_encoder::StreamMemberEncoder;

/// A codec held in a static field and made on first use: Java's `static final StreamCodec<B, V>`.
pub type StaticStreamCodec<B, V> =
    LazyLock<Box<dyn StreamCodec<Buffer = B, Value = V> + Send + Sync>>;

/// Reads values from a buffer and writes them to one. Java's `StreamEncoder` and `StreamDecoder`
/// are its parents, but they're functional interfaces any closure implements, so a codec isn't
/// one: a caller that wants one passes `|input| codec.decode(input)`.
pub trait StreamCodec {
    type Buffer;
    type Value;

    fn decode(&self, input: &mut Self::Buffer) -> Self::Value;

    fn encode(&self, output: &mut Self::Buffer, value: &Self::Value);

    /// `apply(CodecOperation)`: the codec the operation makes of this one, like
    /// `ByteBufCodecs::list()`'s.
    fn apply<O: CodecOperation<Self>>(self, operation: O) -> O::Output
    where
        Self: Sized,
    {
        operation.apply(self)
    }

    /// `map(Function, Function)`: this codec's values turned into others and back.
    fn map<O, F, G>(self, to: F, from: G) -> Mapped<Self, F, G>
    where
        Self: Sized,
        F: Fn(Self::Value) -> O,
        G: Fn(&O) -> Self::Value,
    {
        Mapped {
            original: self,
            to,
            from,
        }
    }

    /// `cast()`: this codec, for a buffer that wraps its own. Java returns `this`.
    fn cast<S: AsMut<Self::Buffer>>(self) -> Cast<Self, S>
    where
        Self: Sized,
    {
        Cast {
            original: self,
            buffer: PhantomData,
        }
    }

    /// `mapStream(Function)`: this codec on a `ByteBuf`, which `operation` wraps in this codec's
    /// buffer for each call, like the protocols' `FriendlyByteBuf::new`. Java's wrapper shares
    /// the buffer's bytes; here it takes the buffer for the call and hands it back after, even
    /// when the call panics. Java's is generic in the outer buffer, which is always a `ByteBuf`.
    fn map_stream<F>(self, operation: F) -> MapStream<Self, F>
    where
        Self: Sized,
        F: Fn(ByteBuf) -> Self::Buffer,
        Self::Buffer: Into<ByteBuf>,
    {
        MapStream {
            original: self,
            operation,
        }
    }
}

/// `StreamCodec.CodecOperation`: makes a codec of another. A functional interface, so any
/// `FnOnce(C) -> O` is one.
pub trait CodecOperation<C> {
    type Output: StreamCodec;

    fn apply(self, original: C) -> Self::Output;
}

impl<C, O: StreamCodec, F: FnOnce(C) -> O> CodecOperation<C> for F {
    type Output = O;

    fn apply(self, original: C) -> O {
        self(original)
    }
}

// Java shares codecs by reference: a codec field is used in other codecs (`BlockPos.STREAM_CODEC`
// in `BlockHitResult.STREAM_CODEC`), and holds them boxed.
impl<C: StreamCodec + ?Sized> StreamCodec for &C {
    type Buffer = C::Buffer;
    type Value = C::Value;

    fn decode(&self, input: &mut C::Buffer) -> C::Value {
        (**self).decode(input)
    }

    fn encode(&self, output: &mut C::Buffer, value: &C::Value) {
        (**self).encode(output, value)
    }
}

impl<C: StreamCodec + ?Sized> StreamCodec for Box<C> {
    type Buffer = C::Buffer;
    type Value = C::Value;

    fn decode(&self, input: &mut C::Buffer) -> C::Value {
        (**self).decode(input)
    }

    fn encode(&self, output: &mut C::Buffer, value: &C::Value) {
        (**self).encode(output, value)
    }
}

impl<C: StreamCodec, F: FnOnce() -> C> StreamCodec for LazyLock<C, F> {
    type Buffer = C::Buffer;
    type Value = C::Value;

    fn decode(&self, input: &mut C::Buffer) -> C::Value {
        (**self).decode(input)
    }

    fn encode(&self, output: &mut C::Buffer, value: &C::Value) {
        (**self).encode(output, value)
    }
}

/// What `map` returns.
pub struct Mapped<C, F, G> {
    original: C,
    to: F,
    from: G,
}

impl<O, C: StreamCodec, F: Fn(C::Value) -> O, G: Fn(&O) -> C::Value> StreamCodec
    for Mapped<C, F, G>
{
    type Buffer = C::Buffer;
    type Value = O;

    fn decode(&self, input: &mut C::Buffer) -> O {
        (self.to)(self.original.decode(input))
    }

    fn encode(&self, output: &mut C::Buffer, value: &O) {
        self.original.encode(output, &(self.from)(value))
    }
}

/// What `cast` returns.
pub struct Cast<C, S> {
    original: C,
    buffer: PhantomData<fn(&mut S)>,
}

impl<C: StreamCodec, S: AsMut<C::Buffer>> StreamCodec for Cast<C, S> {
    type Buffer = S;
    type Value = C::Value;

    fn decode(&self, input: &mut S) -> C::Value {
        self.original.decode(input.as_mut())
    }

    fn encode(&self, output: &mut S, value: &C::Value) {
        self.original.encode(output.as_mut(), value)
    }
}

/// What `map_stream` returns.
pub struct MapStream<C, F> {
    original: C,
    operation: F,
}

impl<C: StreamCodec, F: Fn(ByteBuf) -> C::Buffer> StreamCodec for MapStream<C, F>
where
    C::Buffer: Into<ByteBuf>,
{
    type Buffer = ByteBuf;
    type Value = C::Value;

    fn decode(&self, input: &mut ByteBuf) -> C::Value {
        let mut wrapped_stream = WrappedStream::new(input, &self.operation);
        self.original.decode(&mut wrapped_stream)
    }

    fn encode(&self, output: &mut ByteBuf, value: &C::Value) {
        let mut wrapped_stream = WrappedStream::new(output, &self.operation);
        self.original.encode(&mut wrapped_stream, value);
    }
}

/// The buffer `map_stream`'s operation made of another's bytes, which go back to the other when
/// it's dropped.
struct WrappedStream<'a, W: Into<ByteBuf>> {
    source: &'a mut ByteBuf,
    wrapped: Option<W>,
}

impl<'a, W: Into<ByteBuf>> WrappedStream<'a, W> {
    fn new(source: &'a mut ByteBuf, operation: impl Fn(ByteBuf) -> W) -> WrappedStream<'a, W> {
        let wrapped = operation(mem::replace(source, ByteBuf::empty()));
        WrappedStream {
            source,
            wrapped: Some(wrapped),
        }
    }
}

impl<W: Into<ByteBuf>> Deref for WrappedStream<'_, W> {
    type Target = W;

    fn deref(&self) -> &W {
        self.wrapped
            .as_ref()
            .expect("the buffer is only given back when dropped")
    }
}

impl<W: Into<ByteBuf>> DerefMut for WrappedStream<'_, W> {
    fn deref_mut(&mut self) -> &mut W {
        self.wrapped
            .as_mut()
            .expect("the buffer is only given back when dropped")
    }
}

impl<W: Into<ByteBuf>> Drop for WrappedStream<'_, W> {
    fn drop(&mut self) {
        if let Some(wrapped) = self.wrapped.take() {
            *self.source = wrapped.into();
        }
    }
}

/// `of(StreamEncoder, StreamDecoder)`. Java's anonymous classes implementing `decode` and
/// `encode` are this too.
pub fn of<B, V>(
    encoder: impl StreamEncoder<B, V>,
    decoder: impl StreamDecoder<B, V>,
) -> impl StreamCodec<Buffer = B, Value = V> {
    struct Of<B, V, E, D> {
        encoder: E,
        decoder: D,
        types: PhantomData<fn(&mut B) -> V>,
    }

    impl<B, V, E: StreamEncoder<B, V>, D: StreamDecoder<B, V>> StreamCodec for Of<B, V, E, D> {
        type Buffer = B;
        type Value = V;

        fn decode(&self, input: &mut B) -> V {
            self.decoder.decode(input)
        }

        fn encode(&self, output: &mut B, value: &V) {
            self.encoder.encode(output, value);
        }
    }

    Of {
        encoder,
        decoder,
        types: PhantomData,
    }
}

/// `ofMember(StreamMemberEncoder, StreamDecoder)`: a packet's codec, from its `write` method and
/// its constructor reading a buffer.
pub fn of_member<B, V>(
    encoder: impl StreamMemberEncoder<B, V>,
    decoder: impl StreamDecoder<B, V>,
) -> impl StreamCodec<Buffer = B, Value = V> {
    struct OfMember<B, V, E, D> {
        encoder: E,
        decoder: D,
        types: PhantomData<fn(&mut B) -> V>,
    }

    impl<B, V, E: StreamMemberEncoder<B, V>, D: StreamDecoder<B, V>> StreamCodec
        for OfMember<B, V, E, D>
    {
        type Buffer = B;
        type Value = V;

        fn decode(&self, input: &mut B) -> V {
            self.decoder.decode(input)
        }

        fn encode(&self, output: &mut B, value: &V) {
            self.encoder.encode(value, output);
        }
    }

    OfMember {
        encoder,
        decoder,
        types: PhantomData,
    }
}

/// `unit(Object)`: reads and writes nothing. Panics when asked to write any other value, where
/// Java throws `IllegalStateException`.
pub fn unit<B, V: PartialEq + Clone + Debug>(
    instance: V,
) -> impl StreamCodec<Buffer = B, Value = V> {
    struct Unit<B, V> {
        instance: V,
        buffer: PhantomData<fn(&mut B)>,
    }

    impl<B, V: PartialEq + Clone + Debug> StreamCodec for Unit<B, V> {
        type Buffer = B;
        type Value = V;

        fn decode(&self, _input: &mut B) -> V {
            self.instance.clone()
        }

        fn encode(&self, _output: &mut B, value: &V) {
            assert!(
                *value == self.instance,
                "Can't encode '{value:?}', expected '{:?}'",
                self.instance
            );
        }
    }

    Unit {
        instance,
        buffer: PhantomData,
    }
}

/// Java's `composite` overloads, one per number of fields: each field's codec and getter, then
/// the constructor taking the fields. They're read and written in order, each by its codec on
/// the composite's buffer `B`, which the field codecs' buffers are `AsMut` of.
macro_rules! composite {
    ($name:ident: $(($codec:ident $Codec:ident $getter:ident $Getter:ident $field:ident)),+) => {
        #[allow(clippy::too_many_arguments)]
        pub fn $name<B, C, $($Codec: StreamCodec,)+ $($Getter: Fn(&C) -> $Codec::Value,)+ K>(
            $($codec: $Codec, $getter: $Getter,)+
            constructor: K,
        ) -> impl StreamCodec<Buffer = B, Value = C>
        where
            B: $(AsMut<$Codec::Buffer> +)+,
            K: Fn($($Codec::Value),+) -> C,
        {
            struct Composite<B, C, $($Codec, $Getter,)+ K> {
                $($codec: $Codec, $getter: $Getter,)+
                constructor: K,
                types: PhantomData<fn(&mut B) -> C>,
            }

            impl<B, C, $($Codec: StreamCodec, $Getter: Fn(&C) -> $Codec::Value,)+ K> StreamCodec
                for Composite<B, C, $($Codec, $Getter,)+ K>
            where
                B: $(AsMut<$Codec::Buffer> +)+,
                K: Fn($($Codec::Value),+) -> C,
            {
                type Buffer = B;
                type Value = C;

                fn decode(&self, input: &mut B) -> C {
                    $(let $field = self.$codec.decode(AsMut::<$Codec::Buffer>::as_mut(input));)+
                    (self.constructor)($($field),+)
                }

                fn encode(&self, output: &mut B, value: &C) {
                    $(self.$codec.encode(AsMut::<$Codec::Buffer>::as_mut(output), &(self.$getter)(value));)+
                }
            }

            Composite {
                $($codec, $getter,)+
                constructor,
                types: PhantomData,
            }
        }
    };
}

composite!(composite1: (codec1 C1 getter1 G1 v1));
composite!(composite2: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2));
composite!(
    composite3: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3)
);
composite!(
    composite4: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4)
);
composite!(
    composite5: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4), (codec5 C5 getter5 G5 v5)
);
composite!(
    composite6: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4), (codec5 C5 getter5 G5 v5), (codec6 C6 getter6 G6 v6)
);
composite!(
    composite7: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4), (codec5 C5 getter5 G5 v5), (codec6 C6 getter6 G6 v6),
    (codec7 C7 getter7 G7 v7)
);
composite!(
    composite8: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4), (codec5 C5 getter5 G5 v5), (codec6 C6 getter6 G6 v6),
    (codec7 C7 getter7 G7 v7), (codec8 C8 getter8 G8 v8)
);
composite!(
    composite9: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4), (codec5 C5 getter5 G5 v5), (codec6 C6 getter6 G6 v6),
    (codec7 C7 getter7 G7 v7), (codec8 C8 getter8 G8 v8), (codec9 C9 getter9 G9 v9)
);
composite!(
    composite10: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4), (codec5 C5 getter5 G5 v5), (codec6 C6 getter6 G6 v6),
    (codec7 C7 getter7 G7 v7), (codec8 C8 getter8 G8 v8), (codec9 C9 getter9 G9 v9),
    (codec10 C10 getter10 G10 v10)
);
composite!(
    composite11: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4), (codec5 C5 getter5 G5 v5), (codec6 C6 getter6 G6 v6),
    (codec7 C7 getter7 G7 v7), (codec8 C8 getter8 G8 v8), (codec9 C9 getter9 G9 v9),
    (codec10 C10 getter10 G10 v10), (codec11 C11 getter11 G11 v11)
);
composite!(
    composite12: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4), (codec5 C5 getter5 G5 v5), (codec6 C6 getter6 G6 v6),
    (codec7 C7 getter7 G7 v7), (codec8 C8 getter8 G8 v8), (codec9 C9 getter9 G9 v9),
    (codec10 C10 getter10 G10 v10), (codec11 C11 getter11 G11 v11), (codec12 C12 getter12 G12 v12)
);
composite!(
    composite13: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4), (codec5 C5 getter5 G5 v5), (codec6 C6 getter6 G6 v6),
    (codec7 C7 getter7 G7 v7), (codec8 C8 getter8 G8 v8), (codec9 C9 getter9 G9 v9),
    (codec10 C10 getter10 G10 v10), (codec11 C11 getter11 G11 v11), (codec12 C12 getter12 G12 v12),
    (codec13 C13 getter13 G13 v13)
);
composite!(
    composite14: (codec1 C1 getter1 G1 v1), (codec2 C2 getter2 G2 v2), (codec3 C3 getter3 G3 v3),
    (codec4 C4 getter4 G4 v4), (codec5 C5 getter5 G5 v5), (codec6 C6 getter6 G6 v6),
    (codec7 C7 getter7 G7 v7), (codec8 C8 getter8 G8 v8), (codec9 C9 getter9 G9 v9),
    (codec10 C10 getter10 G10 v10), (codec11 C11 getter11 G11 v11), (codec12 C12 getter12 G12 v12),
    (codec13 C13 getter13 G13 v13), (codec14 C14 getter14 G14 v14)
);

#[cfg(test)]
mod tests {
    use super::StreamCodec;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
    use crate::net::minecraft::network::codec::stream_codec;
    use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
    use crate::net::minecraft::network::protocol::handshake::client_intention_packet::ClientIntentionPacket;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::util::mth::Mth;

    const CODEC: &str = "Lnet/minecraft/network/codec/StreamCodec;";

    // A codec whose buffer only its caller would pin, on a `ByteBuf` like vanilla's cases.
    fn on_byte_buf<C: StreamCodec<Buffer = ByteBuf>>(codec: C) -> C {
        codec
    }

    // The cases' name for the composite of these fields, each a codec and a getter, which takes
    // a Function, a BiFunction, or DataFixerUpper's Function3 and up to make the value.
    fn composite_name(codecs: &[&str]) -> String {
        let constructor = match codecs.len() {
            1 => "Ljava/util/function/Function;".to_owned(),
            2 => "Ljava/util/function/BiFunction;".to_owned(),
            fields => format!("Lcom/mojang/datafixers/util/Function{fields};"),
        };
        let fields = format!("{CODEC}Ljava/util/function/Function;").repeat(codecs.len());
        format!(
            "composite({fields}{constructor}){CODEC}[{}]",
            codecs.join(", ")
        )
    }

    // Vanilla's composites take VAR_INT, STRING_UTF8, BOOL and FLOAT fields in turn, and its
    // values are the lists of the fields, which are tuples here.
    macro_rules! check_composite {
        ($golden:ident, $composite:ident, $tuple:ty: $(($codec:ident $index:tt $field:ident)),+) => {
            $golden.check_codec(&composite_name(&[$(stringify!($codec)),+]), || {
                on_byte_buf(stream_codec::$composite(
                    $(ByteBufCodecs::$codec, |value: &$tuple| value.$index.clone(),)+
                    |$($field),+| ($($field,)+),
                ))
            });
        };
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.codec.StreamCodec");
        golden.pending(
            "dispatch(Ljava/util/function/Function;Ljava/util/function/Function;)Lnet/minecraft/network/codec/StreamCodec;",
            "picks the codec by the value, which comes with its first caller (particles, stats)",
        );
        golden.pending(
            "recursive(Ljava/util/function/UnaryOperator;)Lnet/minecraft/network/codec/StreamCodec;",
            "comes with text components",
        );
        golden.check_codec(
            &format!("of(Lnet/minecraft/network/codec/StreamEncoder;Lnet/minecraft/network/codec/StreamDecoder;){CODEC}[writeUUID, readUUID]"),
            || stream_codec::of(FriendlyByteBuf::write_uuid_to, FriendlyByteBuf::read_uuid_from),
        );
        golden.check_codec(
            &format!("ofMember(Lnet/minecraft/network/codec/StreamMemberEncoder;Lnet/minecraft/network/codec/StreamDecoder;){CODEC}[writeIdentifier, readIdentifier]"),
            || {
                stream_codec::of_member(
                    |identifier: &Identifier, output: &mut FriendlyByteBuf| {
                        output.write_identifier(identifier);
                    },
                    FriendlyByteBuf::read_identifier,
                )
            },
        );
        golden.check_codec(&format!("unit(Ljava/lang/Object;){CODEC}[NORTH]"), || {
            on_byte_buf(stream_codec::unit(Direction::NORTH))
        });
        golden.check_codec(
            &format!("apply(Lnet/minecraft/network/codec/StreamCodec$CodecOperation;){CODEC}[VAR_INT, optional]"),
            || ByteBufCodecs::VAR_INT.apply(ByteBufCodecs::optional),
        );
        golden.check_codec(
            &format!("map(Ljava/util/function/Function;Ljava/util/function/Function;){CODEC}[BYTE, unpackDegrees, packDegrees]"),
            || {
                ByteBufCodecs::BYTE
                    .map(Mth::unpack_degrees, |angle: &f32| Mth::pack_degrees(*angle))
            },
        );
        golden.check_codec(&format!("cast(){CODEC}[VAR_INT]"), || {
            ByteBufCodecs::VAR_INT.cast::<FriendlyByteBuf>()
        });
        golden.check_codec(
            &format!("mapStream(Ljava/util/function/Function;){CODEC}[ClientIntentionPacket.STREAM_CODEC, FriendlyByteBuf::new]"),
            || ClientIntentionPacket::STREAM_CODEC.map_stream(FriendlyByteBuf::new),
        );
        check_composite!(golden, composite1, (i32,): (VAR_INT 0 a));
        check_composite!(golden, composite2, (i32, String): (VAR_INT 0 a), (STRING_UTF8 1 b));
        check_composite!(
            golden, composite3, (i32, String, bool):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c)
        );
        check_composite!(
            golden, composite4, (i32, String, bool, f32):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d)
        );
        check_composite!(
            golden, composite5, (i32, String, bool, f32, i32):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d), (VAR_INT 4 e)
        );
        check_composite!(
            golden, composite6, (i32, String, bool, f32, i32, String):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d), (VAR_INT 4 e),
            (STRING_UTF8 5 f)
        );
        check_composite!(
            golden, composite7, (i32, String, bool, f32, i32, String, bool):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d), (VAR_INT 4 e),
            (STRING_UTF8 5 f), (BOOL 6 g)
        );
        check_composite!(
            golden, composite8, (i32, String, bool, f32, i32, String, bool, f32):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d), (VAR_INT 4 e),
            (STRING_UTF8 5 f), (BOOL 6 g), (FLOAT 7 h)
        );
        check_composite!(
            golden, composite9, (i32, String, bool, f32, i32, String, bool, f32, i32):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d), (VAR_INT 4 e),
            (STRING_UTF8 5 f), (BOOL 6 g), (FLOAT 7 h), (VAR_INT 8 i)
        );
        check_composite!(
            golden, composite10, (i32, String, bool, f32, i32, String, bool, f32, i32, String):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d), (VAR_INT 4 e),
            (STRING_UTF8 5 f), (BOOL 6 g), (FLOAT 7 h), (VAR_INT 8 i), (STRING_UTF8 9 j)
        );
        check_composite!(
            golden, composite11,
            (i32, String, bool, f32, i32, String, bool, f32, i32, String, bool):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d), (VAR_INT 4 e),
            (STRING_UTF8 5 f), (BOOL 6 g), (FLOAT 7 h), (VAR_INT 8 i), (STRING_UTF8 9 j),
            (BOOL 10 k)
        );
        check_composite!(
            golden, composite12,
            (i32, String, bool, f32, i32, String, bool, f32, i32, String, bool, f32):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d), (VAR_INT 4 e),
            (STRING_UTF8 5 f), (BOOL 6 g), (FLOAT 7 h), (VAR_INT 8 i), (STRING_UTF8 9 j),
            (BOOL 10 k), (FLOAT 11 l)
        );
        check_composite!(
            golden, composite13,
            (i32, String, bool, f32, i32, String, bool, f32, i32, String, bool, f32, i32):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d), (VAR_INT 4 e),
            (STRING_UTF8 5 f), (BOOL 6 g), (FLOAT 7 h), (VAR_INT 8 i), (STRING_UTF8 9 j),
            (BOOL 10 k), (FLOAT 11 l), (VAR_INT 12 m)
        );
        check_composite!(
            golden, composite14,
            (i32, String, bool, f32, i32, String, bool, f32, i32, String, bool, f32, i32, String):
            (VAR_INT 0 a), (STRING_UTF8 1 b), (BOOL 2 c), (FLOAT 3 d), (VAR_INT 4 e),
            (STRING_UTF8 5 f), (BOOL 6 g), (FLOAT 7 h), (VAR_INT 8 i), (STRING_UTF8 9 j),
            (BOOL 10 k), (FLOAT 11 l), (VAR_INT 12 m), (STRING_UTF8 13 n)
        );
        golden.finish();
    }
}
