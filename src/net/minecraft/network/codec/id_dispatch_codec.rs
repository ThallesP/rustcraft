//! net/minecraft/network/codec/IdDispatchCodec.java
//!
//! Java wraps an exception a value's codec throws in a `DecoderException` or `EncoderException`
//! naming the value's type, unless it's a `DontDecorateException` (the ones a skippable packet
//! throws). Exceptions are panics here, which carry no class, so the panic goes on as it is.
//! `HandshakeProtocols`' codec checks this one against vanilla.
//!
//! A codec `add` takes reads and writes values of `V` itself: Java's `StreamCodec<? super B, ?
//! extends V>` is a codec its caller casts, like `ProtocolCodecBuilder` boxing each packet.

use std::collections::HashMap;
use std::collections::hash_map::Entry as MapEntry;
use std::fmt::Display;
use std::hash::Hash;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
use crate::net::minecraft::network::var_int::VarInt;

type Serializer<B, V> = Box<dyn StreamCodec<Buffer = B, Value = V> + Send + Sync>;

/// Reads and writes values of several types: a VarInt id, which is the type's place in the
/// order they were added in, then the value by its type's codec.
pub struct IdDispatchCodec<B, V, T> {
    type_getter: Box<dyn Fn(&V) -> T + Send + Sync>,
    by_id: Vec<Entry<B, V, T>>,
    to_id: HashMap<T, i32>,
}

impl<B, V, T> IdDispatchCodec<B, V, T> {
    const UNKNOWN_TYPE: i32 = -1;

    pub fn builder(type_getter: impl Fn(&V) -> T + Send + Sync + 'static) -> Builder<B, V, T> {
        Builder {
            entries: Vec::new(),
            type_getter: Box::new(type_getter),
        }
    }
}

impl<B: AsMut<ByteBuf>, V, T: Eq + Hash + Display> StreamCodec for IdDispatchCodec<B, V, T> {
    type Buffer = B;
    type Value = V;

    fn decode(&self, input: &mut B) -> V {
        let id = VarInt::read(input.as_mut());
        if id >= 0 && (id as usize) < self.by_id.len() {
            let entry = &self.by_id[id as usize];
            entry.serializer.decode(input)
        } else {
            panic!("Received unknown packet id {id}");
        }
    }

    fn encode(&self, output: &mut B, value: &V) {
        let r#type = (self.type_getter)(value);
        let id = self
            .to_id
            .get(&r#type)
            .copied()
            .unwrap_or(IdDispatchCodec::<B, V, T>::UNKNOWN_TYPE);
        if id == IdDispatchCodec::<B, V, T>::UNKNOWN_TYPE {
            panic!("Sending unknown packet '{type}'");
        }

        VarInt::write(output.as_mut(), id);
        let entry = &self.by_id[id as usize];
        entry.serializer.encode(output, value);
    }
}

/// `IdDispatchCodec.Builder`: the types and their codecs, in the order of their ids.
pub struct Builder<B, V, T> {
    entries: Vec<Entry<B, V, T>>,
    type_getter: Box<dyn Fn(&V) -> T + Send + Sync>,
}

impl<B, V, T: Eq + Hash + Clone + Display> Builder<B, V, T> {
    pub fn add(
        &mut self,
        r#type: T,
        serializer: impl StreamCodec<Buffer = B, Value = V> + Send + Sync + 'static,
    ) -> &mut Builder<B, V, T> {
        self.entries.push(Entry {
            serializer: Box::new(serializer),
            r#type,
        });
        self
    }

    /// Panics when a type was added twice, where Java throws `IllegalStateException`.
    pub fn build(self) -> IdDispatchCodec<B, V, T> {
        let mut to_id = HashMap::new();

        for entry in &self.entries {
            let id = to_id.len() as i32;
            match to_id.entry(entry.r#type.clone()) {
                MapEntry::Occupied(_) => {
                    panic!("Duplicate registration for type {}", entry.r#type)
                }
                MapEntry::Vacant(vacant) => {
                    vacant.insert(id);
                }
            }
        }

        IdDispatchCodec {
            type_getter: self.type_getter,
            by_id: self.entries,
            to_id,
        }
    }
}

/// `IdDispatchCodec.Entry`.
struct Entry<B, V, T> {
    serializer: Serializer<B, V>,
    r#type: T,
}
