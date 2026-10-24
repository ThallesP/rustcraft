//! net/minecraft/network/protocol/common/ClientboundResourcePackPushPacket.java
//!
//! Not ported yet: the record's `hashCode`, which hashes the prompt (`MutableComponent.hashCode`,
//! not ported).

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::core::uuid_util::UUIDUtil;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundResourcePackPushPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite5(
            UUIDUtil::STREAM_CODEC,
            ClientboundResourcePackPushPacket::id,
            ByteBufCodecs::STRING_UTF8,
            ClientboundResourcePackPushPacket::url,
            ByteBufCodecs::string_utf8(ClientboundResourcePackPushPacket::MAX_HASH_LENGTH),
            ClientboundResourcePackPushPacket::hash,
            ByteBufCodecs::BOOL,
            ClientboundResourcePackPushPacket::required,
            ComponentSerialization::TRUSTED_CONTEXT_FREE_STREAM_CODEC
                .apply(ByteBufCodecs::optional),
            ClientboundResourcePackPushPacket::prompt,
            ClientboundResourcePackPushPacket::new,
        ))
    });

/// A resource pack the server wants the client to use: its id, where to download it, its SHA-1
/// hash, whether the client may join without it, and what to tell the player.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundResourcePackPushPacket {
    id: UUID,
    url: String,
    hash: String,
    required: bool,
    prompt: Option<Component>,
}

impl ClientboundResourcePackPushPacket {
    pub const MAX_HASH_LENGTH: i32 = 40;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundResourcePackPushPacket> =
        &STREAM_CODEC;

    /// Panics for a hash longer than 40, where Java throws `IllegalArgumentException`.
    pub fn new(
        id: UUID,
        url: String,
        hash: String,
        required: bool,
        prompt: Option<Component>,
    ) -> ClientboundResourcePackPushPacket {
        let length = hash.encode_utf16().count();
        assert!(
            length <= 40,
            "IllegalArgumentException: Hash is too long (max 40, was {length})"
        );
        ClientboundResourcePackPushPacket {
            id,
            url,
            hash,
            required,
            prompt,
        }
    }

    pub fn id(&self) -> UUID {
        self.id
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn hash(&self) -> String {
        self.hash.clone()
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn prompt(&self) -> Option<Component> {
        self.prompt.clone()
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundResourcePackPushPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_PUSH
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_resource_pack_push(self);
    }
}

/// `ClientboundResourcePackPushPacket[id=…, url=…, hash=…, required=…, prompt=Optional[…]]`.
impl fmt::Display for ClientboundResourcePackPushPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prompt = match &self.prompt {
            Some(prompt) => format!("Optional[{prompt}]"),
            None => "Optional.empty".to_string(),
        };
        write!(
            f,
            "ClientboundResourcePackPushPacket[id={}, url={}, hash={}, required={}, prompt={prompt}]",
            self.id, self.url, self.hash, self.required
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundResourcePackPushPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::java::util::uuid::UUID;
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundResourcePackPushPacket {
        id: UUID,
        url: String,
        hash: String,
        required: bool,
        prompt: Option<Component>
    });

    type Listener = dyn ClientCommonPacketListener;
    type This = ClientboundResourcePackPushPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket");
        golden.pending("hashCode()I", "a component's hashCode isn't ported");
        golden.field(
            "MAX_HASH_LENGTH",
            ClientboundResourcePackPushPacket::MAX_HASH_LENGTH,
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundResourcePackPushPacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Ljava/util/UUID;Ljava/lang/String;Ljava/lang/String;ZLjava/util/Optional;)V",
            "an Optional argument is a Transformation's in the harness: the codec's cases make the packet",
        );
        golden.check_method("id()Ljava/util/UUID;", |this: &mut This| this.id());
        golden.check_method("url()Ljava/lang/String;", |this: &mut This| this.url());
        golden.check_method("hash()Ljava/lang/String;", |this: &mut This| this.hash());
        golden.check_method("required()Z", |this: &mut This| this.required());
        golden.check_method("prompt()Ljava/util/Optional;", |this: &mut This| {
            this.prompt()
        });
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
