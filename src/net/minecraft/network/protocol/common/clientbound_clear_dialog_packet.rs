//! net/minecraft/network/protocol/common/ClientboundClearDialogPacket.java
//!
//! Java's constructor is private, so `INSTANCE` is the only one, and `unit` compares with it by
//! identity. Every value of a Rust unit struct is the same, so `PartialEq` is the same answer.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundClearDialogPacket> =
    LazyLock::new(|| Box::new(stream_codec::unit(ClientboundClearDialogPacket::INSTANCE)));

/// The server closing the dialog it showed the client, if one is open. It has no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundClearDialogPacket;

impl ClientboundClearDialogPacket {
    pub const INSTANCE: ClientboundClearDialogPacket = ClientboundClearDialogPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundClearDialogPacket> =
        &STREAM_CODEC;
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundClearDialogPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_CLEAR_DIALOG
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_clear_dialog(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ClientboundClearDialogPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // No fields: [].
    impl Decode for ClientboundClearDialogPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ClientboundClearDialogPacket::INSTANCE
        }
    }

    impl Encode for ClientboundClearDialogPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ClientCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ClientboundClearDialogPacket");
        golden.field("INSTANCE", ClientboundClearDialogPacket::INSTANCE);
        golden.check_codec("STREAM_CODEC", || {
            ClientboundClearDialogPacket::STREAM_CODEC
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundClearDialogPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut ClientboundClearDialogPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.finish();
    }
}
