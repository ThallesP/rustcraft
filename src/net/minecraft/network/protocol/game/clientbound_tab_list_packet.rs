//! net/minecraft/network/protocol/game/ClientboundTabListPacket.java
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

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundTabListPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            ComponentSerialization::TRUSTED_STREAM_CODEC,
            ClientboundTabListPacket::header,
            ComponentSerialization::TRUSTED_STREAM_CODEC,
            ClientboundTabListPacket::footer,
            ClientboundTabListPacket::new,
        ))
    });

#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundTabListPacket {
    header: Component,
    footer: Component,
}

impl ClientboundTabListPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundTabListPacket,
    > = &STREAM_CODEC;

    pub fn new(header: Component, footer: Component) -> ClientboundTabListPacket {
        ClientboundTabListPacket { header, footer }
    }

    pub fn header(&self) -> Component {
        self.header.clone()
    }

    pub fn footer(&self) -> Component {
        self.footer.clone()
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundTabListPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_TAB_LIST
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_tab_list_customisation(self);
    }
}

/// `ClientboundTabListPacket[header={}, footer={}]`.
impl fmt::Display for ClientboundTabListPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundTabListPacket[header={}, footer={}]",
            self.header, self.footer
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundTabListPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundTabListPacket {
        header: Component,
        footer: Component,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundTabListPacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.ClientboundTabListPacket");
        golden.pending("hashCode()I", "a component's hashCode isn't ported");
        golden.check_codec("STREAM_CODEC", || ClientboundTabListPacket::STREAM_CODEC);
        golden.check(
            "<init>(Lnet/minecraft/network/chat/Component;Lnet/minecraft/network/chat/Component;)V",
            ClientboundTabListPacket::new,
        );
        golden.check_method(
            "header()Lnet/minecraft/network/chat/Component;",
            |this: &mut This| this.header(),
        );
        golden.check_method(
            "footer()Lnet/minecraft/network/chat/Component;",
            |this: &mut This| this.footer(),
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
