//! net/minecraft/network/protocol/common/custom/DiscardedPayload.java

use std::any::Any;
use std::fmt;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
use crate::net::minecraft::network::protocol::common::custom::custom_packet_payload::{
    self, CustomPacketPayload, Type,
};
use crate::net::minecraft::resources::identifier::Identifier;

/// The payload a side reads whatever it doesn't know as: the payload's identifier, and it skips
/// the bytes after it. The scenario's start marker is one, of `rustcraft:scenario_start`.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscardedPayload {
    id: Identifier,
}

impl DiscardedPayload {
    pub fn new(id: Identifier) -> DiscardedPayload {
        DiscardedPayload { id }
    }

    /// Writes nothing, and reads by skipping every byte left, panicking past `max_payload_size`
    /// where Java throws `IllegalArgumentException`. Java's is generic in a `FriendlyByteBuf`'s
    /// subclass; this one in any buffer around a `ByteBuf`.
    pub fn codec<T: AsMut<ByteBuf>>(
        id: Identifier,
        max_payload_size: i32,
    ) -> impl StreamCodec<Buffer = T, Value = DiscardedPayload> {
        custom_packet_payload::codec(
            |_payload: &DiscardedPayload, _buf: &mut T| {},
            move |buf: &mut T| {
                let buf = buf.as_mut();
                let length = buf.readable_bytes();
                if length >= 0 && length <= max_payload_size {
                    buf.skip_bytes(length);
                    DiscardedPayload::new(id.clone())
                } else {
                    panic!("Payload may not be larger than {max_payload_size} bytes");
                }
            },
        )
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }
}

impl CustomPacketPayload for DiscardedPayload {
    /// A new type of its identifier.
    fn r#type(&self) -> Type {
        Type::new(self.id.clone())
    }

    /// A record's: another `DiscardedPayload` with an equal identifier.
    fn equals(&self, other: &dyn CustomPacketPayload) -> bool {
        let other: &dyn Any = other;
        other
            .downcast_ref::<DiscardedPayload>()
            .is_some_and(|other| self == other)
    }

    /// A record's of one component: its hash.
    fn hash_code(&self) -> i32 {
        self.id.hash_code()
    }
}

/// A record's: `DiscardedPayload[id=rustcraft:scenario_start]`.
impl fmt::Display for DiscardedPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DiscardedPayload[id={}]", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::DiscardedPayload;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
    use crate::net::minecraft::network::protocol::common::custom::custom_packet_payload::CustomPacketPayload;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components.
    encode_fields!(DiscardedPayload { id: Identifier });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.common.custom.DiscardedPayload");
        golden.check_codec_from2(
            "codec(Lnet/minecraft/resources/Identifier;I)Lnet/minecraft/network/codec/StreamCodec;",
            |id: Identifier, max_payload_size: i32| {
                DiscardedPayload::codec::<FriendlyByteBuf>(id, max_payload_size)
            },
        );
        golden.check(
            "<init>(Lnet/minecraft/resources/Identifier;)V",
            DiscardedPayload::new,
        );
        golden.check_method(
            "id()Lnet/minecraft/resources/Identifier;",
            |this: &mut DiscardedPayload| this.id().clone(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload$Type;",
            |this: &mut DiscardedPayload| this.r#type(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut DiscardedPayload, other: DiscardedPayload| this.equals(&other),
        );
        golden.check_method("hashCode()I", |this: &mut DiscardedPayload| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut DiscardedPayload| this.to_string(),
        );
        golden.finish();
    }
}
