//! net/minecraft/network/protocol/game/ServerboundConfigurationAcknowledgedPacket.java
//!
//! `unit` compares the value to write with `INSTANCE`, by identity in Java. Every value of a Rust
//! unit struct is the same, so `PartialEq` is the same answer.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ServerboundConfigurationAcknowledgedPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::unit(
            ServerboundConfigurationAcknowledgedPacket,
        ))
    });

/// The client's answer to `start_configuration`: it goes back to the configuration protocol. It has
/// no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundConfigurationAcknowledgedPacket;

impl ServerboundConfigurationAcknowledgedPacket {
    pub const INSTANCE: ServerboundConfigurationAcknowledgedPacket =
        ServerboundConfigurationAcknowledgedPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        ServerboundConfigurationAcknowledgedPacket,
    > = &STREAM_CODEC;
}

impl<L: ServerGamePacketListener + ?Sized> Packet<L>
    for ServerboundConfigurationAcknowledgedPacket
{
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::SERVERBOUND_CONFIGURATION_ACKNOWLEDGED
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_configuration_acknowledged(self);
    }

    fn is_terminal(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ServerboundConfigurationAcknowledgedPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // No fields: [].
    impl Decode for ServerboundConfigurationAcknowledgedPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ServerboundConfigurationAcknowledgedPacket
        }
    }

    impl Encode for ServerboundConfigurationAcknowledgedPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ServerGamePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ServerboundConfigurationAcknowledgedPacket",
        );
        golden.field(
            "INSTANCE",
            ServerboundConfigurationAcknowledgedPacket::INSTANCE,
        );
        golden.check_codec("STREAM_CODEC", || {
            ServerboundConfigurationAcknowledgedPacket::STREAM_CODEC
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundConfigurationAcknowledgedPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V",
            |this: &mut ServerboundConfigurationAcknowledgedPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "isTerminal()Z",
            |this: &mut ServerboundConfigurationAcknowledgedPacket| {
                Packet::<Listener>::is_terminal(this)
            },
        );
        golden.finish();
    }
}
