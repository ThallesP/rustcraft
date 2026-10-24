//! net/minecraft/network/protocol/common/ClientboundServerLinksPacket.java
//!
//! Not ported yet: the record's `hashCode`, which hashes the links' names (`MutableComponent.hashCode`,
//! not ported).

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::server::server_links::{ServerLinks, UntrustedEntry};

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundServerLinksPacket> =
    LazyLock::new(|| {
        Box::new(ServerLinks::UNTRUSTED_LINKS_STREAM_CODEC.map(
            ClientboundServerLinksPacket::new,
            ClientboundServerLinksPacket::links,
        ))
    });

/// The links the server lists for its players, as it sent them.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundServerLinksPacket {
    links: Vec<UntrustedEntry>,
}

impl ClientboundServerLinksPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundServerLinksPacket> =
        &STREAM_CODEC;

    pub fn new(links: Vec<UntrustedEntry>) -> ClientboundServerLinksPacket {
        ClientboundServerLinksPacket { links }
    }

    pub fn links(&self) -> Vec<UntrustedEntry> {
        self.links.clone()
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundServerLinksPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_SERVER_LINKS
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_server_links(self);
    }
}

/// `ClientboundServerLinksPacket[links=[…, …]]`.
impl fmt::Display for ClientboundServerLinksPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let links: Vec<String> = self.links.iter().map(ToString::to_string).collect();
        write!(
            f,
            "ClientboundServerLinksPacket[links=[{}]]",
            links.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ClientboundServerLinksPacket;
    use crate::com::mojang::datafixers::util::either::Either;
    use crate::golden::{Decode, Encode, Golden, elements, encode_fields, encode_names};
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::server::server_links::{KnownLinkType, UntrustedEntry};

    encode_names!(KnownLinkType {
        BUG_REPORT,
        COMMUNITY_GUIDELINES,
        SUPPORT,
        STATUS,
        FEEDBACK,
        COMMUNITY,
        WEBSITE,
        FORUMS,
        NEWS,
        ANNOUNCEMENTS
    });

    /// `[type, link]`, a record's components.
    impl Encode for UntrustedEntry {
        fn encode(&self) -> Value {
            Value::from(vec![self.r#type.encode(), self.link.encode()])
        }
    }

    impl Decode for UntrustedEntry {
        fn decode(value: &Value) -> Self {
            let [r#type, link] = elements(value);
            UntrustedEntry {
                r#type: Either::<KnownLinkType, Component>::decode(r#type),
                link: String::decode(link),
            }
        }
    }

    encode_fields!(ClientboundServerLinksPacket {
        links: Vec<UntrustedEntry>
    });

    type Listener = dyn ClientCommonPacketListener;
    type This = ClientboundServerLinksPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ClientboundServerLinksPacket");
        golden.pending("hashCode()I", "a component's hashCode isn't ported");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundServerLinksPacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Ljava/util/List;)V",
            "the harness has no generator for a list argument; the codec's cases make the packet",
        );
        golden.check_method("links()Ljava/util/List;", |this: &mut This| this.links());
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
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
