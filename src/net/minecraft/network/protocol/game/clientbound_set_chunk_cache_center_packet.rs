//! net/minecraft/network/protocol/game/ClientboundSetChunkCacheCenterPacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetChunkCacheCenterPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetChunkCacheCenterPacket::write,
            ClientboundSetChunkCacheCenterPacket::from_friendly_byte_buf,
        ))
    });

/// The chunk the server centers the client's chunks on, which it sends as the player moves into another.
#[derive(Clone, Debug)]
pub struct ClientboundSetChunkCacheCenterPacket {
    x: i32,
    z: i32,
}

impl ClientboundSetChunkCacheCenterPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetChunkCacheCenterPacket,
    > = &STREAM_CODEC;

    pub fn new(x: i32, z: i32) -> ClientboundSetChunkCacheCenterPacket {
        ClientboundSetChunkCacheCenterPacket { x, z }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundSetChunkCacheCenterPacket {
        let x = input.read_var_int();
        let z = input.read_var_int();
        ClientboundSetChunkCacheCenterPacket { x, z }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.x);
        output.write_var_int(self.z);
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetChunkCacheCenterPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_CHUNK_CACHE_CENTER
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_chunk_cache_center(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetChunkCacheCenterPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields its public constructor sets.
    encode_fields!(ClientboundSetChunkCacheCenterPacket { x: i32, z: i32 });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetChunkCacheCenterPacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundSetChunkCacheCenterPacket",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetChunkCacheCenterPacket::STREAM_CODEC
        });
        golden.check("<init>(II)V", ClientboundSetChunkCacheCenterPacket::new);
        golden.check_method("getX()I", |this: &mut This| this.get_x());
        golden.check_method("getZ()I", |this: &mut This| this.get_z());
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
