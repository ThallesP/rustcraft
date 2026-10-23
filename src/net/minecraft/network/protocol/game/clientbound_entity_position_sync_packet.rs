//! net/minecraft/network/protocol/game/ClientboundEntityPositionSyncPacket.java
//!
//! Not ported yet: `of(Entity)` and `of(Entity, PositionPath)`, which the server sends it with.

use std::fmt;
use std::sync::LazyLock;

use crate::java::lang::float::Float;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::entity::position_path::PositionPath;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundEntityPositionSyncPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite5(
            ByteBufCodecs::VAR_INT,
            ClientboundEntityPositionSyncPacket::id,
            PositionPath::STREAM_CODEC,
            ClientboundEntityPositionSyncPacket::position,
            ByteBufCodecs::FLOAT,
            ClientboundEntityPositionSyncPacket::y_rot,
            ByteBufCodecs::FLOAT,
            ClientboundEntityPositionSyncPacket::x_rot,
            ByteBufCodecs::BOOL,
            ClientboundEntityPositionSyncPacket::on_ground,
            ClientboundEntityPositionSyncPacket::new,
        ))
    });

/// Where another entity is, its full position (or path) and rotation rather than a move's delta:
/// what the server sends when a delta won't do.
#[derive(Clone, Debug)]
pub struct ClientboundEntityPositionSyncPacket {
    id: i32,
    position: PositionPath,
    y_rot: f32,
    x_rot: f32,
    on_ground: bool,
}

impl ClientboundEntityPositionSyncPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundEntityPositionSyncPacket,
    > = &STREAM_CODEC;

    pub fn new(
        id: i32,
        position: PositionPath,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
    ) -> ClientboundEntityPositionSyncPacket {
        ClientboundEntityPositionSyncPacket {
            id,
            position,
            y_rot,
            x_rot,
            on_ground,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn position(&self) -> PositionPath {
        self.position.clone()
    }

    pub fn y_rot(&self) -> f32 {
        self.y_rot
    }

    pub fn x_rot(&self) -> f32 {
        self.x_rot
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn has_position(&self) -> bool {
        true
    }

    pub fn has_rotation(&self) -> bool {
        true
    }

    /// A record's: each component's hash added to 31 times the ones before, a float's by
    /// `Float.hashCode`.
    pub fn hash_code(&self) -> i32 {
        [
            self.position.hash_code(),
            Float::hash_code(self.y_rot),
            Float::hash_code(self.x_rot),
            i32::from(if self.on_ground { 1231i16 } else { 1237 }),
        ]
        .into_iter()
        .fold(self.id, |hash, component| {
            hash.wrapping_mul(31).wrapping_add(component)
        })
    }
}

/// A record's `equals`: the floats compared like `Float.compare`, by their bits but any NaN like any
/// other.
impl PartialEq for ClientboundEntityPositionSyncPacket {
    fn eq(&self, other: &ClientboundEntityPositionSyncPacket) -> bool {
        self.id == other.id
            && self.position == other.position
            && Float::compare(self.y_rot, other.y_rot).is_eq()
            && Float::compare(self.x_rot, other.x_rot).is_eq()
            && self.on_ground == other.on_ground
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundEntityPositionSyncPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_ENTITY_POSITION_SYNC
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_entity_position_sync(self);
    }
}

/// A record's: `ClientboundEntityPositionSyncPacket[id=1, position=Linear[…], yRot=0.0, xRot=0.0,
/// onGround=true]`.
impl fmt::Display for ClientboundEntityPositionSyncPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundEntityPositionSyncPacket[id={}, position={}, yRot={}, xRot={}, onGround={}]",
            self.id,
            self.position,
            Float::to_string(self.y_rot),
            Float::to_string(self.x_rot),
            self.on_ground
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundEntityPositionSyncPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::entity::position_path::PositionPath;

    encode_fields!(ClientboundEntityPositionSyncPacket {
        id: i32,
        position: PositionPath,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundEntityPositionSyncPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundEntityPositionSyncPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(ILnet/minecraft/world/entity/PositionPath;FFZ)V",
            ClientboundEntityPositionSyncPacket::new,
        );
        golden.pending(
            "of(Lnet/minecraft/world/entity/Entity;)Lnet/minecraft/network/protocol/game/ClientboundEntityPositionSyncPacket;",
            "takes an entity, which the harness has no generator for",
        );
        golden.pending(
            "of(Lnet/minecraft/world/entity/Entity;Lnet/minecraft/world/entity/PositionPath;)Lnet/minecraft/network/protocol/game/ClientboundEntityPositionSyncPacket;",
            "takes an entity, which the harness has no generator for",
        );
        golden.check_method("id()I", |this: &mut This| this.id());
        golden.check_method(
            "position()Lnet/minecraft/world/entity/PositionPath;",
            |this: &mut This| this.position(),
        );
        golden.check_method("yRot()F", |this: &mut This| this.y_rot());
        golden.check_method("xRot()F", |this: &mut This| this.x_rot());
        golden.check_method("onGround()Z", |this: &mut This| this.on_ground());
        golden.check_method("hasPosition()Z", |this: &mut This| this.has_position());
        golden.check_method("hasRotation()Z", |this: &mut This| this.has_rotation());
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
