//! net/minecraft/network/protocol/game/ClientboundSetTitlesAnimationPacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetTitlesAnimationPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetTitlesAnimationPacket::write,
            ClientboundSetTitlesAnimationPacket::from_friendly_byte_buf,
        ))
    });

/// How long titles fade in, stay and fade out, in ticks; a negative time keeps the one before.
#[derive(Clone, Debug)]
pub struct ClientboundSetTitlesAnimationPacket {
    fade_in: i32,
    stay: i32,
    fade_out: i32,
}

impl ClientboundSetTitlesAnimationPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetTitlesAnimationPacket,
    > = &STREAM_CODEC;

    pub fn new(fade_in: i32, stay: i32, fade_out: i32) -> ClientboundSetTitlesAnimationPacket {
        ClientboundSetTitlesAnimationPacket {
            fade_in,
            stay,
            fade_out,
        }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundSetTitlesAnimationPacket {
        let fade_in = input.read_int();
        let stay = input.read_int();
        let fade_out = input.read_int();
        ClientboundSetTitlesAnimationPacket {
            fade_in,
            stay,
            fade_out,
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_int(self.fade_in);
        output.write_int(self.stay);
        output.write_int(self.fade_out);
    }

    pub fn get_fade_in(&self) -> i32 {
        self.fade_in
    }

    pub fn get_stay(&self) -> i32 {
        self.stay
    }

    pub fn get_fade_out(&self) -> i32 {
        self.fade_out
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetTitlesAnimationPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_TITLES_ANIMATION
    }

    fn handle(&self, listener: &mut L) {
        listener.set_titles_animation(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetTitlesAnimationPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundSetTitlesAnimationPacket {
        fade_in: i32,
        stay: i32,
        fade_out: i32
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetTitlesAnimationPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetTitlesAnimationPacket::STREAM_CODEC
        });
        golden.check("<init>(III)V", ClientboundSetTitlesAnimationPacket::new);
        golden.check_method("getFadeIn()I", |this: &mut This| this.get_fade_in());
        golden.check_method("getStay()I", |this: &mut This| this.get_stay());
        golden.check_method("getFadeOut()I", |this: &mut This| this.get_fade_out());
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
