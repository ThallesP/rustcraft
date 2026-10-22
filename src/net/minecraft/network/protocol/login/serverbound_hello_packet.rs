//! net/minecraft/network/protocol/login/ServerboundHelloPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::java::lang::string::JavaString;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundHelloPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundHelloPacket::write,
            ServerboundHelloPacket::from_friendly_byte_buf,
        ))
    });

/// The client's first login packet: the player's name, at most 16 characters, and its profile's
/// id.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundHelloPacket {
    name: String,
    profile_id: UUID,
}

impl ServerboundHelloPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, ServerboundHelloPacket> =
        &STREAM_CODEC;

    pub fn new(name: String, profile_id: UUID) -> ServerboundHelloPacket {
        ServerboundHelloPacket { name, profile_id }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ServerboundHelloPacket {
        ServerboundHelloPacket::new(input.read_utf_max(16), input.read_uuid())
    }

    /// Panics for a name over 16 characters, where Java throws `EncoderException`.
    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_utf_max(&self.name, 16);
        output.write_uuid(&self.profile_id);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn profile_id(&self) -> UUID {
        self.profile_id
    }

    /// A record's: each component's hash, added to 31 times the ones before.
    pub fn hash_code(&self) -> i32 {
        self.name
            .hash_code()
            .wrapping_mul(31)
            .wrapping_add(self.profile_id.hash_code())
    }
}

impl<L: ServerLoginPacketListener + ?Sized> Packet<L> for ServerboundHelloPacket {
    fn r#type(&self) -> &'static PacketType {
        LoginPacketTypes::SERVERBOUND_HELLO
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_hello(self);
    }
}

/// A record's: `ServerboundHelloPacket[name=rustcraft, profileId=…]`.
impl fmt::Display for ServerboundHelloPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ServerboundHelloPacket[name={}, profileId={}]",
            self.name, self.profile_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundHelloPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::java::util::uuid::UUID;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ServerboundHelloPacket {
        name: String,
        profile_id: UUID,
    });

    type Listener = dyn ServerLoginPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.login.ServerboundHelloPacket");
        golden.check_codec("STREAM_CODEC", || ServerboundHelloPacket::STREAM_CODEC);
        golden.check(
            "<init>(Ljava/lang/String;Ljava/util/UUID;)V",
            ServerboundHelloPacket::new,
        );
        golden.check_method(
            "name()Ljava/lang/String;",
            |this: &mut ServerboundHelloPacket| this.name().to_owned(),
        );
        golden.check_method(
            "profileId()Ljava/util/UUID;",
            |this: &mut ServerboundHelloPacket| this.profile_id(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundHelloPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/login/ServerLoginPacketListener;)V",
            |this: &mut ServerboundHelloPacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundHelloPacket, other: ServerboundHelloPacket| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut ServerboundHelloPacket| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundHelloPacket| this.to_string(),
        );
        golden.finish();
    }
}
