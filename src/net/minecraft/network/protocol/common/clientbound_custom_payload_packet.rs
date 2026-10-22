//! net/minecraft/network/protocol/common/ClientboundCustomPayloadPacket.java
//!
//! Not ported yet: `GAMEPLAY_STREAM_CODEC`, play's, which reads a `RegistryFriendlyByteBuf`.

use std::fmt;
use std::sync::{Arc, LazyLock};

use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::common::custom::brand_payload::BrandPayload;
use crate::net::minecraft::network::protocol::common::custom::custom_packet_payload::{
    self, CustomPacketPayload, TypeAndCodec,
};
use crate::net::minecraft::network::protocol::common::custom::discarded_payload::DiscardedPayload;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static CONFIG_STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundCustomPayloadPacket> =
    LazyLock::new(|| {
        Box::new(
            custom_packet_payload::codec_with_fallback(
                |id| DiscardedPayload::codec(id, ClientboundCustomPayloadPacket::MAX_PAYLOAD_SIZE),
                vec![TypeAndCodec::new(
                    (*BrandPayload::TYPE).clone(),
                    BrandPayload::STREAM_CODEC,
                )],
            )
            .map(
                ClientboundCustomPayloadPacket::new,
                ClientboundCustomPayloadPacket::payload,
            ),
        )
    });

/// A payload the server sends on a channel, by its identifier: its brand, in configuration. The
/// client skips any payload it doesn't know, up to 1 MiB.
#[derive(Clone, Debug)]
pub struct ClientboundCustomPayloadPacket {
    payload: Arc<dyn CustomPacketPayload>,
}

impl ClientboundCustomPayloadPacket {
    const MAX_PAYLOAD_SIZE: i32 = 1048576;
    /// Configuration's: the brand, or any other payload discarded.
    pub const CONFIG_STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundCustomPayloadPacket,
    > = &CONFIG_STREAM_CODEC;

    pub fn new(payload: Arc<dyn CustomPacketPayload>) -> ClientboundCustomPayloadPacket {
        ClientboundCustomPayloadPacket { payload }
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
impl PartialEq for ClientboundCustomPayloadPacket {
    fn eq(&self, other: &ClientboundCustomPayloadPacket) -> bool {
        *self.payload == *other.payload
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundCustomPayloadPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_CUSTOM_PAYLOAD
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_custom_payload(self);
    }
}

/// A record's: `ClientboundCustomPayloadPacket[payload=BrandPayload[brand=vanilla]]`.
impl fmt::Display for ClientboundCustomPayloadPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundCustomPayloadPacket[payload={}]",
            self.payload
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::ClientboundCustomPayloadPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::common::custom::custom_packet_payload::CustomPacketPayload;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ClientboundCustomPayloadPacket {
        payload: Arc<dyn CustomPacketPayload>
    });

    type Listener = dyn ClientCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket");
        golden.pending(
            "GAMEPLAY_STREAM_CODEC",
            "play's codec reads a RegistryFriendlyByteBuf, which isn't ported",
        );
        golden.check_codec("CONFIG_STREAM_CODEC", || {
            ClientboundCustomPayloadPacket::CONFIG_STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload;)V",
            ClientboundCustomPayloadPacket::new,
        );
        golden.check_method(
            "payload()Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload;",
            |this: &mut ClientboundCustomPayloadPacket| this.payload(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundCustomPayloadPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut ClientboundCustomPayloadPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundCustomPayloadPacket, other: ClientboundCustomPayloadPacket| {
                *this == other
            },
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ClientboundCustomPayloadPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundCustomPayloadPacket| this.to_string(),
        );
        golden.finish();
    }
}
