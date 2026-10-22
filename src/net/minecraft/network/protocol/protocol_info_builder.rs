//! net/minecraft/network/protocol/ProtocolInfoBuilder.java
//!
//! Not ported yet: `addPacket` with a `CodecModifier`, `withBundlePacket`, `buildUnbound()` and
//! the protocols with a context (`contextServerboundProtocol`, `contextClientboundProtocol`),
//! which only the configuration and game protocols use, and come with them, with `CodecModifier`,
//! `BundlerInfo` and `UnboundProtocol`. `HandshakeProtocols` checks the rest against vanilla.
//!
//! The context `C` of a protocol without one is Java's `Unit`, which is `()`. Java's anonymous
//! `SimpleUnboundProtocol` and `ProtocolInfo.Details` are `SimpleUnbound` and `BuiltDetails`.

use std::sync::Arc;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::clientbound_packet_listener::ClientboundPacketListener;
use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
use crate::net::minecraft::network::connection_protocol::ConnectionProtocol;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::protocol::protocol_codec_builder::ProtocolCodecBuilder;
use crate::net::minecraft::network::protocol::simple_unbound_protocol::{
    ContextWrapper, SimpleUnboundProtocol,
};
use crate::net::minecraft::network::protocol_info::{Details, DetailsProvider, ProtocolInfo};
use crate::net::minecraft::network::serverbound_packet_listener::ServerboundPacketListener;

/// The packets that go one way in a protocol for the listener `T`, read from a `B` that wraps the
/// connection's buffer, in the order of their ids.
pub struct ProtocolInfoBuilder<T: ?Sized, B, C> {
    protocol: ConnectionProtocol,
    flow: PacketFlow,
    codecs: Vec<CodecEntry<T, B, C>>,
}

impl<T: ?Sized + 'static, B: AsMut<ByteBuf> + Into<ByteBuf> + 'static, C: Send + Sync + 'static>
    ProtocolInfoBuilder<T, B, C>
{
    pub fn new(protocol: ConnectionProtocol, flow: PacketFlow) -> ProtocolInfoBuilder<T, B, C> {
        ProtocolInfoBuilder {
            protocol,
            flow,
            codecs: Vec::new(),
        }
    }

    /// The next id's packet, and its codec, which reads `B` or a buffer `B` wraps. Java's codec
    /// objects are shared, and so are these: a `&'static` one, or a clone.
    pub fn add_packet<P, S>(
        &mut self,
        r#type: &'static PacketType,
        serializer: S,
    ) -> &mut ProtocolInfoBuilder<T, B, C>
    where
        P: Packet<T>,
        S: StreamCodec<Value = P> + Clone + Send + Sync + 'static,
        B: AsMut<S::Buffer>,
    {
        self.codecs.push(CodecEntry {
            r#type,
            add_to_builder: Box::new(move |codec_builder, context_wrapper, _context| {
                let context_wrapper = Arc::clone(context_wrapper);
                let base_codec = serializer
                    .clone()
                    .cast::<B>()
                    .map_stream(move |buffer| context_wrapper(buffer));
                codec_builder.add(r#type, base_codec);
            }),
        });
        self
    }

    fn build_packet_codec(
        flow: PacketFlow,
        context_wrapper: &ContextWrapper<B>,
        codecs: &[CodecEntry<T, B, C>],
        context: &C,
    ) -> impl StreamCodec<Buffer = ByteBuf, Value = Box<dyn Packet<T>>> + Send + Sync + 'static
    {
        let mut codec_builder = ProtocolCodecBuilder::new(flow);

        for codec in codecs {
            (codec.add_to_builder)(&mut codec_builder, context_wrapper, context);
        }

        codec_builder.build()
    }

    fn build_details(
        protocol: ConnectionProtocol,
        flow: PacketFlow,
        codecs: &[CodecEntry<T, B, C>],
    ) -> BuiltDetails {
        BuiltDetails {
            protocol,
            flow,
            types: codecs.iter().map(|codec| codec.r#type).collect(),
        }
    }

    /// The protocol's template, which `bind` makes protocols of with the codecs it has so far.
    /// Java copies the list, so later packets don't change it; Rust moves the builder in.
    pub fn build_unbound(self, context: C) -> Box<dyn SimpleUnboundProtocol<T, B>> {
        let details = ProtocolInfoBuilder::build_details(self.protocol, self.flow, &self.codecs);
        Box::new(SimpleUnbound {
            protocol: self.protocol,
            flow: self.flow,
            codecs: self.codecs,
            context,
            details,
        })
    }
}

impl<T: ?Sized + 'static, B: AsMut<ByteBuf> + Into<ByteBuf> + 'static>
    ProtocolInfoBuilder<T, B, ()>
{
    fn protocol(
        id: ConnectionProtocol,
        flow: PacketFlow,
        config: impl FnOnce(&mut ProtocolInfoBuilder<T, B, ()>),
    ) -> Box<dyn SimpleUnboundProtocol<T, B>> {
        let mut builder = ProtocolInfoBuilder::new(id, flow);
        config(&mut builder);
        builder.build_unbound(())
    }

    pub fn serverbound_protocol(
        id: ConnectionProtocol,
        config: impl FnOnce(&mut ProtocolInfoBuilder<T, B, ()>),
    ) -> Box<dyn SimpleUnboundProtocol<T, B>>
    where
        T: ServerboundPacketListener,
    {
        ProtocolInfoBuilder::protocol(id, PacketFlow::SERVERBOUND, config)
    }

    pub fn clientbound_protocol(
        id: ConnectionProtocol,
        config: impl FnOnce(&mut ProtocolInfoBuilder<T, B, ()>),
    ) -> Box<dyn SimpleUnboundProtocol<T, B>>
    where
        T: ClientboundPacketListener,
    {
        ProtocolInfoBuilder::protocol(id, PacketFlow::CLIENTBOUND, config)
    }
}

/// `ProtocolInfoBuilder.CodecEntry`: a packet's type, and how its codec joins a protocol's,
/// wrapping the buffer it reads with the protocol's context wrapper.
struct CodecEntry<T: ?Sized, B, C> {
    r#type: &'static PacketType,
    #[allow(clippy::type_complexity)]
    add_to_builder:
        Box<dyn Fn(&mut ProtocolCodecBuilder<ByteBuf, T>, &ContextWrapper<B>, &C) + Send + Sync>,
}

/// What `buildDetails` makes: the packets' types, whose ids are their places.
struct BuiltDetails {
    protocol: ConnectionProtocol,
    flow: PacketFlow,
    types: Vec<&'static PacketType>,
}

impl Details for BuiltDetails {
    fn id(&self) -> ConnectionProtocol {
        self.protocol
    }

    fn flow(&self) -> PacketFlow {
        self.flow
    }

    fn list_packets(&self, output: &mut dyn FnMut(&PacketType, i32)) {
        for (i, r#type) in self.types.iter().enumerate() {
            output(r#type, i as i32);
        }
    }
}

/// What `buildUnbound(C)` makes, holding the builder's protocol, flow, codecs and context.
struct SimpleUnbound<T: ?Sized, B, C> {
    protocol: ConnectionProtocol,
    flow: PacketFlow,
    codecs: Vec<CodecEntry<T, B, C>>,
    context: C,
    details: BuiltDetails,
}

impl<T: ?Sized + 'static, B: AsMut<ByteBuf> + Into<ByteBuf> + 'static, C: Send + Sync + 'static>
    SimpleUnboundProtocol<T, B> for SimpleUnbound<T, B, C>
{
    fn bind(&self, context_wrapper: ContextWrapper<B>) -> Box<dyn ProtocolInfo<T>> {
        Box::new(Implementation {
            id: self.protocol,
            flow: self.flow,
            codec: Box::new(ProtocolInfoBuilder::build_packet_codec(
                self.flow,
                &context_wrapper,
                &self.codecs,
                &self.context,
            )),
        })
    }
}

impl<T: ?Sized, B, C> DetailsProvider for SimpleUnbound<T, B, C> {
    fn details(&self) -> &dyn Details {
        &self.details
    }
}

type PacketCodec<L> =
    Box<dyn StreamCodec<Buffer = ByteBuf, Value = Box<dyn Packet<L>>> + Send + Sync>;

/// `ProtocolInfoBuilder.Implementation`: a bound protocol.
struct Implementation<L: ?Sized> {
    id: ConnectionProtocol,
    flow: PacketFlow,
    codec: PacketCodec<L>,
}

impl<L: ?Sized + 'static> ProtocolInfo<L> for Implementation<L> {
    fn id(&self) -> ConnectionProtocol {
        self.id
    }

    fn flow(&self) -> PacketFlow {
        self.flow
    }

    fn codec(&self) -> &dyn StreamCodec<Buffer = ByteBuf, Value = Box<dyn Packet<L>>> {
        &*self.codec
    }
}
