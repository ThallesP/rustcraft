//! net/minecraft/network/protocol/game/ClientboundRotateHeadPacket.java
//!
//! Not ported yet: the constructor from an entity, which the server sends it with, and
//! `getEntity`, which the listener does itself with the level.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::util::mth::Mth;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundRotateHeadPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundRotateHeadPacket::write,
            ClientboundRotateHeadPacket::read,
        ))
    });

/// Where another entity's head turned, as a byte (`Mth.packDegrees`).
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundRotateHeadPacket {
    entity_id: i32,
    y_head_rot: i8,
}

impl ClientboundRotateHeadPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundRotateHeadPacket,
    > = &STREAM_CODEC;

    fn read(input: &mut FriendlyByteBuf) -> ClientboundRotateHeadPacket {
        let entity_id = input.read_var_int();
        let y_head_rot = input.read_byte();
        ClientboundRotateHeadPacket {
            entity_id,
            y_head_rot,
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.entity_id);
        output.write_byte(i32::from(self.y_head_rot));
    }

    /// The id of the entity, which Java's `getEntity` looks up in the level.
    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }

    pub fn get_y_head_rot(&self) -> f32 {
        Mth::unpack_degrees(self.y_head_rot)
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundRotateHeadPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_ROTATE_HEAD
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_rotate_mob(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundRotateHeadPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields its constructor reading a buffer sets, read by reflection.
    encode_fields!(ClientboundRotateHeadPacket {
        entity_id: i32,
        y_head_rot: i8,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundRotateHeadPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundRotateHeadPacket");
        golden.check_codec("STREAM_CODEC", || ClientboundRotateHeadPacket::STREAM_CODEC);
        golden.pending(
            "<init>(Lnet/minecraft/world/entity/Entity;B)V",
            "takes an entity, which the harness has no generator for",
        );
        golden.pending(
            "getEntity(Lnet/minecraft/world/level/Level;)Lnet/minecraft/world/entity/Entity;",
            "looks the entity up in a level, which the listener does itself with its id",
        );
        golden.check_method("getYHeadRot()F", |this: &mut This| this.get_y_head_rot());
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
