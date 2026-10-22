//! net/minecraft/network/protocol/common/ServerboundCustomPayloadPacket.java

use std::fmt;
use std::sync::{Arc, LazyLock};

use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::common::custom::brand_payload::BrandPayload;
use crate::net::minecraft::network::protocol::common::custom::custom_packet_payload::{
    self, CustomPacketPayload, TypeAndCodec,
};
use crate::net::minecraft::network::protocol::common::custom::discarded_payload::DiscardedPayload;
use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundCustomPayloadPacket> =
    LazyLock::new(|| {
        Box::new(
            custom_packet_payload::codec_with_fallback(
                |id| DiscardedPayload::codec(id, ServerboundCustomPayloadPacket::MAX_PAYLOAD_SIZE),
                vec![TypeAndCodec::new(
                    (*BrandPayload::TYPE).clone(),
                    BrandPayload::STREAM_CODEC,
                )],
            )
            .map(
                ServerboundCustomPayloadPacket::new,
                ServerboundCustomPayloadPacket::payload,
            ),
        )
    });

/// A payload the client sends on a channel, by its identifier: its brand after login, and the
/// scenario's start marker. The server skips any payload it doesn't know, up to 32767 bytes.
#[derive(Clone, Debug)]
pub struct ServerboundCustomPayloadPacket {
    payload: Arc<dyn CustomPacketPayload>,
}

impl ServerboundCustomPayloadPacket {
    const MAX_PAYLOAD_SIZE: i32 = 32767;
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundCustomPayloadPacket,
    > = &STREAM_CODEC;

    pub fn new(payload: Arc<dyn CustomPacketPayload>) -> ServerboundCustomPayloadPacket {
        ServerboundCustomPayloadPacket { payload }
    }

    pub fn payload(&self) -> Arc<dyn CustomPacketPayload> {
        Arc::clone(&self.payload)
    }

    /// A record's of one component: its hash.
    pub fn hash_code(&self) -> i32 {
        self.payload.hash_code()
    }
}

/// A record's: the payloads' `equals`.
impl PartialEq for ServerboundCustomPayloadPacket {
    fn eq(&self, other: &ServerboundCustomPayloadPacket) -> bool {
        *self.payload == *other.payload
    }
}

impl<L: ServerCommonPacketListener + ?Sized> Packet<L> for ServerboundCustomPayloadPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::SERVERBOUND_CUSTOM_PAYLOAD
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_custom_payload(self);
    }
}

/// A record's: `ServerboundCustomPayloadPacket[payload=BrandPayload[brand=vanilla]]`.
impl fmt::Display for ServerboundCustomPayloadPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ServerboundCustomPayloadPacket[payload={}]",
            self.payload
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::ServerboundCustomPayloadPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::custom::custom_packet_payload::CustomPacketPayload;
    use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ServerboundCustomPayloadPacket {
        payload: Arc<dyn CustomPacketPayload>
    });

    type Listener = dyn ServerCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket");
        golden.check_codec("STREAM_CODEC", || {
            ServerboundCustomPayloadPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload;)V",
            ServerboundCustomPayloadPacket::new,
        );
        golden.check_method(
            "payload()Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload;",
            |this: &mut ServerboundCustomPayloadPacket| this.payload(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundCustomPayloadPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ServerCommonPacketListener;)V",
            |this: &mut ServerboundCustomPayloadPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundCustomPayloadPacket, other: ServerboundCustomPayloadPacket| {
                *this == other
            },
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ServerboundCustomPayloadPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundCustomPayloadPacket| this.to_string(),
        );
        golden.finish();
    }
}
