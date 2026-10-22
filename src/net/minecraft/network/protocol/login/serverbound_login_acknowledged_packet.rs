//! net/minecraft/network/protocol/login/ServerboundLoginAcknowledgedPacket.java
//!
//! Java's constructor is private, so `INSTANCE` is the only one, and `unit` compares with it by
//! identity. Every value of a Rust unit struct is the same, so `PartialEq` is the same answer.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ServerboundLoginAcknowledgedPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::unit(
            ServerboundLoginAcknowledgedPacket::INSTANCE,
        ))
    });

/// The client's last login packet, after the server's `login_finished`: it moves on to the
/// configuration protocol. It has no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundLoginAcknowledgedPacket;

impl ServerboundLoginAcknowledgedPacket {
    pub const INSTANCE: ServerboundLoginAcknowledgedPacket = ServerboundLoginAcknowledgedPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        ServerboundLoginAcknowledgedPacket,
    > = &STREAM_CODEC;
}

impl<L: ServerLoginPacketListener + ?Sized> Packet<L> for ServerboundLoginAcknowledgedPacket {
    fn r#type(&self) -> &'static PacketType {
        LoginPacketTypes::SERVERBOUND_LOGIN_ACKNOWLEDGED
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_login_acknowledgement(self);
    }

    fn is_terminal(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ServerboundLoginAcknowledgedPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // No fields: [].
    impl Decode for ServerboundLoginAcknowledgedPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ServerboundLoginAcknowledgedPacket::INSTANCE
        }
    }

    impl Encode for ServerboundLoginAcknowledgedPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ServerLoginPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.login.ServerboundLoginAcknowledgedPacket");
        golden.field("INSTANCE", ServerboundLoginAcknowledgedPacket::INSTANCE);
        golden.check_codec("STREAM_CODEC", || {
            ServerboundLoginAcknowledgedPacket::STREAM_CODEC
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundLoginAcknowledgedPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/login/ServerLoginPacketListener;)V",
            |this: &mut ServerboundLoginAcknowledgedPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "isTerminal()Z",
            |this: &mut ServerboundLoginAcknowledgedPacket| Packet::<Listener>::is_terminal(this),
        );
        golden.finish();
    }
}
