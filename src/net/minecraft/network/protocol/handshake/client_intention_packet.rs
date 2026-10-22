//! net/minecraft/network/protocol/handshake/ClientIntentionPacket.java
//!
//! Not ported: `hashCode`, a record's, which hashes `ClientIntent`'s identity hash and so changes
//! from run to run. Nothing hashes a packet.

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::handshake::client_intent::ClientIntent;
use crate::net::minecraft::network::protocol::handshake::handshake_packet_types::HandshakePacketTypes;
use crate::net::minecraft::network::protocol::handshake::server_handshake_packet_listener::ServerHandshakePacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientIntentionPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientIntentionPacket::write,
            ClientIntentionPacket::from_friendly_byte_buf,
        ))
    });

/// The handshake, the first packet a client sends: its protocol version, the address and port it
/// connected to, and what it connects for. The port is an int, written as a short.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientIntentionPacket {
    protocol_version: i32,
    host_name: String,
    port: i32,
    intention: ClientIntent,
}

impl ClientIntentionPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, ClientIntentionPacket> =
        &STREAM_CODEC;
    const MAX_HOST_LENGTH: i32 = 255;

    pub fn new(
        protocol_version: i32,
        host_name: String,
        port: i32,
        intention: ClientIntent,
    ) -> ClientIntentionPacket {
        ClientIntentionPacket {
            protocol_version,
            host_name,
            port,
            intention,
        }
    }

    /// The constructor reading a buffer. Panics on an unknown intent, where Java throws.
    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientIntentionPacket {
        ClientIntentionPacket::new(
            input.read_var_int(),
            input.read_utf_max(ClientIntentionPacket::MAX_HOST_LENGTH),
            input.read_unsigned_short(),
            ClientIntent::by_id(input.read_var_int()),
        )
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.protocol_version);
        output.write_utf(&self.host_name);
        output.write_short(self.port);
        output.write_var_int(self.intention.id());
    }

    pub fn protocol_version(&self) -> i32 {
        self.protocol_version
    }

    pub fn host_name(&self) -> &str {
        &self.host_name
    }

    pub fn port(&self) -> i32 {
        self.port
    }

    pub fn intention(&self) -> ClientIntent {
        self.intention
    }
}

impl<L: ServerHandshakePacketListener + ?Sized> Packet<L> for ClientIntentionPacket {
    fn r#type(&self) -> &'static PacketType {
        HandshakePacketTypes::CLIENT_INTENTION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_intention(self);
    }

    fn is_terminal(&self) -> bool {
        true
    }
}

/// A record's: `ClientIntentionPacket[protocolVersion=777, hostName=localhost, …]`.
impl fmt::Display for ClientIntentionPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientIntentionPacket[protocolVersion={}, hostName={}, port={}, intention={}]",
            self.protocol_version, self.host_name, self.port, self.intention
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientIntentionPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::handshake::client_intent::ClientIntent;
    use crate::net::minecraft::network::protocol::handshake::server_handshake_packet_listener::ServerHandshakePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ClientIntentionPacket {
        protocol_version: i32,
        host_name: String,
        port: i32,
        intention: ClientIntent,
    });

    type Listener = dyn ServerHandshakePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.handshake.ClientIntentionPacket");
        golden.pending(
            "hashCode()I",
            "hashes ClientIntent's identity hash, which changes from run to run",
        );
        golden.check_codec("STREAM_CODEC", || ClientIntentionPacket::STREAM_CODEC);
        golden.check(
            "<init>(ILjava/lang/String;ILnet/minecraft/network/protocol/handshake/ClientIntent;)V",
            ClientIntentionPacket::new,
        );
        golden.check_method("protocolVersion()I", |this: &mut ClientIntentionPacket| {
            this.protocol_version()
        });
        golden.check_method(
            "hostName()Ljava/lang/String;",
            |this: &mut ClientIntentionPacket| this.host_name().to_owned(),
        );
        golden.check_method("port()I", |this: &mut ClientIntentionPacket| this.port());
        golden.check_method(
            "intention()Lnet/minecraft/network/protocol/handshake/ClientIntent;",
            |this: &mut ClientIntentionPacket| this.intention(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientIntentionPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/handshake/ServerHandshakePacketListener;)V",
            |this: &mut ClientIntentionPacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.check_method("isTerminal()Z", |this: &mut ClientIntentionPacket| {
            Packet::<Listener>::is_terminal(this)
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientIntentionPacket, other: ClientIntentionPacket| *this == other,
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientIntentionPacket| this.to_string(),
        );
        golden.finish();
    }
}
