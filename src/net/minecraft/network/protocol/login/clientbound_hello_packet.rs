//! net/minecraft/network/protocol/login/ClientboundHelloPacket.java
//!
//! Not ported yet: `getPublicKey`, which decodes the key with `Crypt`, and comes with encryption
//! (online mode).

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundHelloPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundHelloPacket::write,
            ClientboundHelloPacket::from_friendly_byte_buf,
        ))
    });

/// The server's answer to the client's hello when it wants encryption: its id (empty for the
/// vanilla server), its public key, a challenge for the client to encrypt, and whether the
/// client has to authenticate with Mojang. An offline server never sends it.
#[derive(Clone, Debug)]
pub struct ClientboundHelloPacket {
    server_id: String,
    public_key: Vec<i8>,
    challenge: Vec<i8>,
    should_authenticate: bool,
}

impl ClientboundHelloPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, ClientboundHelloPacket> =
        &STREAM_CODEC;

    pub fn new(
        server_id: String,
        public_key: Vec<i8>,
        challenge: Vec<i8>,
        should_authenticate: bool,
    ) -> ClientboundHelloPacket {
        ClientboundHelloPacket {
            server_id,
            public_key,
            challenge,
            should_authenticate,
        }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundHelloPacket {
        ClientboundHelloPacket {
            server_id: input.read_utf_max(20),
            public_key: input.read_byte_array(),
            challenge: input.read_byte_array(),
            should_authenticate: input.read_boolean(),
        }
    }

    /// The id with `writeUtf`'s usual limit, which a reader's 20 can be under.
    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_utf(&self.server_id);
        output.write_byte_array(&self.public_key);
        output.write_byte_array(&self.challenge);
        output.write_boolean(self.should_authenticate);
    }

    pub fn get_server_id(&self) -> &str {
        &self.server_id
    }

    pub fn get_challenge(&self) -> &[i8] {
        &self.challenge
    }

    pub fn should_authenticate(&self) -> bool {
        self.should_authenticate
    }
}

impl<L: ClientLoginPacketListener + ?Sized> Packet<L> for ClientboundHelloPacket {
    fn r#type(&self) -> &'static PacketType {
        LoginPacketTypes::CLIENTBOUND_HELLO
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_hello(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundHelloPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // Its fields, in the order its constructor takes them.
    encode_fields!(ClientboundHelloPacket {
        server_id: String,
        public_key: Vec<i8>,
        challenge: Vec<i8>,
        should_authenticate: bool,
    });

    type Listener = dyn ClientLoginPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.login.ClientboundHelloPacket");
        golden.pending(
            "getPublicKey()Ljava/security/PublicKey;",
            "decodes the key with Crypt, which comes with encryption (online mode)",
        );
        golden.check_codec("STREAM_CODEC", || ClientboundHelloPacket::STREAM_CODEC);
        golden.check(
            "<init>(Ljava/lang/String;[B[BZ)V",
            ClientboundHelloPacket::new,
        );
        golden.check_method(
            "getServerId()Ljava/lang/String;",
            |this: &mut ClientboundHelloPacket| this.get_server_id().to_owned(),
        );
        golden.check_method("getChallenge()[B", |this: &mut ClientboundHelloPacket| {
            this.get_challenge().to_vec()
        });
        golden.check_method(
            "shouldAuthenticate()Z",
            |this: &mut ClientboundHelloPacket| this.should_authenticate(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundHelloPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/login/ClientLoginPacketListener;)V",
            |this: &mut ClientboundHelloPacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
