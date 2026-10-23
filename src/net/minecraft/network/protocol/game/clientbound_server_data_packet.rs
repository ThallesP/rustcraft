//! net/minecraft/network/protocol/game/ClientboundServerDataPacket.java
//!
//! Not ported yet: the record's `equals`, `hashCode` and `toString`, which compare, hash and print
//! the icon's array by identity, like the cookie packets'.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundServerDataPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            ComponentSerialization::TRUSTED_CONTEXT_FREE_STREAM_CODEC,
            ClientboundServerDataPacket::motd,
            ByteBufCodecs::optional(ByteBufCodecs::BYTE_ARRAY),
            ClientboundServerDataPacket::icon_bytes,
            ClientboundServerDataPacket::new,
        ))
    });

/// The server's message of the day and icon, as the server list shows them.
#[derive(Clone, Debug)]
pub struct ClientboundServerDataPacket {
    motd: Component,
    icon_bytes: Option<Vec<i8>>,
}

impl ClientboundServerDataPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundServerDataPacket> =
        &STREAM_CODEC;

    pub fn new(motd: Component, icon_bytes: Option<Vec<i8>>) -> ClientboundServerDataPacket {
        ClientboundServerDataPacket { motd, icon_bytes }
    }

    pub fn motd(&self) -> Component {
        self.motd.clone()
    }

    pub fn icon_bytes(&self) -> Option<Vec<i8>> {
        self.icon_bytes.clone()
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundServerDataPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SERVER_DATA
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_server_data(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundServerDataPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundServerDataPacket {
        motd: Component,
        icon_bytes: Option<Vec<i8>>,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundServerDataPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundServerDataPacket");
        for name in [
            "equals(Ljava/lang/Object;)Z",
            "hashCode()I",
            "toString()Ljava/lang/String;",
        ] {
            golden.pending(
                name,
                "compares, hashes or prints the icon's array by identity",
            );
        }
        golden.check_codec("STREAM_CODEC", || ClientboundServerDataPacket::STREAM_CODEC);
        golden.pending(
            "<init>(Lnet/minecraft/network/chat/Component;Ljava/util/Optional;)V",
            "the harness's Optional argument is a Transformation's, not an icon's bytes",
        );
        golden.check_method(
            "motd()Lnet/minecraft/network/chat/Component;",
            |this: &mut This| this.motd(),
        );
        golden.check_method("iconBytes()Ljava/util/Optional;", |this: &mut This| {
            this.icon_bytes()
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
