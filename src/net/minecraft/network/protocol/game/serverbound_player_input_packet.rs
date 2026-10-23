//! net/minecraft/network/protocol/game/ServerboundPlayerInputPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::entity::player::input::Input;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundPlayerInputPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            Input::STREAM_CODEC,
            |packet: &ServerboundPlayerInputPacket| packet.input,
            ServerboundPlayerInputPacket::new,
        ))
    });

/// The movement keys the player holds, which the client sends when they change
/// (`LocalPlayer.sendChanges`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ServerboundPlayerInputPacket {
    input: Input,
}

impl ServerboundPlayerInputPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundPlayerInputPacket,
    > = &STREAM_CODEC;

    pub fn new(input: Input) -> ServerboundPlayerInputPacket {
        ServerboundPlayerInputPacket { input }
    }

    pub fn input(&self) -> Input {
        self.input
    }

    /// A record's, of its one component: 31 times 0, plus the input's.
    pub fn hash_code(&self) -> i32 {
        self.input.hash_code()
    }
}

impl<L: ServerGamePacketListener + ?Sized> Packet<L> for ServerboundPlayerInputPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::SERVERBOUND_PLAYER_INPUT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_player_input(self);
    }
}

/// A record's: `ServerboundPlayerInputPacket[input=Input[…]]`.
impl fmt::Display for ServerboundPlayerInputPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerboundPlayerInputPacket[input={}]", self.input)
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundPlayerInputPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::entity::player::input::Input;

    // A record: its component.
    encode_fields!(ServerboundPlayerInputPacket { input: Input });

    type Listener = dyn ServerGamePacketListener;
    type This = ServerboundPlayerInputPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ServerboundPlayerInputPacket");
        golden.check_codec("STREAM_CODEC", || {
            ServerboundPlayerInputPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/world/entity/player/Input;)V",
            ServerboundPlayerInputPacket::new,
        );
        golden.check_method(
            "input()Lnet/minecraft/world/entity/player/Input;",
            |this: &mut This| this.input(),
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
            "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
