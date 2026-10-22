//! net/minecraft/network/protocol/login/ServerboundKeyPacket.java
//!
//! Not ported yet: the public constructor, `getSecretKey` and `isChallengeValid`, which encrypt and
//! decrypt with `Crypt`, and come with encryption (online mode). An offline client never sends
//! it.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundKeyPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundKeyPacket::write,
            ServerboundKeyPacket::from_friendly_byte_buf,
        ))
    });

/// The client's answer to the server's hello when it wants encryption: the shared secret key and
/// the server's challenge, each encrypted with the server's public key.
#[derive(Clone, Debug)]
pub struct ServerboundKeyPacket {
    keybytes: Vec<i8>,
    encrypted_challenge: Vec<i8>,
}

impl ServerboundKeyPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, ServerboundKeyPacket> =
        &STREAM_CODEC;

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ServerboundKeyPacket {
        ServerboundKeyPacket {
            keybytes: input.read_byte_array(),
            encrypted_challenge: input.read_byte_array(),
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_byte_array(&self.keybytes);
        output.write_byte_array(&self.encrypted_challenge);
    }
}

impl<L: ServerLoginPacketListener + ?Sized> Packet<L> for ServerboundKeyPacket {
    fn r#type(&self) -> &'static PacketType {
        LoginPacketTypes::SERVERBOUND_KEY
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_key(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundKeyPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // Its fields, which vanilla's cases set by reading them from a buffer.
    encode_fields!(ServerboundKeyPacket {
        keybytes: Vec<i8>,
        encrypted_challenge: Vec<i8>,
    });

    type Listener = dyn ServerLoginPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.login.ServerboundKeyPacket");
        golden.pending(
            "<init>(Ljavax/crypto/SecretKey;Ljava/security/PublicKey;[B)V",
            "encrypts with Crypt, which comes with encryption (online mode)",
        );
        golden.pending(
            "getSecretKey(Ljava/security/PrivateKey;)Ljavax/crypto/SecretKey;",
            "decrypts with Crypt, which comes with encryption (online mode)",
        );
        golden.pending(
            "isChallengeValid([BLjava/security/PrivateKey;)Z",
            "decrypts with Crypt, which comes with encryption (online mode)",
        );
        golden.check_codec("STREAM_CODEC", || ServerboundKeyPacket::STREAM_CODEC);
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundKeyPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/login/ServerLoginPacketListener;)V",
            |this: &mut ServerboundKeyPacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
