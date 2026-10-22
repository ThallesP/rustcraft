//! net/minecraft/network/protocol/common/ServerboundKeepAlivePacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundKeepAlivePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundKeepAlivePacket::write,
            ServerboundKeepAlivePacket::from_friendly_byte_buf,
        ))
    });

/// The client's answer to a server's keep-alive: the id the server sent.
#[derive(Clone, Debug)]
pub struct ServerboundKeepAlivePacket {
    id: i64,
}

impl ServerboundKeepAlivePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundKeepAlivePacket,
    > = &STREAM_CODEC;

    pub fn new(id: i64) -> ServerboundKeepAlivePacket {
        ServerboundKeepAlivePacket { id }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ServerboundKeepAlivePacket {
        ServerboundKeepAlivePacket::new(input.read_long())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_long(self.id);
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }
}

impl<L: ServerCommonPacketListener + ?Sized> Packet<L> for ServerboundKeepAlivePacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::SERVERBOUND_KEEP_ALIVE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_keep_alive(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundKeepAlivePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The field its public constructor sets.
    encode_fields!(ServerboundKeepAlivePacket { id: i64 });

    type Listener = dyn ServerCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ServerboundKeepAlivePacket");
        golden.check_codec("STREAM_CODEC", || ServerboundKeepAlivePacket::STREAM_CODEC);
        golden.check("<init>(J)V", ServerboundKeepAlivePacket::new);
        golden.check_method("getId()J", |this: &mut ServerboundKeepAlivePacket| {
            this.get_id()
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundKeepAlivePacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ServerCommonPacketListener;)V",
            |this: &mut ServerboundKeepAlivePacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
