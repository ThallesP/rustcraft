//! net/minecraft/network/protocol/configuration/ServerboundFinishConfigurationPacket.java
//!
//! Java's constructor is private, so `INSTANCE` is the only one, and `unit` compares with it by
//! identity. Every value of a Rust unit struct is the same, so `PartialEq` is the same answer.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ServerboundFinishConfigurationPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::unit(
            ServerboundFinishConfigurationPacket::INSTANCE,
        ))
    });

/// The client's last configuration packet, after the server's `finish_configuration`: it moves
/// on to the play protocol. It has no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundFinishConfigurationPacket;

impl ServerboundFinishConfigurationPacket {
    pub const INSTANCE: ServerboundFinishConfigurationPacket = ServerboundFinishConfigurationPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        ServerboundFinishConfigurationPacket,
    > = &STREAM_CODEC;
}

impl<L: ServerConfigurationPacketListener + ?Sized> Packet<L>
    for ServerboundFinishConfigurationPacket
{
    fn r#type(&self) -> &'static PacketType {
        ConfigurationPacketTypes::SERVERBOUND_FINISH_CONFIGURATION
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

    use super::ServerboundFinishConfigurationPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // No fields: [].
    impl Decode for ServerboundFinishConfigurationPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ServerboundFinishConfigurationPacket::INSTANCE
        }
    }

    impl Encode for ServerboundFinishConfigurationPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ServerConfigurationPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket",
        );
        golden.field("INSTANCE", ServerboundFinishConfigurationPacket::INSTANCE);
        golden.check_codec("STREAM_CODEC", || {
            ServerboundFinishConfigurationPacket::STREAM_CODEC
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundFinishConfigurationPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/configuration/ServerConfigurationPacketListener;)V",
            |this: &mut ServerboundFinishConfigurationPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "isTerminal()Z",
            |this: &mut ServerboundFinishConfigurationPacket| Packet::<Listener>::is_terminal(this),
        );
        golden.finish();
    }
}
