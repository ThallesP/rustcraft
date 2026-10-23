//! net/minecraft/network/protocol/game/ClientboundAddEntityPacket.java
//!
//! Not ported yet: the constructors from an entity (with its `ServerEntity`, or at a block),
//! which the server sends it with.

use std::sync::LazyLock;

use crate::java::util::uuid::UUID;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::entity_type::EntityType;
use crate::net::minecraft::world::phys::vec3::Vec3;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundAddEntityPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundAddEntityPacket::write,
            ClientboundAddEntityPacket::from_registry_friendly_byte_buf,
        ))
    });

/// An entity coming into the client's view: its type, where it is, how it moves and looks, and
/// its type's data (a falling block's state, a projectile's owner, a painting's direction).
#[derive(Clone, Debug)]
pub struct ClientboundAddEntityPacket {
    id: i32,
    uuid: UUID,
    r#type: EntityType,
    x: f64,
    y: f64,
    z: f64,
    movement: Vec3,
    x_rot: i8,
    y_rot: i8,
    y_head_rot: i8,
    data: i32,
}

impl ClientboundAddEntityPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundAddEntityPacket,
    > = &STREAM_CODEC;

    /// The rotations are packed into a byte each (`Mth.packDegrees`), the head's after Java's
    /// cast of the double to a float.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i32,
        uuid: UUID,
        x: f64,
        y: f64,
        z: f64,
        x_rot: f32,
        y_rot: f32,
        r#type: EntityType,
        data: i32,
        movement: Vec3,
        y_head_rot: f64,
    ) -> ClientboundAddEntityPacket {
        ClientboundAddEntityPacket {
            id,
            uuid,
            x,
            y,
            z,
            movement,
            x_rot: Mth::pack_degrees(x_rot),
            y_rot: Mth::pack_degrees(y_rot),
            y_head_rot: Mth::pack_degrees(y_head_rot as f32),
            r#type,
            data,
        }
    }

    fn from_registry_friendly_byte_buf(
        input: &mut RegistryFriendlyByteBuf,
    ) -> ClientboundAddEntityPacket {
        let id = input.read_var_int();
        let uuid = input.read_uuid();
        let r#type = EntityType::STREAM_CODEC.decode(input);
        let x = input.read_double();
        let y = input.read_double();
        let z = input.read_double();
        let movement = Vec3::LP_STREAM_CODEC.decode(input);
        let x_rot = input.read_byte();
        let y_rot = input.read_byte();
        let y_head_rot = input.read_byte();
        let data = input.read_var_int();
        ClientboundAddEntityPacket {
            id,
            uuid,
            r#type,
            x,
            y,
            z,
            movement,
            x_rot,
            y_rot,
            y_head_rot,
            data,
        }
    }

    fn write(&self, output: &mut RegistryFriendlyByteBuf) {
        output.write_var_int(self.id);
        output.write_uuid(&self.uuid);
        EntityType::STREAM_CODEC.encode(output, &self.r#type);
        output.write_double(self.x);
        output.write_double(self.y);
        output.write_double(self.z);
        Vec3::LP_STREAM_CODEC.encode(output, &self.movement);
        output.write_byte(i32::from(self.x_rot));
        output.write_byte(i32::from(self.y_rot));
        output.write_byte(i32::from(self.y_head_rot));
        output.write_var_int(self.data);
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_uuid(&self) -> UUID {
        self.uuid
    }

    pub fn get_type(&self) -> EntityType {
        self.r#type
    }

    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }

    pub fn get_z(&self) -> f64 {
        self.z
    }

    pub fn get_movement(&self) -> Vec3 {
        self.movement
    }

    pub fn get_x_rot(&self) -> f32 {
        Mth::unpack_degrees(self.x_rot)
    }

    pub fn get_y_rot(&self) -> f32 {
        Mth::unpack_degrees(self.y_rot)
    }

    pub fn get_y_head_rot(&self) -> f32 {
        Mth::unpack_degrees(self.y_head_rot)
    }

    pub fn get_data(&self) -> i32 {
        self.data
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundAddEntityPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_ADD_ENTITY
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_add_entity(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundAddEntityPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::java::util::uuid::UUID;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::entity::entity_type::EntityType;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    // Its fields, the rotations packed.
    encode_fields!(ClientboundAddEntityPacket {
        id: i32,
        uuid: UUID,
        r#type: EntityType,
        x: f64,
        y: f64,
        z: f64,
        movement: Vec3,
        x_rot: i8,
        y_rot: i8,
        y_head_rot: i8,
        data: i32,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundAddEntityPacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.ClientboundAddEntityPacket");
        for constructor in [
            "<init>(Lnet/minecraft/world/entity/Entity;Lnet/minecraft/server/level/ServerEntity;)V",
            "<init>(Lnet/minecraft/world/entity/Entity;Lnet/minecraft/server/level/ServerEntity;I)V",
            "<init>(Lnet/minecraft/world/entity/Entity;ILnet/minecraft/core/BlockPos;)V",
        ] {
            golden.pending(
                constructor,
                "takes an entity, which the harness has no generator for",
            );
        }
        golden.check_codec("STREAM_CODEC", || ClientboundAddEntityPacket::STREAM_CODEC);
        golden.check(
            "<init>(ILjava/util/UUID;DDDFFLnet/minecraft/world/entity/EntityType;ILnet/minecraft/world/phys/Vec3;D)V",
            ClientboundAddEntityPacket::new,
        );
        golden.check_method("getId()I", |this: &mut This| this.get_id());
        golden.check_method("getUUID()Ljava/util/UUID;", |this: &mut This| {
            this.get_uuid()
        });
        golden.check_method(
            "getType()Lnet/minecraft/world/entity/EntityType;",
            |this: &mut This| this.get_type(),
        );
        golden.check_method("getX()D", |this: &mut This| this.get_x());
        golden.check_method("getY()D", |this: &mut This| this.get_y());
        golden.check_method("getZ()D", |this: &mut This| this.get_z());
        golden.check_method(
            "getMovement()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut This| this.get_movement(),
        );
        golden.check_method("getXRot()F", |this: &mut This| this.get_x_rot());
        golden.check_method("getYRot()F", |this: &mut This| this.get_y_rot());
        golden.check_method("getYHeadRot()F", |this: &mut This| this.get_y_head_rot());
        golden.check_method("getData()I", |this: &mut This| this.get_data());
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
