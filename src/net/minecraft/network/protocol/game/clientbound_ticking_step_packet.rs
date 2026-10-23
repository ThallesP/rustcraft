//! net/minecraft/network/protocol/game/ClientboundTickingStepPacket.java
//!
//! Not ported yet: `from(TickRateManager)`, which the server sends it with.

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundTickingStepPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundTickingStepPacket::write,
            ClientboundTickingStepPacket::from_friendly_byte_buf,
        ))
    });

/// How many ticks a frozen server steps (`/tick step`), which the client steps too.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundTickingStepPacket {
    tick_steps: i32,
}

impl ClientboundTickingStepPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundTickingStepPacket,
    > = &STREAM_CODEC;

    pub fn new(tick_steps: i32) -> ClientboundTickingStepPacket {
        ClientboundTickingStepPacket { tick_steps }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundTickingStepPacket {
        ClientboundTickingStepPacket::new(input.read_var_int())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.tick_steps);
    }

    pub fn tick_steps(&self) -> i32 {
        self.tick_steps
    }

    /// A record's: its one component's hash.
    pub fn hash_code(&self) -> i32 {
        self.tick_steps
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundTickingStepPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_TICKING_STEP
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_ticking_step(self);
    }
}

/// A record's: `ClientboundTickingStepPacket[tickSteps=4]`.
impl fmt::Display for ClientboundTickingStepPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundTickingStepPacket[tickSteps={}]",
            self.tick_steps
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundTickingStepPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundTickingStepPacket { tick_steps: i32 });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundTickingStepPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundTickingStepPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundTickingStepPacket::STREAM_CODEC
        });
        golden.pending(
            "from(Lnet/minecraft/world/TickRateManager;)Lnet/minecraft/network/protocol/game/ClientboundTickingStepPacket;",
            "reads a TickRateManager, which isn't ported",
        );
        golden.check("<init>(I)V", ClientboundTickingStepPacket::new);
        golden.check_method("tickSteps()I", |this: &mut This| this.tick_steps());
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
