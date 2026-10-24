//! net/minecraft/network/protocol/game/ClientboundSetBorderCenterPacket.java
//!
//! Not ported yet: its public constructor, which reads a server's `WorldBorder` (`getCenterX` and `getCenterZ`);
//! the client only reads the packet.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetBorderCenterPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetBorderCenterPacket::write,
            ClientboundSetBorderCenterPacket::from_friendly_byte_buf,
        ))
    });

/// The world border's new center.
#[derive(Clone, Debug)]
pub struct ClientboundSetBorderCenterPacket {
    new_center_x: f64,
    new_center_z: f64,
}

impl ClientboundSetBorderCenterPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetBorderCenterPacket,
    > = &STREAM_CODEC;

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundSetBorderCenterPacket {
        let new_center_x = input.read_double();
        let new_center_z = input.read_double();
        ClientboundSetBorderCenterPacket {
            new_center_x,
            new_center_z,
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_double(self.new_center_x);
        output.write_double(self.new_center_z);
    }

    pub fn get_new_center_x(&self) -> f64 {
        self.new_center_x
    }

    pub fn get_new_center_z(&self) -> f64 {
        self.new_center_z
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetBorderCenterPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_BORDER_CENTER
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_border_center(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetBorderCenterPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields it reads.
    encode_fields!(ClientboundSetBorderCenterPacket {
        new_center_x: f64,
        new_center_z: f64
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetBorderCenterPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSetBorderCenterPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetBorderCenterPacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Lnet/minecraft/world/level/border/WorldBorder;)V",
            "reads a server's WorldBorder; the client only reads the packet",
        );
        golden.check_method("getNewCenterX()D", |this: &mut This| {
            this.get_new_center_x()
        });
        golden.check_method("getNewCenterZ()D", |this: &mut This| {
            this.get_new_center_z()
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
