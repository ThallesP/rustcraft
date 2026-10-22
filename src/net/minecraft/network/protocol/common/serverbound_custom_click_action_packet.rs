//! net/minecraft/network/protocol/common/ServerboundCustomClickActionPacket.java
//!
//! Not ported yet: `toString`, a record's, which prints the tag with `Tag.toString` (SNBT).

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::object::HashCode;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

// A tag read with 32 KiB and 16 levels to spend, behind a length prefix of at most 65536 bytes.
static UNTRUSTED_TAG_CODEC: StaticStreamCodec<ByteBuf, Option<Tag>> = LazyLock::new(|| {
    Box::new(
        ByteBufCodecs::optional_tag_codec(|| NbtAccounter::new(32768, 16))
            .apply(ByteBufCodecs::length_prefixed(65536)),
    )
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ServerboundCustomClickActionPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            Identifier::STREAM_CODEC,
            |packet: &ServerboundCustomClickActionPacket| packet.id().clone(),
            &UNTRUSTED_TAG_CODEC,
            |packet: &ServerboundCustomClickActionPacket| packet.payload().cloned(),
            ServerboundCustomClickActionPacket::new,
        ))
    });

/// A click on a dialog's custom action: the action's identifier, and the tag it carries, if any.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundCustomClickActionPacket {
    id: Identifier,
    payload: Option<Tag>,
}

impl ServerboundCustomClickActionPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        ServerboundCustomClickActionPacket,
    > = &STREAM_CODEC;

    pub fn new(id: Identifier, payload: Option<Tag>) -> ServerboundCustomClickActionPacket {
        ServerboundCustomClickActionPacket { id, payload }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn payload(&self) -> Option<&Tag> {
        self.payload.as_ref()
    }

    /// A record's: each component's hash, added to 31 times the ones before. An empty
    /// `Optional` hashes to 0.
    pub fn hash_code(&self) -> i32 {
        self.id
            .hash_code()
            .wrapping_mul(31)
            .wrapping_add(self.payload.as_ref().map_or(0, Tag::hash_code))
    }
}

impl<L: ServerCommonPacketListener + ?Sized> Packet<L> for ServerboundCustomClickActionPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::SERVERBOUND_CUSTOM_CLICK_ACTION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_custom_click_action(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundCustomClickActionPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components.
    encode_fields!(ServerboundCustomClickActionPacket {
        id: Identifier,
        payload: Option<Tag>,
    });

    type Listener = dyn ServerCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket",
        );
        golden.pending(
            "toString()Ljava/lang/String;",
            "prints the tag with Tag.toString, which needs StringTagVisitor's SNBT",
        );
        golden.check_codec("STREAM_CODEC", || {
            ServerboundCustomClickActionPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/resources/Identifier;Ljava/util/Optional;)V",
            ServerboundCustomClickActionPacket::new,
        );
        golden.check_method(
            "id()Lnet/minecraft/resources/Identifier;",
            |this: &mut ServerboundCustomClickActionPacket| this.id().clone(),
        );
        golden.check_method(
            "payload()Ljava/util/Optional;",
            |this: &mut ServerboundCustomClickActionPacket| this.payload().cloned(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundCustomClickActionPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ServerCommonPacketListener;)V",
            |this: &mut ServerboundCustomClickActionPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundCustomClickActionPacket,
             other: ServerboundCustomClickActionPacket| *this == other,
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ServerboundCustomClickActionPacket| this.hash_code(),
        );
        golden.finish();
    }
}
