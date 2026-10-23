//! net/minecraft/network/protocol/game/ClientboundDamageEventPacket.java
//!
//! Not ported yet: the constructor from an entity and its `DamageSource`, which the server sends
//! it with, and `getSource(Level)`, which makes the `DamageSource` of the level's entities; both
//! come with damage sources. Its record's `hashCode` hashes the holder's identity hash, which
//! changes from run to run, and its `toString` prints the damage type's values, which come with
//! the registries' elements.

use std::sync::LazyLock;

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::position::Position;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::world::damagesource::damage_type::DamageType;
use crate::net::minecraft::world::phys::vec3::Vec3;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundDamageEventPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundDamageEventPacket::write,
            ClientboundDamageEventPacket::from_registry_friendly_byte_buf,
        ))
    });

/// An entity taking damage: its type, the entities that caused it and dealt it (-1 for none),
/// and where it came from when no entity did (a bed or respawn anchor exploding).
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundDamageEventPacket {
    entity_id: i32,
    source_type: Holder,
    source_cause_id: i32,
    source_direct_id: i32,
    source_position: Option<Vec3>,
}

impl ClientboundDamageEventPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundDamageEventPacket,
    > = &STREAM_CODEC;

    pub fn new(
        entity_id: i32,
        source_type: Holder,
        source_cause_id: i32,
        source_direct_id: i32,
        source_position: Option<Vec3>,
    ) -> ClientboundDamageEventPacket {
        ClientboundDamageEventPacket {
            entity_id,
            source_type,
            source_cause_id,
            source_direct_id,
            source_position,
        }
    }

    fn from_registry_friendly_byte_buf(
        input: &mut RegistryFriendlyByteBuf,
    ) -> ClientboundDamageEventPacket {
        let entity_id = input.read_var_int();
        let source_type = DamageType::STREAM_CODEC.decode(input);
        let source_cause_id = ClientboundDamageEventPacket::read_optional_entity_id(input);
        let source_direct_id = ClientboundDamageEventPacket::read_optional_entity_id(input);
        let source_position = input.read_optional(|i: &mut FriendlyByteBuf| {
            Vec3::new(i.read_double(), i.read_double(), i.read_double())
        });
        ClientboundDamageEventPacket::new(
            entity_id,
            source_type,
            source_cause_id,
            source_direct_id,
            source_position,
        )
    }

    /// The id one up, so none (-1) is 0.
    fn write_optional_entity_id(output: &mut FriendlyByteBuf, id: i32) {
        output.write_var_int(id.wrapping_add(1));
    }

    fn read_optional_entity_id(input: &mut FriendlyByteBuf) -> i32 {
        input.read_var_int().wrapping_sub(1)
    }

    fn write(&self, output: &mut RegistryFriendlyByteBuf) {
        output.write_var_int(self.entity_id);
        DamageType::STREAM_CODEC.encode(output, &self.source_type);
        ClientboundDamageEventPacket::write_optional_entity_id(output, self.source_cause_id);
        ClientboundDamageEventPacket::write_optional_entity_id(output, self.source_direct_id);
        output.write_optional(
            self.source_position.as_ref(),
            |o: &mut FriendlyByteBuf, pos: &Vec3| {
                o.write_double(pos.x());
                o.write_double(pos.y());
                o.write_double(pos.z());
            },
        );
    }

    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }

    pub fn source_type(&self) -> Holder {
        self.source_type.clone()
    }

    pub fn source_cause_id(&self) -> i32 {
        self.source_cause_id
    }

    pub fn source_direct_id(&self) -> i32 {
        self.source_direct_id
    }

    pub fn source_position(&self) -> Option<Vec3> {
        self.source_position
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundDamageEventPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_DAMAGE_EVENT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_damage_event(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundDamageEventPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    encode_fields!(ClientboundDamageEventPacket {
        entity_id: i32,
        source_type: Holder,
        source_cause_id: i32,
        source_direct_id: i32,
        source_position: Option<Vec3>,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundDamageEventPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundDamageEventPacket");
        golden.pending("hashCode()I", "hashes the holder's identity hash");
        golden.pending(
            "toString()Ljava/lang/String;",
            "prints the damage type's values, which come with the registries' elements",
        );
        golden.pending(
            "<init>(Lnet/minecraft/world/entity/Entity;Lnet/minecraft/world/damagesource/DamageSource;)V",
            "takes an entity and a damage source, which the harness has no generators for",
        );
        golden.pending(
            "getSource(Lnet/minecraft/world/level/Level;)Lnet/minecraft/world/damagesource/DamageSource;",
            "makes a damage source of the level's entities, which aren't ported",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundDamageEventPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(ILnet/minecraft/core/Holder;IILjava/util/Optional;)V",
            ClientboundDamageEventPacket::new,
        );
        golden.check_method("entityId()I", |this: &mut This| this.entity_id());
        golden.check_method(
            "sourceType()Lnet/minecraft/core/Holder;",
            |this: &mut This| this.source_type(),
        );
        golden.check_method("sourceCauseId()I", |this: &mut This| this.source_cause_id());
        golden.check_method("sourceDirectId()I", |this: &mut This| {
            this.source_direct_id()
        });
        golden.check_method("sourcePosition()Ljava/util/Optional;", |this: &mut This| {
            this.source_position()
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
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
