//! net/minecraft/network/protocol/cookie/ServerboundCookieResponsePacket.java
//!
//! Not ported: `hashCode` and `toString`, a record's, which hash and print the payload array's
//! identity hash, and so change from run to run. Nothing hashes or prints a packet.
//!
//! The payload is a shared array, like Java's reference to it, which `equals` compares by
//! reference: an `Arc`, so a clone of the packet shares it.

use std::sync::{Arc, LazyLock};

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::clientbound_store_cookie_packet::ClientboundStoreCookiePacket;
use crate::net::minecraft::network::protocol::cookie::cookie_packet_types::CookiePacketTypes;
use crate::net::minecraft::network::protocol::cookie::server_cookie_packet_listener::ServerCookiePacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundCookieResponsePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundCookieResponsePacket::write,
            ServerboundCookieResponsePacket::from_friendly_byte_buf,
        ))
    });

/// The client's answer to a cookie request: the key, and the cookie stored under it, or none.
#[derive(Clone, Debug)]
pub struct ServerboundCookieResponsePacket {
    key: Identifier,
    payload: Option<Arc<Vec<i8>>>,
}

impl ServerboundCookieResponsePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundCookieResponsePacket,
    > = &STREAM_CODEC;

    pub fn new(key: Identifier, payload: Option<Arc<Vec<i8>>>) -> ServerboundCookieResponsePacket {
        ServerboundCookieResponsePacket { key, payload }
    }

    /// A payload over 5120 bytes panics, where Java throws `DecoderException`.
    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ServerboundCookieResponsePacket {
        ServerboundCookieResponsePacket::new(
            input.read_identifier(),
            input
                .read_nullable(|input: &mut ByteBuf| {
                    ClientboundStoreCookiePacket::PAYLOAD_STREAM_CODEC.decode(input)
                })
                .map(Arc::new),
        )
    }

    /// A payload over 5120 bytes panics, where Java throws `EncoderException`.
    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_identifier(&self.key);
        output.write_nullable(
            self.payload.as_deref(),
            |output: &mut ByteBuf, payload: &Vec<i8>| {
                ClientboundStoreCookiePacket::PAYLOAD_STREAM_CODEC.encode(output, payload)
            },
        );
    }

    pub fn key(&self) -> &Identifier {
        &self.key
    }

    pub fn payload(&self) -> Option<&[i8]> {
        self.payload.as_deref().map(Vec::as_slice)
    }
}

/// A record's: the key, and the payload by reference, so two payloads are only equal when
/// they're the same array (or both none).
impl PartialEq for ServerboundCookieResponsePacket {
    fn eq(&self, other: &ServerboundCookieResponsePacket) -> bool {
        self.key == other.key
            && match (&self.payload, &other.payload) {
                (Some(payload), Some(other)) => Arc::ptr_eq(payload, other),
                (payload, other) => payload.is_none() && other.is_none(),
            }
    }
}

impl<L: ServerCookiePacketListener + ?Sized> Packet<L> for ServerboundCookieResponsePacket {
    fn r#type(&self) -> &'static PacketType {
        CookiePacketTypes::SERVERBOUND_COOKIE_RESPONSE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_cookie_response(self);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::ServerboundCookieResponsePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::cookie::server_cookie_packet_listener::ServerCookiePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components. Each argument gets its own payload, like vanilla's.
    encode_fields!(ServerboundCookieResponsePacket {
        key: Identifier,
        payload: Option<Arc<Vec<i8>>>,
    });

    type Listener = dyn ServerCookiePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.cookie.ServerboundCookieResponsePacket");
        golden.pending(
            "hashCode()I",
            "hashes the payload array's identity hash, which changes from run to run",
        );
        golden.pending(
            "toString()Ljava/lang/String;",
            "prints the payload array's identity hash, which changes from run to run",
        );
        golden.check_codec("STREAM_CODEC", || {
            ServerboundCookieResponsePacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/resources/Identifier;[B)V",
            ServerboundCookieResponsePacket::new,
        );
        golden.check_method(
            "key()Lnet/minecraft/resources/Identifier;",
            |this: &mut ServerboundCookieResponsePacket| this.key().clone(),
        );
        golden.check_method(
            "payload()[B",
            |this: &mut ServerboundCookieResponsePacket| this.payload().map(<[i8]>::to_vec),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundCookieResponsePacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/cookie/ServerCookiePacketListener;)V",
            |this: &mut ServerboundCookieResponsePacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundCookieResponsePacket, other: ServerboundCookieResponsePacket| {
                *this == other
            },
        );
        golden.finish();
    }
}
