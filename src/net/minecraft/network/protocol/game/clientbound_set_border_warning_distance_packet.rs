//! net/minecraft/network/protocol/game/ClientboundSetBorderWarningDistancePacket.java
//!
//! Not ported yet: its public constructor, which reads a server's `WorldBorder` (`getWarningBlocks`);
//! the client only reads the packet.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetBorderWarningDistancePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetBorderWarningDistancePacket::write,
            ClientboundSetBorderWarningDistancePacket::from_friendly_byte_buf,
        ))
    });

/// How many blocks from the world border its warning shows.
#[derive(Clone, Debug)]
pub struct ClientboundSetBorderWarningDistancePacket {
    warning_blocks: i32,
}

impl ClientboundSetBorderWarningDistancePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetBorderWarningDistancePacket,
    > = &STREAM_CODEC;

    fn from_friendly_byte_buf(
        input: &mut FriendlyByteBuf,
    ) -> ClientboundSetBorderWarningDistancePacket {
        let warning_blocks = input.read_var_int();
        ClientboundSetBorderWarningDistancePacket { warning_blocks }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.warning_blocks);
    }

    pub fn get_warning_blocks(&self) -> i32 {
        self.warning_blocks
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetBorderWarningDistancePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_BORDER_WARNING_DISTANCE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_border_warning_distance(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetBorderWarningDistancePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields it reads.
    encode_fields!(ClientboundSetBorderWarningDistancePacket {
        warning_blocks: i32
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetBorderWarningDistancePacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundSetBorderWarningDistancePacket",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetBorderWarningDistancePacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Lnet/minecraft/world/level/border/WorldBorder;)V",
            "reads a server's WorldBorder; the client only reads the packet",
        );
        golden.check_method("getWarningBlocks()I", |this: &mut This| {
            this.get_warning_blocks()
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut This| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
