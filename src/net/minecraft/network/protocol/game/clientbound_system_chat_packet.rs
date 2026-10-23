//! net/minecraft/network/protocol/game/ClientboundSystemChatPacket.java
//!
//! Not ported yet: the record's `hashCode`, which hashes its components' contents and styles
//! (`MutableComponent.hashCode`, not ported).

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundSystemChatPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            ComponentSerialization::TRUSTED_STREAM_CODEC,
            ClientboundSystemChatPacket::content,
            ByteBufCodecs::BOOL,
            ClientboundSystemChatPacket::overlay,
            ClientboundSystemChatPacket::new,
        ))
    });

#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSystemChatPacket {
    content: Component,
    overlay: bool,
}

impl ClientboundSystemChatPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundSystemChatPacket,
    > = &STREAM_CODEC;

    pub fn new(content: Component, overlay: bool) -> ClientboundSystemChatPacket {
        ClientboundSystemChatPacket { content, overlay }
    }

    pub fn content(&self) -> Component {
        self.content.clone()
    }

    pub fn overlay(&self) -> bool {
        self.overlay
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSystemChatPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SYSTEM_CHAT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_system_chat(self);
    }

    fn is_skippable(&self) -> bool {
        true
    }
}

/// `ClientboundSystemChatPacket[content={}, overlay={}]`.
impl fmt::Display for ClientboundSystemChatPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundSystemChatPacket[content={}, overlay={}]",
            self.content, self.overlay
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSystemChatPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundSystemChatPacket {
        content: Component,
        overlay: bool,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSystemChatPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSystemChatPacket");
        golden.pending("hashCode()I", "a component's hashCode isn't ported");
        golden.check_codec("STREAM_CODEC", || ClientboundSystemChatPacket::STREAM_CODEC);
        golden.check(
            "<init>(Lnet/minecraft/network/chat/Component;Z)V",
            ClientboundSystemChatPacket::new,
        );
        golden.check_method(
            "content()Lnet/minecraft/network/chat/Component;",
            |this: &mut This| this.content(),
        );
        golden.check_method("overlay()Z", |this: &mut This| this.overlay());
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
        golden.check_method("isSkippable()Z", |this: &mut This| {
            Packet::<Listener>::is_skippable(this)
        });
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
