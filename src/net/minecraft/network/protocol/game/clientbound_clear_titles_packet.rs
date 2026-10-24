//! net/minecraft/network/protocol/game/ClientboundClearTitlesPacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundClearTitlesPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundClearTitlesPacket::write,
            ClientboundClearTitlesPacket::from_friendly_byte_buf,
        ))
    });

/// Takes the title and subtitle away, and puts their times back to the defaults if asked to.
#[derive(Clone, Debug)]
pub struct ClientboundClearTitlesPacket {
    reset_times: bool,
}

impl ClientboundClearTitlesPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundClearTitlesPacket,
    > = &STREAM_CODEC;

    pub fn new(reset_times: bool) -> ClientboundClearTitlesPacket {
        ClientboundClearTitlesPacket { reset_times }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundClearTitlesPacket {
        ClientboundClearTitlesPacket {
            reset_times: input.read_boolean(),
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_boolean(self.reset_times);
    }

    pub fn should_reset_times(&self) -> bool {
        self.reset_times
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundClearTitlesPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_CLEAR_TITLES
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_titles_clear(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundClearTitlesPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundClearTitlesPacket { reset_times: bool });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundClearTitlesPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundClearTitlesPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundClearTitlesPacket::STREAM_CODEC
        });
        golden.check("<init>(Z)V", ClientboundClearTitlesPacket::new);
        golden.check_method("shouldResetTimes()Z", |this: &mut This| {
            this.should_reset_times()
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
