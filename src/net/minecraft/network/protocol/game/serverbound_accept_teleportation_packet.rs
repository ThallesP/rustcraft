//! net/minecraft/network/protocol/game/ServerboundAcceptTeleportationPacket.java
//!

use std::sync::LazyLock;

use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use std::fmt;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundAcceptTeleportationPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite6(
            ByteBufCodecs::VAR_INT,
            ServerboundAcceptTeleportationPacket::id,
            ByteBufCodecs::DOUBLE,
            ServerboundAcceptTeleportationPacket::x,
            ByteBufCodecs::DOUBLE,
            ServerboundAcceptTeleportationPacket::y,
            ByteBufCodecs::DOUBLE,
            ServerboundAcceptTeleportationPacket::z,
            ByteBufCodecs::FLOAT,
            ServerboundAcceptTeleportationPacket::y_rot,
            ByteBufCodecs::FLOAT,
            ServerboundAcceptTeleportationPacket::x_rot,
            ServerboundAcceptTeleportationPacket::new,
        ))
    });

/// The client's answer to `player_position`: the teleport's id, and where it put the player.
#[derive(Clone, Debug)]
pub struct ServerboundAcceptTeleportationPacket {
    id: i32,
    x: f64,
    y: f64,
    z: f64,
    y_rot: f32,
    x_rot: f32,
}

impl ServerboundAcceptTeleportationPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundAcceptTeleportationPacket,
    > = &STREAM_CODEC;

    pub fn new(
        id: i32,
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
    ) -> ServerboundAcceptTeleportationPacket {
        ServerboundAcceptTeleportationPacket {
            id,
            x,
            y,
            z,
            y_rot,
            x_rot,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }

    pub fn y_rot(&self) -> f32 {
        self.y_rot
    }

    pub fn x_rot(&self) -> f32 {
        self.x_rot
    }

    /// A record's: each component's hash (the doubles' and floats' bits), added to 31 times the
    /// ones before.
    pub fn hash_code(&self) -> i32 {
        [
            Double::hash_code(self.x),
            Double::hash_code(self.y),
            Double::hash_code(self.z),
            Float::hash_code(self.y_rot),
            Float::hash_code(self.x_rot),
        ]
        .into_iter()
        .fold(self.id, |hash, component| {
            hash.wrapping_mul(31).wrapping_add(component)
        })
    }
}

/// A record's: floating point components compare by their bits, NaNs made one (`Double.compare`
/// and `Float.compare`).
impl PartialEq for ServerboundAcceptTeleportationPacket {
    fn eq(&self, other: &ServerboundAcceptTeleportationPacket) -> bool {
        self.id == other.id
            && Double::double_to_long_bits(self.x) == Double::double_to_long_bits(other.x)
            && Double::double_to_long_bits(self.y) == Double::double_to_long_bits(other.y)
            && Double::double_to_long_bits(self.z) == Double::double_to_long_bits(other.z)
            && Float::float_to_int_bits(self.y_rot) == Float::float_to_int_bits(other.y_rot)
            && Float::float_to_int_bits(self.x_rot) == Float::float_to_int_bits(other.x_rot)
    }
}

impl<L: ServerGamePacketListener + ?Sized> Packet<L> for ServerboundAcceptTeleportationPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::SERVERBOUND_ACCEPT_TELEPORTATION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_accept_teleport_packet(self);
    }
}

/// A record's: `ServerboundAcceptTeleportationPacket[id=1, x=0.5, y=-60.0, z=0.5, yRot=0.0, xRot=0.0]`.
impl fmt::Display for ServerboundAcceptTeleportationPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ServerboundAcceptTeleportationPacket[id={}, x={}, y={}, z={}, yRot={}, xRot={}]",
            self.id,
            Double::to_string(self.x),
            Double::to_string(self.y),
            Double::to_string(self.z),
            Float::to_string(self.y_rot),
            Float::to_string(self.x_rot)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundAcceptTeleportationPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ServerboundAcceptTeleportationPacket {
        id: i32,
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
    });

    type Listener = dyn ServerGamePacketListener;
    type This = ServerboundAcceptTeleportationPacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket",
        );
        golden.check_codec("STREAM_CODEC", || {
            ServerboundAcceptTeleportationPacket::STREAM_CODEC
        });
        golden.check("<init>(IDDDFF)V", ServerboundAcceptTeleportationPacket::new);
        golden.check_method("id()I", |this: &mut This| this.id());
        golden.check_method("x()D", |this: &mut This| this.x());
        golden.check_method("y()D", |this: &mut This| this.y());
        golden.check_method("z()D", |this: &mut This| this.z());
        golden.check_method("yRot()F", |this: &mut This| this.y_rot());
        golden.check_method("xRot()F", |this: &mut This| this.x_rot());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut This| this.hash_code());
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundAcceptTeleportationPacket| this.to_string(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut This| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
