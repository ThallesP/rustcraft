//! net/minecraft/network/protocol/game/ClientboundHurtAnimationPacket.java
//!
//! Not ported yet: the constructor from a living entity, which the server sends it with.

use std::fmt;
use std::sync::LazyLock;

use crate::java::lang::float::Float;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundHurtAnimationPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundHurtAnimationPacket::write,
            ClientboundHurtAnimationPacket::from_friendly_byte_buf,
        ))
    });

/// An entity being hurt, and the direction the hit came from, which its hurt animation tilts
/// toward.
#[derive(Clone, Debug)]
pub struct ClientboundHurtAnimationPacket {
    id: i32,
    yaw: f32,
}

impl ClientboundHurtAnimationPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundHurtAnimationPacket,
    > = &STREAM_CODEC;

    pub fn new(id: i32, yaw: f32) -> ClientboundHurtAnimationPacket {
        ClientboundHurtAnimationPacket { id, yaw }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundHurtAnimationPacket {
        let id = input.read_var_int();
        ClientboundHurtAnimationPacket::new(id, input.read_float())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.id);
        output.write_float(self.yaw);
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// A record's: the id's hash, then `Float.hashCode` of the yaw, added to 31 times the one
    /// before.
    pub fn hash_code(&self) -> i32 {
        self.id
            .wrapping_mul(31)
            .wrapping_add(Float::hash_code(self.yaw))
    }
}

/// A record's equals compares a float component with `Float.compare`, so by `floatToIntBits`: NaN
/// equals NaN, and -0.0 isn't 0.0.
impl PartialEq for ClientboundHurtAnimationPacket {
    fn eq(&self, other: &ClientboundHurtAnimationPacket) -> bool {
        self.id == other.id
            && Float::float_to_int_bits(self.yaw) == Float::float_to_int_bits(other.yaw)
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundHurtAnimationPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_HURT_ANIMATION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_hurt_animation(self);
    }
}

/// A record's: `ClientboundHurtAnimationPacket[id=1, yaw=90.0]`.
impl fmt::Display for ClientboundHurtAnimationPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundHurtAnimationPacket[id={}, yaw={}]",
            self.id,
            Float::to_string(self.yaw)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundHurtAnimationPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundHurtAnimationPacket { id: i32, yaw: f32 });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundHurtAnimationPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundHurtAnimationPacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Lnet/minecraft/world/entity/LivingEntity;)V",
            "takes an entity, which the harness has no generator for",
        );
        golden.check("<init>(IF)V", ClientboundHurtAnimationPacket::new);
        golden.check_method("id()I", |this: &mut This| this.id());
        golden.check_method("yaw()F", |this: &mut This| this.yaw());
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
