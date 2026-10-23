//! net/minecraft/network/protocol/game/ClientboundSetChunkCacheRadiusPacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetChunkCacheRadiusPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetChunkCacheRadiusPacket::write,
            ClientboundSetChunkCacheRadiusPacket::from_friendly_byte_buf,
        ))
    });

/// How many chunks around its center the server sends the client.
#[derive(Clone, Debug)]
pub struct ClientboundSetChunkCacheRadiusPacket {
    radius: i32,
}

impl ClientboundSetChunkCacheRadiusPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetChunkCacheRadiusPacket,
    > = &STREAM_CODEC;

    pub fn new(radius: i32) -> ClientboundSetChunkCacheRadiusPacket {
        ClientboundSetChunkCacheRadiusPacket { radius }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundSetChunkCacheRadiusPacket {
        let radius = input.read_var_int();
        ClientboundSetChunkCacheRadiusPacket { radius }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.radius);
    }

    pub fn get_radius(&self) -> i32 {
        self.radius
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetChunkCacheRadiusPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_CHUNK_CACHE_RADIUS
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_chunk_cache_radius(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetChunkCacheRadiusPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields its public constructor sets.
    encode_fields!(ClientboundSetChunkCacheRadiusPacket { radius: i32 });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetChunkCacheRadiusPacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundSetChunkCacheRadiusPacket",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetChunkCacheRadiusPacket::STREAM_CODEC
        });
        golden.check("<init>(I)V", ClientboundSetChunkCacheRadiusPacket::new);
        golden.check_method("getRadius()I", |this: &mut This| this.get_radius());
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
