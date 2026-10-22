//! net/minecraft/network/protocol/common/ClientboundResourcePackPopPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::core::uuid_util::UUIDUtil;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundResourcePackPopPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundResourcePackPopPacket::write,
            ClientboundResourcePackPopPacket::from_friendly_byte_buf,
        ))
    });

/// A server taking back the resource pack it pushed with the id, or every one of them.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundResourcePackPopPacket {
    id: Option<UUID>,
}

impl ClientboundResourcePackPopPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundResourcePackPopPacket,
    > = &STREAM_CODEC;

    pub fn new(id: Option<UUID>) -> ClientboundResourcePackPopPacket {
        ClientboundResourcePackPopPacket { id }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundResourcePackPopPacket {
        ClientboundResourcePackPopPacket::new(
            input.read_optional(|input: &mut ByteBuf| UUIDUtil::STREAM_CODEC.decode(input)),
        )
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_optional(self.id.as_ref(), |output: &mut ByteBuf, id: &UUID| {
            UUIDUtil::STREAM_CODEC.encode(output, id)
        });
    }

    pub fn id(&self) -> Option<UUID> {
        self.id
    }

    /// A record's of one component: its hash, an `Optional`'s, which is the id's or 0.
    pub fn hash_code(&self) -> i32 {
        self.id.as_ref().map_or(0, UUID::hash_code)
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundResourcePackPopPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_POP
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_resource_pack_pop(self);
    }
}

/// A record's, with an `Optional`'s `toString`:
/// `ClientboundResourcePackPopPacket[id=Optional[…]]` or `[id=Optional.empty]`.
impl fmt::Display for ClientboundResourcePackPopPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.id {
            Some(id) => write!(f, "ClientboundResourcePackPopPacket[id=Optional[{id}]]"),
            None => f.write_str("ClientboundResourcePackPopPacket[id=Optional.empty]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundResourcePackPopPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::java::util::uuid::UUID;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ClientboundResourcePackPopPacket { id: Option<UUID> });

    type Listener = dyn ClientCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ClientboundResourcePackPopPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundResourcePackPopPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Ljava/util/Optional;)V",
            ClientboundResourcePackPopPacket::new,
        );
        golden.check_method(
            "id()Ljava/util/Optional;",
            |this: &mut ClientboundResourcePackPopPacket| this.id(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundResourcePackPopPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut ClientboundResourcePackPopPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundResourcePackPopPacket,
             other: ClientboundResourcePackPopPacket| *this == other,
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ClientboundResourcePackPopPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundResourcePackPopPacket| this.to_string(),
        );
        golden.finish();
    }
}
