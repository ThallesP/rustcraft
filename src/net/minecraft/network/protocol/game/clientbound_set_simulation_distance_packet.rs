//! net/minecraft/network/protocol/game/ClientboundSetSimulationDistancePacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetSimulationDistancePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetSimulationDistancePacket::write,
            ClientboundSetSimulationDistancePacket::from_friendly_byte_buf,
        ))
    });

/// How far from the player the server ticks the world, in chunks.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetSimulationDistancePacket {
    simulation_distance: i32,
}

impl ClientboundSetSimulationDistancePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetSimulationDistancePacket,
    > = &STREAM_CODEC;

    pub fn new(simulation_distance: i32) -> ClientboundSetSimulationDistancePacket {
        ClientboundSetSimulationDistancePacket {
            simulation_distance,
        }
    }

    fn from_friendly_byte_buf(
        input: &mut FriendlyByteBuf,
    ) -> ClientboundSetSimulationDistancePacket {
        ClientboundSetSimulationDistancePacket::new(input.read_var_int())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.simulation_distance);
    }

    pub fn simulation_distance(&self) -> i32 {
        self.simulation_distance
    }

    /// A record's: its one component's hash.
    pub fn hash_code(&self) -> i32 {
        self.simulation_distance
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetSimulationDistancePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_SIMULATION_DISTANCE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_simulation_distance(self);
    }
}

/// A record's: `ClientboundSetSimulationDistancePacket[simulationDistance=4]`.
impl fmt::Display for ClientboundSetSimulationDistancePacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundSetSimulationDistancePacket[simulationDistance={}]",
            self.simulation_distance
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetSimulationDistancePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundSetSimulationDistancePacket {
        simulation_distance: i32
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetSimulationDistancePacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundSetSimulationDistancePacket",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetSimulationDistancePacket::STREAM_CODEC
        });
        golden.check("<init>(I)V", ClientboundSetSimulationDistancePacket::new);
        golden.check_method("simulationDistance()I", |this: &mut This| {
            this.simulation_distance()
        });
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
