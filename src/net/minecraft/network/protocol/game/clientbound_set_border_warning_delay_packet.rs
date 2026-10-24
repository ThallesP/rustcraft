//! net/minecraft/network/protocol/game/ClientboundSetBorderWarningDelayPacket.java
//!
//! Not ported yet: its public constructor, which reads a server's `WorldBorder` (`getWarningTime`);
//! the client only reads the packet.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetBorderWarningDelayPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetBorderWarningDelayPacket::write,
            ClientboundSetBorderWarningDelayPacket::from_friendly_byte_buf,
        ))
    });

/// How many seconds ahead of a shrinking border the warning shows.
#[derive(Clone, Debug)]
pub struct ClientboundSetBorderWarningDelayPacket {
    warning_delay: i32,
}

impl ClientboundSetBorderWarningDelayPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetBorderWarningDelayPacket,
    > = &STREAM_CODEC;

    fn from_friendly_byte_buf(
        input: &mut FriendlyByteBuf,
    ) -> ClientboundSetBorderWarningDelayPacket {
        let warning_delay = input.read_var_int();
        ClientboundSetBorderWarningDelayPacket { warning_delay }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.warning_delay);
    }

    pub fn get_warning_delay(&self) -> i32 {
        self.warning_delay
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetBorderWarningDelayPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_BORDER_WARNING_DELAY
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_border_warning_delay(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetBorderWarningDelayPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields it reads.
    encode_fields!(ClientboundSetBorderWarningDelayPacket { warning_delay: i32 });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetBorderWarningDelayPacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundSetBorderWarningDelayPacket",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetBorderWarningDelayPacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Lnet/minecraft/world/level/border/WorldBorder;)V",
            "reads a server's WorldBorder; the client only reads the packet",
        );
        golden.check_method("getWarningDelay()I", |this: &mut This| {
            this.get_warning_delay()
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
