//! net/minecraft/network/protocol/status/ServerboundStatusRequestPacket.java
//!
//! `unit` compares the value to write with `INSTANCE`, by identity in Java. Every value of a Rust
//! unit struct is the same, so `PartialEq` is the same answer.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::protocol::status::server_status_packet_listener::ServerStatusPacketListener;
use crate::net::minecraft::network::protocol::status::status_packet_types::StatusPacketTypes;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ServerboundStatusRequestPacket> =
    LazyLock::new(|| Box::new(stream_codec::unit(ServerboundStatusRequestPacket)));

/// The client asking a server for its status. It has no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundStatusRequestPacket;

impl ServerboundStatusRequestPacket {
    pub const INSTANCE: ServerboundStatusRequestPacket = ServerboundStatusRequestPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ServerboundStatusRequestPacket> =
        &STREAM_CODEC;
}

impl<L: ServerStatusPacketListener + ?Sized> Packet<L> for ServerboundStatusRequestPacket {
    fn r#type(&self) -> &'static PacketType {
        StatusPacketTypes::SERVERBOUND_STATUS_REQUEST
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_status_request(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ServerboundStatusRequestPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::network::protocol::status::server_status_packet_listener::ServerStatusPacketListener;

    // No fields: [].
    impl Decode for ServerboundStatusRequestPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ServerboundStatusRequestPacket
        }
    }

    impl Encode for ServerboundStatusRequestPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ServerStatusPacketListener;
    type This = ServerboundStatusRequestPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.status.ServerboundStatusRequestPacket");
        golden.field("INSTANCE", ServerboundStatusRequestPacket::INSTANCE);
        golden.check_codec("STREAM_CODEC", || {
            ServerboundStatusRequestPacket::STREAM_CODEC
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut This| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/status/ServerStatusPacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
