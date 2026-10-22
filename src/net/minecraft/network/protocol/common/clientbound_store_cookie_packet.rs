//! net/minecraft/network/protocol/common/ClientboundStoreCookiePacket.java
//!
//! Not ported: `hashCode` and `toString`, a record's, which hash and print the payload array's
//! identity hash, and so change from run to run. Nothing hashes or prints a packet.
//!
//! The payload is a shared array, like Java's reference to it, which `equals` compares by
//! reference: an `Arc`, so a clone of the packet shares it.

use std::sync::{Arc, LazyLock};

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundStoreCookiePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundStoreCookiePacket::write,
            ClientboundStoreCookiePacket::from_friendly_byte_buf,
        ))
    });
static PAYLOAD_STREAM_CODEC: StaticStreamCodec<ByteBuf, Vec<i8>> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::byte_array(
        ClientboundStoreCookiePacket::MAX_PAYLOAD_SIZE,
    ))
});

/// A server storing a cookie on the client under the key, which a later connection (after a
/// transfer, say) can ask for with a `ClientboundCookieRequestPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundStoreCookiePacket {
    key: Identifier,
    payload: Arc<Vec<i8>>,
}

impl ClientboundStoreCookiePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundStoreCookiePacket,
    > = &STREAM_CODEC;
    const MAX_PAYLOAD_SIZE: i32 = 5120;
    /// A cookie: at most 5120 bytes.
    pub const PAYLOAD_STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Vec<i8>> =
        &PAYLOAD_STREAM_CODEC;

    pub fn new(key: Identifier, payload: Arc<Vec<i8>>) -> ClientboundStoreCookiePacket {
        ClientboundStoreCookiePacket { key, payload }
    }

    /// A payload over 5120 bytes panics, where Java throws `DecoderException`.
    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundStoreCookiePacket {
        ClientboundStoreCookiePacket::new(
            input.read_identifier(),
            Arc::new(ClientboundStoreCookiePacket::PAYLOAD_STREAM_CODEC.decode(input)),
        )
    }

    /// A payload over 5120 bytes panics, where Java throws `EncoderException`.
    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_identifier(&self.key);
        ClientboundStoreCookiePacket::PAYLOAD_STREAM_CODEC.encode(output, &self.payload);
    }

    pub fn key(&self) -> &Identifier {
        &self.key
    }

    pub fn payload(&self) -> &[i8] {
        &self.payload
    }
}

/// A record's: the key, and the payload by reference, so two payloads are only equal when
/// they're the same array.
impl PartialEq for ClientboundStoreCookiePacket {
    fn eq(&self, other: &ClientboundStoreCookiePacket) -> bool {
        self.key == other.key && Arc::ptr_eq(&self.payload, &other.payload)
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundStoreCookiePacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_STORE_COOKIE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_store_cookie(self);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::ClientboundStoreCookiePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components. Each argument gets its own payload, like vanilla's.
    encode_fields!(ClientboundStoreCookiePacket {
        key: Identifier,
        payload: Arc<Vec<i8>>,
    });

    type Listener = dyn ClientCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ClientboundStoreCookiePacket");
        golden.pending(
            "hashCode()I",
            "hashes the payload array's identity hash, which changes from run to run",
        );
        golden.pending(
            "toString()Ljava/lang/String;",
            "prints the payload array's identity hash, which changes from run to run",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundStoreCookiePacket::STREAM_CODEC
        });
        golden.check_codec("PAYLOAD_STREAM_CODEC", || {
            ClientboundStoreCookiePacket::PAYLOAD_STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/resources/Identifier;[B)V",
            ClientboundStoreCookiePacket::new,
        );
        golden.check_method(
            "key()Lnet/minecraft/resources/Identifier;",
            |this: &mut ClientboundStoreCookiePacket| this.key().clone(),
        );
        golden.check_method("payload()[B", |this: &mut ClientboundStoreCookiePacket| {
            this.payload().to_vec()
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundStoreCookiePacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut ClientboundStoreCookiePacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundStoreCookiePacket, other: ClientboundStoreCookiePacket| {
                *this == other
            },
        );
        golden.finish();
    }
}
