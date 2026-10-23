//! net/minecraft/network/protocol/game/ClientboundChangeDifficultyPacket.java
//!
//! Its record's `hashCode` hashes the difficulty's identity hash, which changes from run to run,
//! so it isn't ported.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::difficulty::Difficulty;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundChangeDifficultyPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            Difficulty::STREAM_CODEC,
            ClientboundChangeDifficultyPacket::difficulty,
            ByteBufCodecs::BOOL,
            ClientboundChangeDifficultyPacket::locked,
            ClientboundChangeDifficultyPacket::new,
        ))
    });

/// The world's difficulty, and whether it's locked so players can't change it.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundChangeDifficultyPacket {
    difficulty: Difficulty,
    locked: bool,
}

impl ClientboundChangeDifficultyPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundChangeDifficultyPacket> =
        &STREAM_CODEC;

    pub fn new(difficulty: Difficulty, locked: bool) -> ClientboundChangeDifficultyPacket {
        ClientboundChangeDifficultyPacket { difficulty, locked }
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub fn locked(&self) -> bool {
        self.locked
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundChangeDifficultyPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_CHANGE_DIFFICULTY
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_change_difficulty(self);
    }
}

/// A record's: `ClientboundChangeDifficultyPacket[difficulty=NORMAL, locked=false]`.
impl fmt::Display for ClientboundChangeDifficultyPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundChangeDifficultyPacket[difficulty={}, locked={}]",
            self.difficulty, self.locked
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundChangeDifficultyPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::difficulty::Difficulty;

    encode_fields!(ClientboundChangeDifficultyPacket {
        difficulty: Difficulty,
        locked: bool,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundChangeDifficultyPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundChangeDifficultyPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundChangeDifficultyPacket::STREAM_CODEC
        });
        golden.pending(
            "hashCode()I",
            "hashes the difficulty's identity hash, which changes from run to run",
        );
        golden.check(
            "<init>(Lnet/minecraft/world/Difficulty;Z)V",
            ClientboundChangeDifficultyPacket::new,
        );
        golden.check_method(
            "difficulty()Lnet/minecraft/world/Difficulty;",
            |this: &mut This| this.difficulty(),
        );
        golden.check_method("locked()Z", |this: &mut This| this.locked());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
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
