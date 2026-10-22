//! net/minecraft/network/protocol/configuration/ClientboundResetChatPacket.java
//!
//! Java's constructor is private, so `INSTANCE` is the only one, and `unit` compares with it by
//! identity. Every value of a Rust unit struct is the same, so `PartialEq` is the same answer.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundResetChatPacket> =
    LazyLock::new(|| Box::new(stream_codec::unit(ClientboundResetChatPacket::INSTANCE)));

/// The server telling the client to start its chat session over, as when it comes back from
/// play to configuration. It has no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundResetChatPacket;

impl ClientboundResetChatPacket {
    pub const INSTANCE: ClientboundResetChatPacket = ClientboundResetChatPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundResetChatPacket> =
        &STREAM_CODEC;
}

impl<L: ClientConfigurationPacketListener + ?Sized> Packet<L> for ClientboundResetChatPacket {
    fn r#type(&self) -> &'static PacketType {
        ConfigurationPacketTypes::CLIENTBOUND_RESET_CHAT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_reset_chat(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ClientboundResetChatPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // No fields: [].
    impl Decode for ClientboundResetChatPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ClientboundResetChatPacket::INSTANCE
        }
    }

    impl Encode for ClientboundResetChatPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ClientConfigurationPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.configuration.ClientboundResetChatPacket");
        golden.field("INSTANCE", ClientboundResetChatPacket::INSTANCE);
        golden.check_codec("STREAM_CODEC", || ClientboundResetChatPacket::STREAM_CODEC);
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundResetChatPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/configuration/ClientConfigurationPacketListener;)V",
            |this: &mut ClientboundResetChatPacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
