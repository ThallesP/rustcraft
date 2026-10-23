//! net/minecraft/network/protocol/game/ClientboundTickingStatePacket.java
//!
//! Not ported yet: `from(TickRateManager)`, which the server sends it with.

use std::sync::LazyLock;

use crate::java::lang::float::Float;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use std::fmt;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundTickingStatePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundTickingStatePacket::write,
            ClientboundTickingStatePacket::from_friendly_byte_buf,
        ))
    });

/// The server's tick rate (`/tick rate`), and whether its ticking is frozen (`/tick freeze`),
/// which the client's own ticks follow.
#[derive(Clone, Debug)]
pub struct ClientboundTickingStatePacket {
    tick_rate: f32,
    is_frozen: bool,
}

impl ClientboundTickingStatePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundTickingStatePacket,
    > = &STREAM_CODEC;

    pub fn new(tick_rate: f32, is_frozen: bool) -> ClientboundTickingStatePacket {
        ClientboundTickingStatePacket {
            tick_rate,
            is_frozen,
        }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundTickingStatePacket {
        let tick_rate = input.read_float();
        ClientboundTickingStatePacket::new(tick_rate, input.read_boolean())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_float(self.tick_rate);
        output.write_boolean(self.is_frozen);
    }

    pub fn tick_rate(&self) -> f32 {
        self.tick_rate
    }

    pub fn is_frozen(&self) -> bool {
        self.is_frozen
    }

    /// A record's: each component's hash (`Float.hashCode`, and `Boolean.hashCode`'s 1231 or
    /// 1237), added to 31 times the ones before.
    pub fn hash_code(&self) -> i32 {
        Float::hash_code(self.tick_rate)
            .wrapping_mul(31)
            .wrapping_add(if self.is_frozen { 1231 } else { 1237 })
    }
}

/// A record's equals compares a float component with `Float.compare`, so by `floatToIntBits`: NaN
/// equals NaN, and -0.0 isn't 0.0.
impl PartialEq for ClientboundTickingStatePacket {
    fn eq(&self, other: &ClientboundTickingStatePacket) -> bool {
        Float::float_to_int_bits(self.tick_rate) == Float::float_to_int_bits(other.tick_rate)
            && self.is_frozen == other.is_frozen
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundTickingStatePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_TICKING_STATE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_ticking_state(self);
    }
}

/// A record's: `ClientboundTickingStatePacket[tickRate=20.0, isFrozen=false]`.
impl fmt::Display for ClientboundTickingStatePacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundTickingStatePacket[tickRate={}, isFrozen={}]",
            Float::to_string(self.tick_rate),
            self.is_frozen
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundTickingStatePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundTickingStatePacket {
        tick_rate: f32,
        is_frozen: bool,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundTickingStatePacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundTickingStatePacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundTickingStatePacket::STREAM_CODEC
        });
        golden.pending(
            "from(Lnet/minecraft/world/TickRateManager;)Lnet/minecraft/network/protocol/game/ClientboundTickingStatePacket;",
            "reads a TickRateManager, which isn't ported",
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
        });
        golden.check("<init>(FZ)V", ClientboundTickingStatePacket::new);
        golden.check_method("tickRate()F", |this: &mut This| this.tick_rate());
        golden.check_method("isFrozen()Z", |this: &mut This| this.is_frozen());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut This| this.hash_code());
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
