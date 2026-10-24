//! net/minecraft/network/protocol/game/ClientboundSetBorderLerpSizePacket.java
//!
//! Not ported yet: its public constructor, which reads a server's `WorldBorder` (`getSize`, `getLerpTarget` and `getLerpTime`);
//! the client only reads the packet.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetBorderLerpSizePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetBorderLerpSizePacket::write,
            ClientboundSetBorderLerpSizePacket::from_friendly_byte_buf,
        ))
    });

/// The world border moving from one size to another, over a number of ticks.
#[derive(Clone, Debug)]
pub struct ClientboundSetBorderLerpSizePacket {
    old_size: f64,
    new_size: f64,
    lerp_time: i64,
}

impl ClientboundSetBorderLerpSizePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetBorderLerpSizePacket,
    > = &STREAM_CODEC;

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundSetBorderLerpSizePacket {
        let old_size = input.read_double();
        let new_size = input.read_double();
        let lerp_time = input.read_var_long();
        ClientboundSetBorderLerpSizePacket {
            old_size,
            new_size,
            lerp_time,
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_double(self.old_size);
        output.write_double(self.new_size);
        output.write_var_long(self.lerp_time);
    }

    pub fn get_old_size(&self) -> f64 {
        self.old_size
    }

    pub fn get_new_size(&self) -> f64 {
        self.new_size
    }

    pub fn get_lerp_time(&self) -> i64 {
        self.lerp_time
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetBorderLerpSizePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_BORDER_LERP_SIZE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_border_lerp_size(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetBorderLerpSizePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields it reads.
    encode_fields!(ClientboundSetBorderLerpSizePacket {
        old_size: f64,
        new_size: f64,
        lerp_time: i64
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetBorderLerpSizePacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSetBorderLerpSizePacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetBorderLerpSizePacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Lnet/minecraft/world/level/border/WorldBorder;)V",
            "reads a server's WorldBorder; the client only reads the packet",
        );
        golden.check_method("getOldSize()D", |this: &mut This| this.get_old_size());
        golden.check_method("getNewSize()D", |this: &mut This| this.get_new_size());
        golden.check_method("getLerpTime()J", |this: &mut This| this.get_lerp_time());
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
