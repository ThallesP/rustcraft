//! net/minecraft/network/protocol/game/ClientboundSetExperiencePacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetExperiencePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetExperiencePacket::write,
            ClientboundSetExperiencePacket::from_friendly_byte_buf,
        ))
    });

/// The player's experience: the progress to the next level, the total and the level. It writes the level before the total, the other way round from its fields.
#[derive(Clone, Debug)]
pub struct ClientboundSetExperiencePacket {
    experience_progress: f32,
    total_experience: i32,
    experience_level: i32,
}

impl ClientboundSetExperiencePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetExperiencePacket,
    > = &STREAM_CODEC;

    pub fn new(
        experience_progress: f32,
        total_experience: i32,
        experience_level: i32,
    ) -> ClientboundSetExperiencePacket {
        ClientboundSetExperiencePacket {
            experience_progress,
            total_experience,
            experience_level,
        }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundSetExperiencePacket {
        let experience_progress = input.read_float();
        let experience_level = input.read_var_int();
        let total_experience = input.read_var_int();
        ClientboundSetExperiencePacket {
            experience_progress,
            total_experience,
            experience_level,
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_float(self.experience_progress);
        output.write_var_int(self.experience_level);
        output.write_var_int(self.total_experience);
    }

    pub fn get_experience_progress(&self) -> f32 {
        self.experience_progress
    }

    pub fn get_total_experience(&self) -> i32 {
        self.total_experience
    }

    pub fn get_experience_level(&self) -> i32 {
        self.experience_level
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetExperiencePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_EXPERIENCE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_experience(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetExperiencePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields its public constructor sets.
    encode_fields!(ClientboundSetExperiencePacket {
        experience_progress: f32,
        total_experience: i32,
        experience_level: i32
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetExperiencePacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSetExperiencePacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetExperiencePacket::STREAM_CODEC
        });
        golden.check("<init>(FII)V", ClientboundSetExperiencePacket::new);
        golden.check_method("getExperienceProgress()F", |this: &mut This| {
            this.get_experience_progress()
        });
        golden.check_method("getTotalExperience()I", |this: &mut This| {
            this.get_total_experience()
        });
        golden.check_method("getExperienceLevel()I", |this: &mut This| {
            this.get_experience_level()
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
