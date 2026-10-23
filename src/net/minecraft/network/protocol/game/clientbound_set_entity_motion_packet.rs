//! net/minecraft/network/protocol/game/ClientboundSetEntityMotionPacket.java
//!
//! Not ported yet: the constructor from an entity, which the server sends it with.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::phys::vec3::Vec3;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundSetEntityMotionPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            ByteBufCodecs::VAR_INT,
            ClientboundSetEntityMotionPacket::id,
            Vec3::LP_STREAM_CODEC,
            ClientboundSetEntityMotionPacket::movement,
            ClientboundSetEntityMotionPacket::new,
        ))
    });

/// An entity's movement, which the server sets (a knockback, an explosion, a launch) and the
/// client lerps to.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetEntityMotionPacket {
    id: i32,
    movement: Vec3,
}

impl ClientboundSetEntityMotionPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundSetEntityMotionPacket> =
        &STREAM_CODEC;

    pub fn new(id: i32, movement: Vec3) -> ClientboundSetEntityMotionPacket {
        ClientboundSetEntityMotionPacket { id, movement }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn movement(&self) -> Vec3 {
        self.movement
    }

    /// A record's: the id's hash, then the movement's, added to 31 times the one before.
    pub fn hash_code(&self) -> i32 {
        self.id
            .wrapping_mul(31)
            .wrapping_add(self.movement.hash_code())
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetEntityMotionPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_ENTITY_MOTION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_entity_motion(self);
    }
}

/// A record's: `ClientboundSetEntityMotionPacket[id=1, movement=(0.0, 0.5, 0.0)]`.
impl fmt::Display for ClientboundSetEntityMotionPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundSetEntityMotionPacket[id={}, movement={}]",
            self.id, self.movement
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetEntityMotionPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    encode_fields!(ClientboundSetEntityMotionPacket {
        id: i32,
        movement: Vec3,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetEntityMotionPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetEntityMotionPacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Lnet/minecraft/world/entity/Entity;)V",
            "takes an entity, which the harness has no generator for",
        );
        golden.check(
            "<init>(ILnet/minecraft/world/phys/Vec3;)V",
            ClientboundSetEntityMotionPacket::new,
        );
        golden.check_method("id()I", |this: &mut This| this.id());
        golden.check_method(
            "movement()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut This| this.movement(),
        );
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
            "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
