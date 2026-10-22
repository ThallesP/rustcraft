//! net/minecraft/network/protocol/ProtocolCodecBuilder.java
//!
//! `HandshakeProtocols`' codec checks this one against vanilla.

use std::any::Any;
use std::marker::PhantomData;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::id_dispatch_codec::{self, IdDispatchCodec};
use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

/// The codec of the packets that go one way in a protocol for the listener `L`, each by its type.
pub struct ProtocolCodecBuilder<B, L: ?Sized> {
    dispatch_builder: id_dispatch_codec::Builder<B, Box<dyn Packet<L>>, &'static PacketType>,
    flow: PacketFlow,
}

impl<B: AsMut<ByteBuf> + 'static, L: ?Sized + 'static> ProtocolCodecBuilder<B, L> {
    pub fn new(flow: PacketFlow) -> ProtocolCodecBuilder<B, L> {
        ProtocolCodecBuilder {
            dispatch_builder: IdDispatchCodec::<B, Box<dyn Packet<L>>, _>::builder(|packet| {
                packet.r#type()
            }),
            flow,
        }
    }

    /// Panics for a type that goes the other way, where Java throws `IllegalArgumentException`.
    pub fn add<P: Packet<L>>(
        &mut self,
        r#type: &'static PacketType,
        serializer: impl StreamCodec<Buffer = B, Value = P> + Send + Sync + 'static,
    ) -> &mut ProtocolCodecBuilder<B, L> {
        if r#type.flow() != self.flow {
            panic!(
                "Invalid packet flow for packet {type}, expected {:?}",
                self.flow
            );
        }

        self.dispatch_builder.add(
            r#type,
            Upcast {
                serializer,
                listener: PhantomData,
            },
        );
        self
    }

    pub fn build(
        self,
    ) -> impl StreamCodec<Buffer = B, Value = Box<dyn Packet<L>>> + Send + Sync + 'static {
        self.dispatch_builder.build()
    }
}

/// A packet's codec where a codec of any packet for `L` is wanted, which Java's generics allow
/// without a cast: it boxes what it reads, and casts what it's given to write back to its packet.
/// The dispatch codec picks it by the packet's type, so the cast never fails unless a packet's
/// `type()` is another's, where Java throws `ClassCastException`.
struct Upcast<C, L: ?Sized> {
    serializer: C,
    listener: PhantomData<fn(&mut L)>,
}

impl<C, P, L> StreamCodec for Upcast<C, L>
where
    C: StreamCodec<Value = P>,
    P: Packet<L>,
    L: ?Sized + 'static,
{
    type Buffer = C::Buffer;
    type Value = Box<dyn Packet<L>>;

    fn decode(&self, input: &mut C::Buffer) -> Box<dyn Packet<L>> {
        Box::new(self.serializer.decode(input))
    }

    fn encode(&self, output: &mut C::Buffer, value: &Box<dyn Packet<L>>) {
        let packet: &dyn Packet<L> = value.as_ref();
        let packet: &dyn Any = packet;
        let packet = packet
            .downcast_ref::<P>()
            .unwrap_or_else(|| panic!("ClassCastException: {value:?}"));
        self.serializer.encode(output, packet);
    }
}
