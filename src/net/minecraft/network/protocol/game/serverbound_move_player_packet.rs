//! net/minecraft/network/protocol/game/ServerboundMovePlayerPacket.java
//!
//! Java's abstract class has a subclass for each kind of move, `Pos`, `PosRot`, `Rot` and
//! `StatusOnly`, each with its own type and codec, which the listener handles as the class they
//! extend. Here each holds the fields it sets in the class and derefs to it (PORTING.md).

use std::ops::Deref;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// Where the player is and where it looks, or the parts of that a kind of move sends, with
/// whether it's on the ground and walking into a wall.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundMovePlayerPacket {
    x: f64,
    y: f64,
    z: f64,
    y_rot: f32,
    x_rot: f32,
    on_ground: bool,
    horizontal_collision: bool,
    has_pos: bool,
    has_rot: bool,
}

impl ServerboundMovePlayerPacket {
    const FLAG_ON_GROUND: i32 = 1;
    const FLAG_HORIZONTAL_COLLISION: i32 = 2;

    fn pack_flags(on_ground: bool, horizontal_collision: bool) -> i32 {
        let mut flags = 0;
        if on_ground {
            flags |= ServerboundMovePlayerPacket::FLAG_ON_GROUND;
        }

        if horizontal_collision {
            flags |= ServerboundMovePlayerPacket::FLAG_HORIZONTAL_COLLISION;
        }

        flags
    }

    fn unpack_on_ground(flags: i16) -> bool {
        (i32::from(flags) & ServerboundMovePlayerPacket::FLAG_ON_GROUND) != 0
    }

    fn unpack_horizontal_collision(flags: i16) -> bool {
        (i32::from(flags) & ServerboundMovePlayerPacket::FLAG_HORIZONTAL_COLLISION) != 0
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
        has_pos: bool,
        has_rot: bool,
    ) -> ServerboundMovePlayerPacket {
        ServerboundMovePlayerPacket {
            x,
            y,
            z,
            y_rot,
            x_rot,
            on_ground,
            horizontal_collision,
            has_pos,
            has_rot,
        }
    }

    pub fn get_x(&self, fallback: f64) -> f64 {
        if self.has_pos { self.x } else { fallback }
    }

    pub fn get_y(&self, fallback: f64) -> f64 {
        if self.has_pos { self.y } else { fallback }
    }

    pub fn get_z(&self, fallback: f64) -> f64 {
        if self.has_pos { self.z } else { fallback }
    }

    pub fn get_y_rot(&self, fallback: f32) -> f32 {
        if self.has_rot { self.y_rot } else { fallback }
    }

    pub fn get_x_rot(&self, fallback: f32) -> f32 {
        if self.has_rot { self.x_rot } else { fallback }
    }

    pub fn is_on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn horizontal_collision(&self) -> bool {
        self.horizontal_collision
    }

    pub fn has_position(&self) -> bool {
        self.has_pos
    }

    pub fn has_rotation(&self) -> bool {
        self.has_rot
    }

    /// The flags byte every kind ends with.
    fn read_flags(input: &mut FriendlyByteBuf) -> (bool, bool) {
        let flags = input.read_unsigned_byte();
        (
            ServerboundMovePlayerPacket::unpack_on_ground(flags),
            ServerboundMovePlayerPacket::unpack_horizontal_collision(flags),
        )
    }

    fn write_flags(&self, output: &mut FriendlyByteBuf) {
        output.write_byte(ServerboundMovePlayerPacket::pack_flags(
            self.on_ground,
            self.horizontal_collision,
        ));
    }
}

static POS_STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, Pos> =
    LazyLock::new(|| Box::new(packet::codec(Pos::write, Pos::read)));
static POS_ROT_STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, PosRot> =
    LazyLock::new(|| Box::new(packet::codec(PosRot::write, PosRot::read)));
static ROT_STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, Rot> =
    LazyLock::new(|| Box::new(packet::codec(Rot::write, Rot::read)));
static STATUS_ONLY_STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, StatusOnly> =
    LazyLock::new(|| Box::new(packet::codec(StatusOnly::write, StatusOnly::read)));

/// `ServerboundMovePlayerPacket.Pos`: where the player is.
#[derive(Clone, Debug, PartialEq)]
pub struct Pos {
    parent: ServerboundMovePlayerPacket,
}

impl Pos {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, Pos> = &POS_STREAM_CODEC;

    pub fn new(pos: Vec3, on_ground: bool, horizontal_collision: bool) -> Pos {
        Pos::from_xyz(pos.x, pos.y, pos.z, on_ground, horizontal_collision)
    }

    pub fn from_xyz(x: f64, y: f64, z: f64, on_ground: bool, horizontal_collision: bool) -> Pos {
        Pos {
            parent: ServerboundMovePlayerPacket::new(
                x,
                y,
                z,
                0.0,
                0.0,
                on_ground,
                horizontal_collision,
                true,
                false,
            ),
        }
    }

    fn read(input: &mut FriendlyByteBuf) -> Pos {
        let x = input.read_double();
        let y = input.read_double();
        let z = input.read_double();
        let (on_ground, horizontal_collision) = ServerboundMovePlayerPacket::read_flags(input);
        Pos::from_xyz(x, y, z, on_ground, horizontal_collision)
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_double(self.x);
        output.write_double(self.y);
        output.write_double(self.z);
        self.write_flags(output);
    }
}

/// `ServerboundMovePlayerPacket.PosRot`: where the player is and where it looks.
#[derive(Clone, Debug, PartialEq)]
pub struct PosRot {
    parent: ServerboundMovePlayerPacket,
}

impl PosRot {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, PosRot> =
        &POS_ROT_STREAM_CODEC;

    pub fn new(
        pos: Vec3,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    ) -> PosRot {
        PosRot::from_xyz(
            pos.x,
            pos.y,
            pos.z,
            y_rot,
            x_rot,
            on_ground,
            horizontal_collision,
        )
    }

    pub fn from_xyz(
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    ) -> PosRot {
        PosRot {
            parent: ServerboundMovePlayerPacket::new(
                x,
                y,
                z,
                y_rot,
                x_rot,
                on_ground,
                horizontal_collision,
                true,
                true,
            ),
        }
    }

    fn read(input: &mut FriendlyByteBuf) -> PosRot {
        let x = input.read_double();
        let y = input.read_double();
        let z = input.read_double();
        let y_rot = input.read_float();
        let x_rot = input.read_float();
        let (on_ground, horizontal_collision) = ServerboundMovePlayerPacket::read_flags(input);
        PosRot::from_xyz(x, y, z, y_rot, x_rot, on_ground, horizontal_collision)
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_double(self.x);
        output.write_double(self.y);
        output.write_double(self.z);
        output.write_float(self.y_rot);
        output.write_float(self.x_rot);
        self.write_flags(output);
    }
}

/// `ServerboundMovePlayerPacket.Rot`: where the player looks.
#[derive(Clone, Debug, PartialEq)]
pub struct Rot {
    parent: ServerboundMovePlayerPacket,
}

impl Rot {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, Rot> = &ROT_STREAM_CODEC;

    pub fn new(y_rot: f32, x_rot: f32, on_ground: bool, horizontal_collision: bool) -> Rot {
        Rot {
            parent: ServerboundMovePlayerPacket::new(
                0.0,
                0.0,
                0.0,
                y_rot,
                x_rot,
                on_ground,
                horizontal_collision,
                false,
                true,
            ),
        }
    }

    fn read(input: &mut FriendlyByteBuf) -> Rot {
        let y_rot = input.read_float();
        let x_rot = input.read_float();
        let (on_ground, horizontal_collision) = ServerboundMovePlayerPacket::read_flags(input);
        Rot::new(y_rot, x_rot, on_ground, horizontal_collision)
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_float(self.y_rot);
        output.write_float(self.x_rot);
        self.write_flags(output);
    }
}

/// `ServerboundMovePlayerPacket.StatusOnly`: only whether the player is on the ground and walking
/// into a wall.
#[derive(Clone, Debug, PartialEq)]
pub struct StatusOnly {
    parent: ServerboundMovePlayerPacket,
}

impl StatusOnly {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, StatusOnly> =
        &STATUS_ONLY_STREAM_CODEC;

    pub fn new(on_ground: bool, horizontal_collision: bool) -> StatusOnly {
        StatusOnly {
            parent: ServerboundMovePlayerPacket::new(
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                on_ground,
                horizontal_collision,
                false,
                false,
            ),
        }
    }

    fn read(input: &mut FriendlyByteBuf) -> StatusOnly {
        let (on_ground, horizontal_collision) = ServerboundMovePlayerPacket::read_flags(input);
        StatusOnly::new(on_ground, horizontal_collision)
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        self.write_flags(output);
    }
}

macro_rules! kind {
    ($kind:ident, $type:ident) => {
        impl Deref for $kind {
            type Target = ServerboundMovePlayerPacket;

            fn deref(&self) -> &ServerboundMovePlayerPacket {
                &self.parent
            }
        }

        impl<L: ServerGamePacketListener + ?Sized> Packet<L> for $kind {
            fn r#type(&self) -> &'static PacketType {
                GamePacketTypes::$type
            }

            fn handle(&self, listener: &mut L) {
                listener.handle_move_player(&self.parent);
            }
        }
    };
}

kind!(Pos, SERVERBOUND_MOVE_PLAYER_POS);
kind!(PosRot, SERVERBOUND_MOVE_PLAYER_POS_ROT);
kind!(Rot, SERVERBOUND_MOVE_PLAYER_ROT);
kind!(StatusOnly, SERVERBOUND_MOVE_PLAYER_STATUS_ONLY);

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{Pos, PosRot, Rot, ServerboundMovePlayerPacket, StatusOnly};
    use crate::golden::{Decode, Encode, Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields the class they all extend declares, read by reflection.
    encode_fields!(ServerboundMovePlayerPacket {
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
        has_pos: bool,
        has_rot: bool,
    });

    macro_rules! kind_fields {
        ($kind:ident) => {
            impl Decode for $kind {
                fn decode(value: &Value) -> Self {
                    $kind {
                        parent: ServerboundMovePlayerPacket::decode(value),
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
    kind_fields!(StatusOnly);

    type Listener = dyn ServerGamePacketListener;

    /// The methods every kind has from the class they extend.
    macro_rules! check_inherited {
        ($golden:ident, $kind:ident) => {
            $golden.check_method("getX(D)D", |this: &mut $kind, fallback: f64| {
                this.get_x(fallback)
            });
            $golden.check_method("getY(D)D", |this: &mut $kind, fallback: f64| {
                this.get_y(fallback)
            });
            $golden.check_method("getZ(D)D", |this: &mut $kind, fallback: f64| {
                this.get_z(fallback)
            });
            $golden.check_method("getYRot(F)F", |this: &mut $kind, fallback: f32| {
                this.get_y_rot(fallback)
            });
            $golden.check_method("getXRot(F)F", |this: &mut $kind, fallback: f32| {
                this.get_x_rot(fallback)
            });
            $golden.check_method("isOnGround()Z", |this: &mut $kind| this.is_on_ground());
            $golden.check_method("horizontalCollision()Z", |this: &mut $kind| {
                this.horizontal_collision()
            });
            $golden.check_method("hasPosition()Z", |this: &mut $kind| this.has_position());
            $golden.check_method("hasRotation()Z", |this: &mut $kind| this.has_rotation());
            $golden.check_method(
                "type()Lnet/minecraft/network/protocol/PacketType;",
                |this: &mut $kind| Packet::<Listener>::r#type(this).clone(),
            );
            $golden.check_mut(
                "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V",
                |this: &mut $kind, listener: &mut Recorder| this.handle(listener),
            );
        };
    }

    #[test]
    fn pos_matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Pos");
        golden.check_codec("STREAM_CODEC", || Pos::STREAM_CODEC);
        golden.check("<init>(DDDZZ)V", Pos::from_xyz);
        golden.check("<init>(Lnet/minecraft/world/phys/Vec3;ZZ)V", Pos::new);
        check_inherited!(golden, Pos);
        golden.finish();
    }

    #[test]
    fn pos_rot_matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$PosRot");
        golden.check_codec("STREAM_CODEC", || PosRot::STREAM_CODEC);
        golden.check("<init>(DDDFFZZ)V", PosRot::from_xyz);
        golden.check("<init>(Lnet/minecraft/world/phys/Vec3;FFZZ)V", PosRot::new);
        check_inherited!(golden, PosRot);
        golden.finish();
    }

    #[test]
    fn rot_matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Rot");
        golden.check_codec("STREAM_CODEC", || Rot::STREAM_CODEC);
        golden.check("<init>(FFZZ)V", Rot::new);
        check_inherited!(golden, Rot);
        golden.finish();
    }

    #[test]
    fn status_only_matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$StatusOnly",
        );
        golden.check_codec("STREAM_CODEC", || StatusOnly::STREAM_CODEC);
        golden.check("<init>(ZZ)V", StatusOnly::new);
        check_inherited!(golden, StatusOnly);
        golden.finish();
    }
}
