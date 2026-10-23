//! net/minecraft/network/protocol/game/ClientboundEntityEventPacket.java
//!
//! Not ported yet: the constructor from an entity, which the server sends it with, and
//! `getEntity(Level)`, which looks the entity up in the level and comes with the level's
//! entities; until then `entity_id` hands out the id it looks up.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundEntityEventPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundEntityEventPacket::write,
            ClientboundEntityEventPacket::from_friendly_byte_buf,
        ))
    });

/// Something an entity does that the client plays out (`EntityEvent`): an animation, a sound or
/// particles, its death, or for the player, the debug screen's detail and its permission level.
#[derive(Clone, Debug)]
pub struct ClientboundEntityEventPacket {
    entity_id: i32,
    event_id: i8,
}

impl ClientboundEntityEventPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundEntityEventPacket,
    > = &STREAM_CODEC;

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundEntityEventPacket {
        let entity_id = input.read_int();
        let event_id = input.read_byte();
        ClientboundEntityEventPacket {
            entity_id,
            event_id,
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_int(self.entity_id);
        output.write_byte(i32::from(self.event_id));
    }

    /// The id of the entity `getEntity(Level)` looks up, which isn't ported yet.
    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }

    /// One of `EntityEvent`'s constants.
    pub fn get_event_id(&self) -> i8 {
        self.event_id
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundEntityEventPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_ENTITY_EVENT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_entity_event(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundEntityEventPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // Its fields, which the reader sets.
    encode_fields!(ClientboundEntityEventPacket {
        entity_id: i32,
        event_id: i8,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundEntityEventPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundEntityEventPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundEntityEventPacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Lnet/minecraft/world/entity/Entity;B)V",
            "takes an entity, which the harness has no generator for",
        );
        golden.pending(
            "getEntity(Lnet/minecraft/world/level/Level;)Lnet/minecraft/world/entity/Entity;",
            "looks the entity up in the level, whose entities aren't ported",
        );
        golden.check_method("getEventId()B", |this: &mut This| this.get_event_id());
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
