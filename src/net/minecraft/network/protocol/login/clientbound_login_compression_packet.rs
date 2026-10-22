//! net/minecraft/network/protocol/login/ClientboundLoginCompressionPacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundLoginCompressionPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundLoginCompressionPacket::write,
            ClientboundLoginCompressionPacket::from_friendly_byte_buf,
        ))
    });

/// Turns on compression for the packets after it: the size from which a packet is compressed.
#[derive(Clone, Debug)]
pub struct ClientboundLoginCompressionPacket {
    compression_threshold: i32,
}

impl ClientboundLoginCompressionPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundLoginCompressionPacket,
    > = &STREAM_CODEC;

    pub fn new(compression_threshold: i32) -> ClientboundLoginCompressionPacket {
        ClientboundLoginCompressionPacket {
            compression_threshold,
        }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundLoginCompressionPacket {
        ClientboundLoginCompressionPacket {
            compression_threshold: input.read_var_int(),
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.compression_threshold);
    }

    pub fn get_compression_threshold(&self) -> i32 {
        self.compression_threshold
    }
}

impl<L: ClientLoginPacketListener + ?Sized> Packet<L> for ClientboundLoginCompressionPacket {
    fn r#type(&self) -> &'static PacketType {
        LoginPacketTypes::CLIENTBOUND_LOGIN_COMPRESSION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_compression(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundLoginCompressionPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundLoginCompressionPacket {
        compression_threshold: i32
    });

    type Listener = dyn ClientLoginPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundLoginCompressionPacket::STREAM_CODEC
        });
        golden.check("<init>(I)V", ClientboundLoginCompressionPacket::new);
        golden.check_method(
            "getCompressionThreshold()I",
            |this: &mut ClientboundLoginCompressionPacket| this.get_compression_threshold(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundLoginCompressionPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/login/ClientLoginPacketListener;)V",
            |this: &mut ClientboundLoginCompressionPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.finish();
    }
}
