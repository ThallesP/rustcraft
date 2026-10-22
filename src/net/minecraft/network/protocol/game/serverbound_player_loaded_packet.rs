//! net/minecraft/network/protocol/game/ServerboundPlayerLoadedPacket.java
//!
//! Java's `unit` codec holds one instance, made where the codec is, and compares the value to
//! write with it by identity. Every value of a Rust unit struct is the same, so `PartialEq` is the
//! same answer.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ServerboundPlayerLoadedPacket> =
    LazyLock::new(|| Box::new(stream_codec::unit(ServerboundPlayerLoadedPacket)));

/// The client has loaded the world around the player, which the server waits for before it moves or hurts the player. It has no
/// fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundPlayerLoadedPacket;

impl ServerboundPlayerLoadedPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ServerboundPlayerLoadedPacket> =
        &STREAM_CODEC;

    /// A record's with no components: 0.
    pub fn hash_code(&self) -> i32 {
        0
    }
}

/// A record's: `ServerboundPlayerLoadedPacket[]`.
impl std::fmt::Display for ServerboundPlayerLoadedPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerboundPlayerLoadedPacket[]")
    }
}

impl<L: ServerGamePacketListener + ?Sized> Packet<L> for ServerboundPlayerLoadedPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::SERVERBOUND_PLAYER_LOADED
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_accept_player_load(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ServerboundPlayerLoadedPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // No fields: [].
    impl Decode for ServerboundPlayerLoadedPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ServerboundPlayerLoadedPacket
        }
    }

    impl Encode for ServerboundPlayerLoadedPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ServerGamePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket");
        golden.check("<init>()V", || ServerboundPlayerLoadedPacket);
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundPlayerLoadedPacket, other: ServerboundPlayerLoadedPacket| {
                *this == other
            },
        );
        golden.check_method("hashCode()I", |this: &mut ServerboundPlayerLoadedPacket| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundPlayerLoadedPacket| this.to_string(),
        );
        golden.check_codec("STREAM_CODEC", || {
            ServerboundPlayerLoadedPacket::STREAM_CODEC
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundPlayerLoadedPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V",
            |this: &mut ServerboundPlayerLoadedPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.finish();
    }
}
