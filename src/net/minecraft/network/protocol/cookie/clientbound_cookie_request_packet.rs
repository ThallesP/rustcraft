//! net/minecraft/network/protocol/cookie/ClientboundCookieRequestPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener;
use crate::net::minecraft::network::protocol::cookie::cookie_packet_types::CookiePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundCookieRequestPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundCookieRequestPacket::write,
            ClientboundCookieRequestPacket::from_friendly_byte_buf,
        ))
    });

/// A server asking for the cookie it stored under the key, which the client answers with a
/// `ServerboundCookieResponsePacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundCookieRequestPacket {
    key: Identifier,
}

impl ClientboundCookieRequestPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundCookieRequestPacket,
    > = &STREAM_CODEC;

    pub fn new(key: Identifier) -> ClientboundCookieRequestPacket {
        ClientboundCookieRequestPacket { key }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundCookieRequestPacket {
        ClientboundCookieRequestPacket::new(input.read_identifier())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_identifier(&self.key);
    }

    pub fn key(&self) -> &Identifier {
        &self.key
    }

    /// A record's of one component: its hash.
    pub fn hash_code(&self) -> i32 {
        self.key.hash_code()
    }
}

impl<L: ClientCookiePacketListener + ?Sized> Packet<L> for ClientboundCookieRequestPacket {
    fn r#type(&self) -> &'static PacketType {
        CookiePacketTypes::CLIENTBOUND_COOKIE_REQUEST
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_request_cookie(self);
    }
}

/// A record's: `ClientboundCookieRequestPacket[key=minecraft:stone]`.
impl fmt::Display for ClientboundCookieRequestPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientboundCookieRequestPacket[key={}]", self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundCookieRequestPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components.
    encode_fields!(ClientboundCookieRequestPacket { key: Identifier });

    type Listener = dyn ClientCookiePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.cookie.ClientboundCookieRequestPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundCookieRequestPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/resources/Identifier;)V",
            ClientboundCookieRequestPacket::new,
        );
        golden.check_method(
            "key()Lnet/minecraft/resources/Identifier;",
            |this: &mut ClientboundCookieRequestPacket| this.key().clone(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundCookieRequestPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/cookie/ClientCookiePacketListener;)V",
            |this: &mut ClientboundCookieRequestPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundCookieRequestPacket, other: ClientboundCookieRequestPacket| {
                *this == other
            },
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ClientboundCookieRequestPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundCookieRequestPacket| this.to_string(),
        );
        golden.finish();
    }
}
