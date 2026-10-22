//! net/minecraft/network/protocol/game/ClientboundStartConfigurationPacket.java
//!
//! `unit` compares the value to write with `INSTANCE`, by identity in Java. Every value of a Rust
//! unit struct is the same, so `PartialEq` is the same answer.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundStartConfigurationPacket> =
    LazyLock::new(|| Box::new(stream_codec::unit(ClientboundStartConfigurationPacket)));

/// The server taking the client back to the configuration protocol. It has no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundStartConfigurationPacket;

impl ClientboundStartConfigurationPacket {
    pub const INSTANCE: ClientboundStartConfigurationPacket = ClientboundStartConfigurationPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        ClientboundStartConfigurationPacket,
    > = &STREAM_CODEC;
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundStartConfigurationPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_START_CONFIGURATION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_configuration_start(self);
    }

    fn is_terminal(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ClientboundStartConfigurationPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // No fields: [].
    impl Decode for ClientboundStartConfigurationPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ClientboundStartConfigurationPacket
        }
    }

    impl Encode for ClientboundStartConfigurationPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ClientGamePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundStartConfigurationPacket");
        golden.field("INSTANCE", ClientboundStartConfigurationPacket::INSTANCE);
        golden.check_codec("STREAM_CODEC", || {
            ClientboundStartConfigurationPacket::STREAM_CODEC
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundStartConfigurationPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
            |this: &mut ClientboundStartConfigurationPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "isTerminal()Z",
            |this: &mut ClientboundStartConfigurationPacket| Packet::<Listener>::is_terminal(this),
        );
        golden.finish();
    }
}
