//! net/minecraft/network/protocol/common/ClientboundPostEffectsPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundPostEffectsPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            Identifier::STREAM_CODEC.apply(ByteBufCodecs::list()),
            |packet: &ClientboundPostEffectsPacket| packet.post_effects().to_vec(),
            ClientboundPostEffectsPacket::new,
        ))
    });

/// The post-processing shaders a server puts over the client's view, by their identifiers.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundPostEffectsPacket {
    post_effects: Vec<Identifier>,
}

impl ClientboundPostEffectsPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundPostEffectsPacket> =
        &STREAM_CODEC;

    pub fn new(post_effects: Vec<Identifier>) -> ClientboundPostEffectsPacket {
        ClientboundPostEffectsPacket { post_effects }
    }

    pub fn post_effects(&self) -> &[Identifier] {
        &self.post_effects
    }

    /// A record's of one component: its hash, a list's, which adds each identifier's to 31 times
    /// the ones before, from 1.
    pub fn hash_code(&self) -> i32 {
        self.post_effects.iter().fold(1i32, |hash, effect| {
            hash.wrapping_mul(31).wrapping_add(effect.hash_code())
        })
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundPostEffectsPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_POST_EFFECTS
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_post_effects(self);
    }
}

/// A record's, with a list's `toString`: `ClientboundPostEffectsPacket[postEffects=[minecraft:creeper]]`.
impl fmt::Display for ClientboundPostEffectsPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ClientboundPostEffectsPacket[postEffects=[")?;
        for (index, effect) in self.post_effects.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{effect}")?;
        }
        f.write_str("]]")
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundPostEffectsPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components.
    encode_fields!(ClientboundPostEffectsPacket {
        post_effects: Vec<Identifier>
    });

    type Listener = dyn ClientCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ClientboundPostEffectsPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundPostEffectsPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Ljava/util/List;)V",
            ClientboundPostEffectsPacket::new,
        );
        golden.check_method(
            "postEffects()Ljava/util/List;",
            |this: &mut ClientboundPostEffectsPacket| this.post_effects().to_vec(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundPostEffectsPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut ClientboundPostEffectsPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundPostEffectsPacket, other: ClientboundPostEffectsPacket| {
                *this == other
            },
        );
        golden.check_method("hashCode()I", |this: &mut ClientboundPostEffectsPacket| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundPostEffectsPacket| this.to_string(),
        );
        golden.finish();
    }
}
