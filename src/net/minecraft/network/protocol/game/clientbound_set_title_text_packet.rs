//! net/minecraft/network/protocol/game/ClientboundSetTitleTextPacket.java
//!
//! Not ported yet: the record's `hashCode`, which hashes its components' contents and styles
//! (`MutableComponent.hashCode`, not ported).

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundSetTitleTextPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            ComponentSerialization::TRUSTED_STREAM_CODEC,
            ClientboundSetTitleTextPacket::text,
            ClientboundSetTitleTextPacket::new,
        ))
    });

#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetTitleTextPacket {
    text: Component,
}

impl ClientboundSetTitleTextPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundSetTitleTextPacket,
    > = &STREAM_CODEC;

    pub fn new(text: Component) -> ClientboundSetTitleTextPacket {
        ClientboundSetTitleTextPacket { text }
    }

    pub fn text(&self) -> Component {
        self.text.clone()
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetTitleTextPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_TITLE_TEXT
    }

    fn handle(&self, listener: &mut L) {
        listener.set_title_text(self);
    }
}

/// `ClientboundSetTitleTextPacket[text={}]`.
impl fmt::Display for ClientboundSetTitleTextPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientboundSetTitleTextPacket[text={}]", self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetTitleTextPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundSetTitleTextPacket { text: Component });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetTitleTextPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket");
        golden.pending("hashCode()I", "a component's hashCode isn't ported");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetTitleTextPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/network/chat/Component;)V",
            ClientboundSetTitleTextPacket::new,
        );
        golden.check_method(
            "text()Lnet/minecraft/network/chat/Component;",
            |this: &mut This| this.text(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
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
