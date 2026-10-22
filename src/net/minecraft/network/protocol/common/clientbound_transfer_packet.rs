//! net/minecraft/network/protocol/common/ClientboundTransferPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::java::lang::object::HashCode;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundTransferPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundTransferPacket::write,
            ClientboundTransferPacket::from_friendly_byte_buf,
        ))
    });

/// A server sending the client to another server, which it connects to with a transfer intent.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundTransferPacket {
    host: String,
    port: i32,
}

impl ClientboundTransferPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, ClientboundTransferPacket> =
        &STREAM_CODEC;

    pub fn new(host: String, port: i32) -> ClientboundTransferPacket {
        ClientboundTransferPacket { host, port }
    }

    /// A host of up to 32767 characters, and panics past them, where Java throws
    /// `DecoderException`. The port is a VarInt, any int.
    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundTransferPacket {
        ClientboundTransferPacket::new(input.read_utf(), input.read_var_int())
    }

    /// Panics for a host past 32767 characters, where Java throws `EncoderException`.
    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_utf(&self.host);
        output.write_var_int(self.port);
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> i32 {
        self.port
    }

    /// A record's: each component's hash, added to 31 times the ones before.
    pub fn hash_code(&self) -> i32 {
        self.host
            .hash_code()
            .wrapping_mul(31)
            .wrapping_add(self.port)
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundTransferPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_TRANSFER
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_transfer(self);
    }
}

/// A record's: `ClientboundTransferPacket[host=localhost, port=25565]`.
impl fmt::Display for ClientboundTransferPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundTransferPacket[host={}, port={}]",
            self.host, self.port
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundTransferPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ClientboundTransferPacket {
        host: String,
        port: i32,
    });

    type Listener = dyn ClientCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ClientboundTransferPacket");
        golden.check_codec("STREAM_CODEC", || ClientboundTransferPacket::STREAM_CODEC);
        golden.check(
            "<init>(Ljava/lang/String;I)V",
            ClientboundTransferPacket::new,
        );
        golden.check_method(
            "host()Ljava/lang/String;",
            |this: &mut ClientboundTransferPacket| this.host().to_owned(),
        );
        golden.check_method("port()I", |this: &mut ClientboundTransferPacket| {
            this.port()
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundTransferPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut ClientboundTransferPacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundTransferPacket, other: ClientboundTransferPacket| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut ClientboundTransferPacket| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundTransferPacket| this.to_string(),
        );
        golden.finish();
    }
}
