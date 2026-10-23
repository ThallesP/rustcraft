//! net/minecraft/network/protocol/ping/ServerboundPingRequestPacket.java

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::protocol::ping::ping_packet_types::PingPacketTypes;
use crate::net::minecraft::network::protocol::ping::server_ping_packet_listener::ServerPingPacketListener;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ServerboundPingRequestPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundPingRequestPacket::write,
            ServerboundPingRequestPacket::from_byte_buf,
        ))
    });

/// A client's ping: the server answers with a `ClientboundPongResponsePacket` of the same time,
/// the client's clock when it sent it.
#[derive(Clone, Debug)]
pub struct ServerboundPingRequestPacket {
    time: i64,
}

impl ServerboundPingRequestPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ServerboundPingRequestPacket> =
        &STREAM_CODEC;

    pub fn new(time: i64) -> ServerboundPingRequestPacket {
        ServerboundPingRequestPacket { time }
    }

    fn from_byte_buf(input: &mut ByteBuf) -> ServerboundPingRequestPacket {
        ServerboundPingRequestPacket::new(input.read_long())
    }

    fn write(&self, output: &mut ByteBuf) {
        output.write_long(self.time);
    }

    pub fn get_time(&self) -> i64 {
        self.time
    }
}

impl<L: ServerPingPacketListener + ?Sized> Packet<L> for ServerboundPingRequestPacket {
    fn r#type(&self) -> &'static PacketType {
        PingPacketTypes::SERVERBOUND_PING_REQUEST
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_ping_request(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundPingRequestPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::network::protocol::ping::server_ping_packet_listener::ServerPingPacketListener;

    // The field its public constructor sets.
    encode_fields!(ServerboundPingRequestPacket { time: i64 });

    type Listener = dyn ServerPingPacketListener;
    type This = ServerboundPingRequestPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.ping.ServerboundPingRequestPacket");
        golden.check_codec("STREAM_CODEC", || {
            ServerboundPingRequestPacket::STREAM_CODEC
        });
        golden.check("<init>(J)V", ServerboundPingRequestPacket::new);
        golden.check_method("getTime()J", |this: &mut This| this.get_time());
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut This| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/ping/ServerPingPacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
