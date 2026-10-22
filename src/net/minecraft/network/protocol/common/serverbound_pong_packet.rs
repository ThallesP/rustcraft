//! net/minecraft/network/protocol/common/ServerboundPongPacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundPongPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundPongPacket::write,
            ServerboundPongPacket::from_friendly_byte_buf,
        ))
    });

/// The client's answer to a server's ping: the id the server sent.
#[derive(Clone, Debug)]
pub struct ServerboundPongPacket {
    id: i32,
}

impl ServerboundPongPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, ServerboundPongPacket> =
        &STREAM_CODEC;

    pub fn new(id: i32) -> ServerboundPongPacket {
        ServerboundPongPacket { id }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ServerboundPongPacket {
        ServerboundPongPacket::new(input.read_int())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_int(self.id);
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }
}

impl<L: ServerCommonPacketListener + ?Sized> Packet<L> for ServerboundPongPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::SERVERBOUND_PONG
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_pong(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundPongPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The field its public constructor sets.
    encode_fields!(ServerboundPongPacket { id: i32 });

    type Listener = dyn ServerCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.common.ServerboundPongPacket");
        golden.check_codec("STREAM_CODEC", || ServerboundPongPacket::STREAM_CODEC);
        golden.check("<init>(I)V", ServerboundPongPacket::new);
        golden.check_method("getId()I", |this: &mut ServerboundPongPacket| this.get_id());
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundPongPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ServerCommonPacketListener;)V",
            |this: &mut ServerboundPongPacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
