//! net/minecraft/network/protocol/configuration/ClientboundFinishConfigurationPacket.java
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

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundFinishConfigurationPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::unit(
            ClientboundFinishConfigurationPacket::INSTANCE,
        ))
    });

/// The server's last configuration packet: the client answers with its own
/// `finish_configuration`, and both move on to play. It has no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundFinishConfigurationPacket;

impl ClientboundFinishConfigurationPacket {
    pub const INSTANCE: ClientboundFinishConfigurationPacket = ClientboundFinishConfigurationPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        ClientboundFinishConfigurationPacket,
    > = &STREAM_CODEC;
}

impl<L: ClientConfigurationPacketListener + ?Sized> Packet<L>
    for ClientboundFinishConfigurationPacket
{
    fn r#type(&self) -> &'static PacketType {
        ConfigurationPacketTypes::CLIENTBOUND_FINISH_CONFIGURATION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_configuration_finished(self);
    }

    fn is_terminal(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ClientboundFinishConfigurationPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // No fields: [].
    impl Decode for ClientboundFinishConfigurationPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ClientboundFinishConfigurationPacket::INSTANCE
        }
    }

    impl Encode for ClientboundFinishConfigurationPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ClientConfigurationPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.configuration.ClientboundFinishConfigurationPacket",
        );
        golden.field("INSTANCE", ClientboundFinishConfigurationPacket::INSTANCE);
        golden.check_codec("STREAM_CODEC", || {
            ClientboundFinishConfigurationPacket::STREAM_CODEC
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundFinishConfigurationPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/configuration/ClientConfigurationPacketListener;)V",
            |this: &mut ClientboundFinishConfigurationPacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.check_method(
            "isTerminal()Z",
            |this: &mut ClientboundFinishConfigurationPacket| Packet::<Listener>::is_terminal(this),
        );
        golden.finish();
    }
}
