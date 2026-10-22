//! net/minecraft/network/protocol/game/ServerboundChatCommandPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::java::lang::object::HashCode;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundChatCommandPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundChatCommandPacket::write,
            ServerboundChatCommandPacket::from_friendly_byte_buf,
        ))
    });

/// A command the player ran, without its slash, unsigned.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundChatCommandPacket {
    command: String,
}

impl ServerboundChatCommandPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundChatCommandPacket,
    > = &STREAM_CODEC;

    pub fn new(command: String) -> ServerboundChatCommandPacket {
        ServerboundChatCommandPacket { command }
    }

    /// A command of up to 32767 characters, and panics past them, where Java throws
    /// `DecoderException`.
    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ServerboundChatCommandPacket {
        ServerboundChatCommandPacket::new(input.read_utf())
    }

    /// Panics for a command past 32767 characters, where Java throws `EncoderException`.
    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_utf(&self.command);
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    /// A record's: its one component's hash.
    pub fn hash_code(&self) -> i32 {
        self.command.hash_code()
    }
}

impl<L: ServerGamePacketListener + ?Sized> Packet<L> for ServerboundChatCommandPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::SERVERBOUND_CHAT_COMMAND
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_chat_command(self);
    }
}

/// A record's: `ServerboundChatCommandPacket[command=time set 6000]`.
impl fmt::Display for ServerboundChatCommandPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerboundChatCommandPacket[command={}]", self.command)
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundChatCommandPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ServerboundChatCommandPacket { command: String });

    type Listener = dyn ServerGamePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ServerboundChatCommandPacket");
        golden.check_codec("STREAM_CODEC", || {
            ServerboundChatCommandPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Ljava/lang/String;)V",
            ServerboundChatCommandPacket::new,
        );
        golden.check_method(
            "command()Ljava/lang/String;",
            |this: &mut ServerboundChatCommandPacket| this.command().to_string(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundChatCommandPacket, other: ServerboundChatCommandPacket| {
                *this == other
            },
        );
        golden.check_method("hashCode()I", |this: &mut ServerboundChatCommandPacket| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundChatCommandPacket| this.to_string(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundChatCommandPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V",
            |this: &mut ServerboundChatCommandPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.finish();
    }
}
