//! net/minecraft/network/protocol/game/ClientboundSetHealthPacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetHealthPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetHealthPacket::write,
            ClientboundSetHealthPacket::from_friendly_byte_buf,
        ))
    });

/// The player's health, food and saturation.
#[derive(Clone, Debug)]
pub struct ClientboundSetHealthPacket {
    health: f32,
    food: i32,
    saturation: f32,
}

impl ClientboundSetHealthPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetHealthPacket,
    > = &STREAM_CODEC;

    pub fn new(health: f32, food: i32, saturation: f32) -> ClientboundSetHealthPacket {
        ClientboundSetHealthPacket {
            health,
            food,
            saturation,
        }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundSetHealthPacket {
        let health = input.read_float();
        let food = input.read_var_int();
        let saturation = input.read_float();
        ClientboundSetHealthPacket {
            health,
            food,
            saturation,
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_float(self.health);
        output.write_var_int(self.food);
        output.write_float(self.saturation);
    }

    pub fn get_health(&self) -> f32 {
        self.health
    }

    pub fn get_food(&self) -> i32 {
        self.food
    }

    pub fn get_saturation(&self) -> f32 {
        self.saturation
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetHealthPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_HEALTH
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_health(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetHealthPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields its public constructor sets.
    encode_fields!(ClientboundSetHealthPacket {
        health: f32,
        food: i32,
        saturation: f32
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetHealthPacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.ClientboundSetHealthPacket");
        golden.check_codec("STREAM_CODEC", || ClientboundSetHealthPacket::STREAM_CODEC);
        golden.check("<init>(FIF)V", ClientboundSetHealthPacket::new);
        golden.check_method("getHealth()F", |this: &mut This| this.get_health());
        golden.check_method("getFood()I", |this: &mut This| this.get_food());
        golden.check_method("getSaturation()F", |this: &mut This| this.get_saturation());
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
