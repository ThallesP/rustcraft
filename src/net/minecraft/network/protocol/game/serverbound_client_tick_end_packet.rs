//! net/minecraft/network/protocol/game/ServerboundClientTickEndPacket.java
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

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ServerboundClientTickEndPacket> =
    LazyLock::new(|| Box::new(stream_codec::unit(ServerboundClientTickEndPacket)));

/// The client's last packet of each of its ticks. It has no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundClientTickEndPacket;

impl ServerboundClientTickEndPacket {
    pub const INSTANCE: ServerboundClientTickEndPacket = ServerboundClientTickEndPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ServerboundClientTickEndPacket> =
        &STREAM_CODEC;

    /// A record's with no components: 0.
    pub fn hash_code(&self) -> i32 {
        0
    }
}

/// A record's: `ServerboundClientTickEndPacket[]`.
impl std::fmt::Display for ServerboundClientTickEndPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerboundClientTickEndPacket[]")
    }
}

impl<L: ServerGamePacketListener + ?Sized> Packet<L> for ServerboundClientTickEndPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::SERVERBOUND_CLIENT_TICK_END
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_client_tick_end(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ServerboundClientTickEndPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // No fields: [].
    impl Decode for ServerboundClientTickEndPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ServerboundClientTickEndPacket
        }
    }

    impl Encode for ServerboundClientTickEndPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ServerGamePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ServerboundClientTickEndPacket");
        golden.field("INSTANCE", ServerboundClientTickEndPacket::INSTANCE);
        golden.check("<init>()V", || ServerboundClientTickEndPacket);
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundClientTickEndPacket, other: ServerboundClientTickEndPacket| {
                *this == other
            },
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ServerboundClientTickEndPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundClientTickEndPacket| this.to_string(),
        );
        golden.check_codec("STREAM_CODEC", || {
            ServerboundClientTickEndPacket::STREAM_CODEC
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundClientTickEndPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V",
            |this: &mut ServerboundClientTickEndPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.finish();
    }
}
