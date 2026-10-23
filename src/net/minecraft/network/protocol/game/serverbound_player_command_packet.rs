//! net/minecraft/network/protocol/game/ServerboundPlayerCommandPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::entity::entity::EntityClass;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundPlayerCommandPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundPlayerCommandPacket::write,
            ServerboundPlayerCommandPacket::from_friendly_byte_buf,
        ))
    });

/// Something the player did that the server can't see from its moves: sprinting, leaving a bed,
/// a horse's jump, opening its horse's inventory or gliding. The entity is the player, or what it
/// rides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ServerboundPlayerCommandPacket {
    id: i32,
    action: Action,
    data: i32,
}

impl ServerboundPlayerCommandPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundPlayerCommandPacket,
    > = &STREAM_CODEC;

    /// `ServerboundPlayerCommandPacket(Entity, Action)`, with no data.
    pub fn new(
        entity: &(impl EntityClass + ?Sized),
        action: Action,
    ) -> ServerboundPlayerCommandPacket {
        ServerboundPlayerCommandPacket::new_with_data(entity, action, 0)
    }

    pub fn new_with_data(
        entity: &(impl EntityClass + ?Sized),
        action: Action,
        data: i32,
    ) -> ServerboundPlayerCommandPacket {
        ServerboundPlayerCommandPacket {
            id: entity.entity().get_id(),
            action,
            data,
        }
    }

    /// The action by its ordinal, which panics past the last, where Java throws
    /// `ArrayIndexOutOfBoundsException`.
    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ServerboundPlayerCommandPacket {
        let id = input.read_var_int();
        let action = input.read_enum::<Action>();
        ServerboundPlayerCommandPacket {
            id,
            action,
            data: input.read_var_int(),
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.id);
        output.write_enum(&self.action);
        output.write_var_int(self.data);
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_action(&self) -> Action {
        self.action
    }

    pub fn get_data(&self) -> i32 {
        self.data
    }
}

impl<L: ServerGamePacketListener + ?Sized> Packet<L> for ServerboundPlayerCommandPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::SERVERBOUND_PLAYER_COMMAND
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_player_command(self);
    }
}

/// `ServerboundPlayerCommandPacket.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    STOP_SLEEPING,
    START_SPRINTING,
    STOP_SPRINTING,
    START_RIDING_JUMP,
    STOP_RIDING_JUMP,
    OPEN_INVENTORY,
    START_FALL_FLYING,
}

impl Action {
    pub const VALUES: [Action; 7] = [
        Action::STOP_SLEEPING,
        Action::START_SPRINTING,
        Action::STOP_SPRINTING,
        Action::START_RIDING_JUMP,
        Action::STOP_RIDING_JUMP,
        Action::OPEN_INVENTORY,
        Action::START_FALL_FLYING,
    ];

    pub fn values() -> [Action; 7] {
        Action::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Action {
        Action::VALUES
            .into_iter()
            .find(|action| format!("{action:?}") == name)
            .unwrap_or_else(|| panic!("IllegalArgumentException: no Action constant {name}"))
    }
}

impl Enum for Action {
    const ENUM_CONSTANTS: &'static [Action] = &Action::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, ServerboundPlayerCommandPacket};
    use crate::golden::{Golden, encode_fields, encode_names};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // Its fields, which the reader sets.
    encode_fields!(ServerboundPlayerCommandPacket {
        id: i32,
        action: Action,
        data: i32,
    });

    encode_names!(Action {
        STOP_SLEEPING,
        START_SPRINTING,
        STOP_SPRINTING,
        START_RIDING_JUMP,
        STOP_RIDING_JUMP,
        OPEN_INVENTORY,
        START_FALL_FLYING
    });

    type Listener = dyn ServerGamePacketListener;
    type This = ServerboundPlayerCommandPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket");
        golden.check_codec("STREAM_CODEC", || {
            ServerboundPlayerCommandPacket::STREAM_CODEC
        });
        for constructor in [
            "<init>(Lnet/minecraft/world/entity/Entity;Lnet/minecraft/network/protocol/game/ServerboundPlayerCommandPacket$Action;)V",
            "<init>(Lnet/minecraft/world/entity/Entity;Lnet/minecraft/network/protocol/game/ServerboundPlayerCommandPacket$Action;I)V",
        ] {
            golden.pending(
                constructor,
                "takes an entity, which the harness has no generator for: the trace's packets check it",
            );
        }
        golden.check_method("getId()I", |this: &mut This| this.get_id());
        golden.check_method(
            "getAction()Lnet/minecraft/network/protocol/game/ServerboundPlayerCommandPacket$Action;",
            |this: &mut This| this.get_action(),
        );
        golden.check_method("getData()I", |this: &mut This| this.get_data());
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

    #[test]
    fn action_matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket$Action",
        );
        for action in Action::VALUES {
            golden.field(&format!("{action:?}"), action);
        }
        golden.check(
            "values()[Lnet/minecraft/network/protocol/game/ServerboundPlayerCommandPacket$Action;",
            Action::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/network/protocol/game/ServerboundPlayerCommandPacket$Action;",
            |name: String| Action::value_of(&name),
        );
        golden.finish();
    }
}
