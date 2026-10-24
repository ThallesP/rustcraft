//! net/minecraft/network/protocol/login/ClientboundLoginDisconnectPacket.java
//!
//! Not ported yet: the record's `hashCode`, which hashes the reason's contents and style
//! (`MutableComponent.hashCode`, not ported).

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::SharedCodec;
use crate::com::mojang::serialization::json_ops::JsonOps;
use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::registry_ops::RegistryOps;

/// `RegistryAccess.EMPTY`, whose serialization context `OPS` is.
static EMPTY: LazyLock<RegistryAccess> = LazyLock::new(|| RegistryAccess::new(Vec::new()));

// `lenientJson(262144).apply(fromCodec(OPS, ComponentSerialization.CODEC))`, `OPS` being JSON's
// ops over no registries: the reason as JSON text, the login's only component.
static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundLoginDisconnectPacket> =
    LazyLock::new(|| {
        Box::new(
            ByteBufCodecs::from_codec_ops(
                || RegistryOps::create(&JsonOps::INSTANCE, &EMPTY),
                ComponentSerialization::CODEC.shared(),
                ByteBufCodecs::lenient_json(262144),
            )
            .map(
                ClientboundLoginDisconnectPacket::new,
                ClientboundLoginDisconnectPacket::reason,
            ),
        )
    });

/// Why the server's ending the connection while the client logs in.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundLoginDisconnectPacket {
    reason: Component,
}

impl ClientboundLoginDisconnectPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundLoginDisconnectPacket> =
        &STREAM_CODEC;

    pub fn new(reason: Component) -> ClientboundLoginDisconnectPacket {
        ClientboundLoginDisconnectPacket { reason }
    }

    pub fn reason(&self) -> Component {
        self.reason.clone()
    }
}

impl<L: ClientLoginPacketListener + ?Sized> Packet<L> for ClientboundLoginDisconnectPacket {
    fn r#type(&self) -> &'static PacketType {
        LoginPacketTypes::CLIENTBOUND_LOGIN_DISCONNECT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_disconnect(self);
    }
}

/// `ClientboundLoginDisconnectPacket[reason=…]`.
impl fmt::Display for ClientboundLoginDisconnectPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundLoginDisconnectPacket[reason={}]",
            self.reason
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundLoginDisconnectPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundLoginDisconnectPacket { reason: Component });

    type Listener = dyn ClientLoginPacketListener;
    type This = ClientboundLoginDisconnectPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.login.ClientboundLoginDisconnectPacket");
        golden.pending("hashCode()I", "a component's hashCode isn't ported");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundLoginDisconnectPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/network/chat/Component;)V",
            ClientboundLoginDisconnectPacket::new,
        );
        golden.check_method(
            "reason()Lnet/minecraft/network/chat/Component;",
            |this: &mut This| this.reason(),
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
            "handle(Lnet/minecraft/network/protocol/login/ClientLoginPacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
