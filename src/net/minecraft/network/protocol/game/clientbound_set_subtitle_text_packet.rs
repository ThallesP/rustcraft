//! net/minecraft/network/protocol/game/ClientboundSetSubtitleTextPacket.java
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

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundSetSubtitleTextPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            ComponentSerialization::TRUSTED_STREAM_CODEC,
            ClientboundSetSubtitleTextPacket::text,
            ClientboundSetSubtitleTextPacket::new,
        ))
    });

#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetSubtitleTextPacket {
    text: Component,
}

impl ClientboundSetSubtitleTextPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundSetSubtitleTextPacket,
    > = &STREAM_CODEC;

    pub fn new(text: Component) -> ClientboundSetSubtitleTextPacket {
        ClientboundSetSubtitleTextPacket { text }
    }

    pub fn text(&self) -> Component {
        self.text.clone()
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetSubtitleTextPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_SUBTITLE_TEXT
    }

    fn handle(&self, listener: &mut L) {
        listener.set_subtitle_text(self);
    }
}

/// `ClientboundSetSubtitleTextPacket[text={}]`.
impl fmt::Display for ClientboundSetSubtitleTextPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientboundSetSubtitleTextPacket[text={}]", self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetSubtitleTextPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundSetSubtitleTextPacket { text: Component });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetSubtitleTextPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSetSubtitleTextPacket");
        golden.pending("hashCode()I", "a component's hashCode isn't ported");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetSubtitleTextPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/network/chat/Component;)V",
            ClientboundSetSubtitleTextPacket::new,
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
