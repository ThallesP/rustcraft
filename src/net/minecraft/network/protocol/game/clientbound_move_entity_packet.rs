//! net/minecraft/network/protocol/game/ClientboundMoveEntityPacket.java
//!
//! Java's abstract class has a subclass for each kind of move, `Pos`, `PosRot` and `Rot`, each
//! with its own type and codec, which the listener handles as the class they extend. Here each
//! holds the fields it sets in the class and derefs to it (PORTING.md), like
//! `ServerboundMovePlayerPacket`'s. Not ported yet: `getEntity`, which the listener does itself
//! with the level, and `toString`, which prints `Object`'s identity hash.

use std::ops::Deref;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::game::vec_delta::VecDelta;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::util::mth::Mth;

/// An entity's move, its new rotation, or both: the delta from where the client last had it, its
/// rotation as bytes (`Mth.packDegrees`), and whether it's on the ground.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundMoveEntityPacket {
    entity_id: i32,
    delta: VecDelta,
    y_rot: i8,
    x_rot: i8,
    on_ground: bool,
    has_rot: bool,
    has_pos: bool,
}

impl ClientboundMoveEntityPacket {
    const ON_GROUND_FLAG: i32 = 1;
    const STEP_COUNT_OFFSET: i32 = 1;

    fn unpack_on_ground(properties: i32) -> bool {
        (properties & ClientboundMoveEntityPacket::ON_GROUND_FLAG) != 0
    }

    fn unpack_step_count(properties: i32) -> i32 {
        ((properties as u32) >> ClientboundMoveEntityPacket::STEP_COUNT_OFFSET) as i32
    }

    fn pack_properties(on_ground: bool, step_count: i32) -> i32 {
        i32::from(on_ground) | step_count << ClientboundMoveEntityPacket::STEP_COUNT_OFFSET
    }

    fn new(
        entity_id: i32,
        delta: VecDelta,
        y_rot: i8,
        x_rot: i8,
        on_ground: bool,
        has_rot: bool,
        has_pos: bool,
    ) -> ClientboundMoveEntityPacket {
        ClientboundMoveEntityPacket {
            entity_id,
            delta,
            y_rot,
            x_rot,
            on_ground,
            has_rot,
            has_pos,
        }
    }

    /// The id of the entity, which Java's `getEntity` looks up in the level.
    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }

    pub fn get_position_delta(&self) -> &VecDelta {
        &self.delta
    }

    pub fn get_y_rot(&self) -> f32 {
        Mth::unpack_degrees(self.y_rot)
    }

    pub fn get_x_rot(&self) -> f32 {
        Mth::unpack_degrees(self.x_rot)
    }

    pub fn has_rotation(&self) -> bool {
        self.has_rot
    }

    pub fn has_position(&self) -> bool {
        self.has_pos
    }

    pub fn is_on_ground(&self) -> bool {
        self.on_ground
    }

    /// A move's id, then its properties (on the ground, and how many steps its delta has).
    fn read_header(input: &mut FriendlyByteBuf) -> (i32, VecDelta, bool) {
        let entity_id = input.read_var_int();
        let properties = input.read_var_int();
        let delta = VecDelta::read(
            input,
            ClientboundMoveEntityPacket::unpack_step_count(properties),
        );
        (
            entity_id,
            delta,
            ClientboundMoveEntityPacket::unpack_on_ground(properties),
        )
    }

    fn write_header(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.entity_id);
        output.write_var_int(ClientboundMoveEntityPacket::pack_properties(
            self.on_ground,
            self.delta.step_count(),
        ));
        VecDelta::write(output, &self.delta);
    }
}

static POS_STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, Pos> =
    LazyLock::new(|| Box::new(packet::codec(Pos::write, Pos::read)));
static POS_ROT_STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, PosRot> =
    LazyLock::new(|| Box::new(packet::codec(PosRot::write, PosRot::read)));
static ROT_STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, Rot> =
    LazyLock::new(|| Box::new(packet::codec(Rot::write, Rot::read)));

/// `ClientboundMoveEntityPacket.Pos`: where the entity moved.
#[derive(Clone, Debug, PartialEq)]
pub struct Pos {
    parent: ClientboundMoveEntityPacket,
}

impl Pos {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, Pos> = &POS_STREAM_CODEC;

    pub fn new(id: i32, delta: VecDelta, on_ground: bool) -> Pos {
        Pos {
            parent: ClientboundMoveEntityPacket::new(id, delta, 0, 0, on_ground, false, true),
        }
    }

    fn read(input: &mut FriendlyByteBuf) -> Pos {
        let (entity_id, delta, on_ground) = ClientboundMoveEntityPacket::read_header(input);
        Pos::new(entity_id, delta, on_ground)
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        self.write_header(output);
    }
}

/// `ClientboundMoveEntityPacket.PosRot`: where the entity moved, and where it looks.
#[derive(Clone, Debug, PartialEq)]
pub struct PosRot {
    parent: ClientboundMoveEntityPacket,
}

impl PosRot {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, PosRot> =
        &POS_ROT_STREAM_CODEC;

    pub fn new(id: i32, delta: VecDelta, y_rot: i8, x_rot: i8, on_ground: bool) -> PosRot {
        PosRot {
            parent: ClientboundMoveEntityPacket::new(
                id, delta, y_rot, x_rot, on_ground, true, true,
            ),
        }
    }

    fn read(input: &mut FriendlyByteBuf) -> PosRot {
        let (entity_id, delta, on_ground) = ClientboundMoveEntityPacket::read_header(input);
        let y_rot = input.read_byte();
        let x_rot = input.read_byte();
        PosRot::new(entity_id, delta, y_rot, x_rot, on_ground)
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        self.write_header(output);
        output.write_byte(i32::from(self.y_rot));
        output.write_byte(i32::from(self.x_rot));
    }
}

/// `ClientboundMoveEntityPacket.Rot`: where the entity looks.
#[derive(Clone, Debug, PartialEq)]
pub struct Rot {
    parent: ClientboundMoveEntityPacket,
}

impl Rot {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, Rot> = &ROT_STREAM_CODEC;

    pub fn new(id: i32, y_rot: i8, x_rot: i8, on_ground: bool) -> Rot {
        Rot {
            parent: ClientboundMoveEntityPacket::new(
                id,
                VecDelta::ZERO,
                y_rot,
                x_rot,
                on_ground,
                true,
                false,
            ),
        }
    }

    fn read(input: &mut FriendlyByteBuf) -> Rot {
        let entity_id = input.read_var_int();
        let on_ground = input.read_boolean();
        let y_rot = input.read_byte();
        let x_rot = input.read_byte();
        Rot::new(entity_id, y_rot, x_rot, on_ground)
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.entity_id);
        output.write_boolean(self.on_ground);
        output.write_byte(i32::from(self.y_rot));
        output.write_byte(i32::from(self.x_rot));
    }
}

/// The class the kinds extend, as each kind hands itself to the listener: of its kind's type.
impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundMoveEntityPacket {
    fn r#type(&self) -> &'static PacketType {
        match (self.has_pos, self.has_rot) {
            (true, true) => GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_POS_ROT,
            (true, false) => GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_POS,
            _ => GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_ROT,
        }
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_move_entity(self);
    }
}

macro_rules! kind {
    ($kind:ident, $type:ident) => {
        impl Deref for $kind {
            type Target = ClientboundMoveEntityPacket;

            fn deref(&self) -> &ClientboundMoveEntityPacket {
                &self.parent
            }
        }

        impl<L: ClientGamePacketListener + ?Sized> Packet<L> for $kind {
            fn r#type(&self) -> &'static PacketType {
                GamePacketTypes::$type
            }

            fn handle(&self, listener: &mut L) {
                listener.handle_move_entity(&self.parent);
            }
        }
    };
}

kind!(Pos, CLIENTBOUND_MOVE_ENTITY_POS);
kind!(PosRot, CLIENTBOUND_MOVE_ENTITY_POS_ROT);
kind!(Rot, CLIENTBOUND_MOVE_ENTITY_ROT);

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{ClientboundMoveEntityPacket, Pos, PosRot, Rot};
    use crate::golden::{Decode, Encode, Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::game::vec_delta::VecDelta;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields the class they all extend declares, read by reflection.
    encode_fields!(ClientboundMoveEntityPacket {
        entity_id: i32,
        delta: VecDelta,
        y_rot: i8,
        x_rot: i8,
        on_ground: bool,
        has_rot: bool,
        has_pos: bool,
    });

    macro_rules! kind_fields {
        ($kind:ident) => {
            impl Decode for $kind {
                fn decode(value: &Value) -> Self {
                    $kind {
                        parent: ClientboundMoveEntityPacket::decode(value),
                    }
                }
            }

            impl Encode for $kind {
                fn encode(&self) -> Value {
                    self.parent.encode()
                }

                fn matches(&self, expected: &Value) -> bool {
                    self.parent.matches(expected)
                }
            }
        };
    }

    kind_fields!(Pos);
    kind_fields!(PosRot);
    kind_fields!(Rot);

    type Listener = dyn ClientGamePacketListener;

    /// The methods every kind has from the class they extend.
    macro_rules! check_inherited {
        ($golden:ident, $kind:ident) => {
            $golden.check_method(
                "getPositionDelta()Lnet/minecraft/network/protocol/game/VecDelta;",
                |this: &mut $kind| this.get_position_delta().clone(),
            );
            $golden.check_method("getYRot()F", |this: &mut $kind| this.get_y_rot());
            $golden.check_method("getXRot()F", |this: &mut $kind| this.get_x_rot());
            $golden.check_method("hasRotation()Z", |this: &mut $kind| this.has_rotation());
            $golden.check_method("hasPosition()Z", |this: &mut $kind| this.has_position());
            $golden.check_method("isOnGround()Z", |this: &mut $kind| this.is_on_ground());
            $golden.check_method(
                "type()Lnet/minecraft/network/protocol/PacketType;",
                |this: &mut $kind| Packet::<Listener>::r#type(this).clone(),
            );
            $golden.check_mut(
                "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
                |this: &mut $kind, listener: &mut Recorder| this.handle(listener),
            );
        };
    }

    #[test]
    fn pos_matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundMoveEntityPacket$Pos");
        golden.check_codec("STREAM_CODEC", || Pos::STREAM_CODEC);
        golden.check(
            "<init>(ILnet/minecraft/network/protocol/game/VecDelta;Z)V",
            Pos::new,
        );
        check_inherited!(golden, Pos);
        golden.finish();
    }

    #[test]
    fn pos_rot_matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundMoveEntityPacket$PosRot");
        golden.check_codec("STREAM_CODEC", || PosRot::STREAM_CODEC);
        golden.check(
            "<init>(ILnet/minecraft/network/protocol/game/VecDelta;BBZ)V",
            PosRot::new,
        );
        check_inherited!(golden, PosRot);
        golden.finish();
    }

    #[test]
    fn rot_matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundMoveEntityPacket$Rot");
        golden.check_codec("STREAM_CODEC", || Rot::STREAM_CODEC);
        golden.check("<init>(IBBZ)V", Rot::new);
        check_inherited!(golden, Rot);
        golden.finish();
    }
}
