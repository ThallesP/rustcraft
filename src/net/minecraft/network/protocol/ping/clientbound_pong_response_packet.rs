//! net/minecraft/network/protocol/ping/ClientboundPongResponsePacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::protocol::ping::client_pong_packet_listener::ClientPongPacketListener;
use crate::net::minecraft::network::protocol::ping::ping_packet_types::PingPacketTypes;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundPongResponsePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundPongResponsePacket::write,
            ClientboundPongResponsePacket::from_friendly_byte_buf,
        ))
    });

/// The server's answer to a ping request: the time the request carried.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundPongResponsePacket {
    time: i64,
}

impl ClientboundPongResponsePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundPongResponsePacket,
    > = &STREAM_CODEC;

    pub fn new(time: i64) -> ClientboundPongResponsePacket {
        ClientboundPongResponsePacket { time }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundPongResponsePacket {
        ClientboundPongResponsePacket::new(input.read_long())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_long(self.time);
    }

    pub fn time(&self) -> i64 {
        self.time
    }

    /// A record's: its one component's hash, `Long.hashCode`.
    pub fn hash_code(&self) -> i32 {
        (self.time ^ ((self.time as u64) >> 32) as i64) as i32
    }
}

impl<L: ClientPongPacketListener + ?Sized> Packet<L> for ClientboundPongResponsePacket {
    fn r#type(&self) -> &'static PacketType {
        PingPacketTypes::CLIENTBOUND_PONG_RESPONSE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_pong_response(self);
    }
}

/// A record's: `ClientboundPongResponsePacket[time=42]`.
impl fmt::Display for ClientboundPongResponsePacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientboundPongResponsePacket[time={}]", self.time)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundPongResponsePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::network::protocol::ping::client_pong_packet_listener::ClientPongPacketListener;

    encode_fields!(ClientboundPongResponsePacket { time: i64 });

    type Listener = dyn ClientPongPacketListener;
    type This = ClientboundPongResponsePacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.ping.ClientboundPongResponsePacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundPongResponsePacket::STREAM_CODEC
        });
        golden.check("<init>(J)V", ClientboundPongResponsePacket::new);
        golden.check_method("time()J", |this: &mut This| this.time());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut This| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut This| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/ping/ClientPongPacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
