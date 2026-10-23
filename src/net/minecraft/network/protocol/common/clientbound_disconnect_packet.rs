//! net/minecraft/network/protocol/common/ClientboundDisconnectPacket.java
//!
//! Not ported yet: the record's `hashCode`, which hashes the reason's contents and style
//! (`MutableComponent.hashCode`, not ported).

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

// `TRUSTED_CONTEXT_FREE_STREAM_CODEC.map(ClientboundDisconnectPacket::new, …::reason)`.
static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundDisconnectPacket> =
    LazyLock::new(|| {
        Box::new(
            ComponentSerialization::TRUSTED_CONTEXT_FREE_STREAM_CODEC.map(
                ClientboundDisconnectPacket::new,
                ClientboundDisconnectPacket::reason,
            ),
        )
    });

/// Why the server's ending the connection.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundDisconnectPacket {
    reason: Component,
}

impl ClientboundDisconnectPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundDisconnectPacket> =
        &STREAM_CODEC;

    pub fn new(reason: Component) -> ClientboundDisconnectPacket {
        ClientboundDisconnectPacket { reason }
    }

    pub fn reason(&self) -> Component {
        self.reason.clone()
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundDisconnectPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_DISCONNECT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_disconnect(self);
    }
}

/// `ClientboundDisconnectPacket[reason=…]`.
impl fmt::Display for ClientboundDisconnectPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientboundDisconnectPacket[reason={}]", self.reason)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundDisconnectPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundDisconnectPacket { reason: Component });

    type Listener = dyn ClientCommonPacketListener;
    type This = ClientboundDisconnectPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ClientboundDisconnectPacket");
        golden.pending("hashCode()I", "a component's hashCode isn't ported");
        golden.check_codec("STREAM_CODEC", || ClientboundDisconnectPacket::STREAM_CODEC);
        golden.check(
            "<init>(Lnet/minecraft/network/chat/Component;)V",
            ClientboundDisconnectPacket::new,
        );
        golden.check_method(
            "reason()Lnet/minecraft/network/chat/Component;",
            |this: &mut This| this.reason(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut This| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
