//! net/minecraft/network/protocol/configuration/ClientboundSelectKnownPacks.java

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::server::packs::repository::known_pack::KnownPack;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundSelectKnownPacks> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            KnownPack::STREAM_CODEC.apply(ByteBufCodecs::list()),
            |packet: &ClientboundSelectKnownPacks| packet.known_packs().to_vec(),
            ClientboundSelectKnownPacks::new,
        ))
    });

/// The packs the server's registries and tags come from, which the client answers with the ones
/// it has too (`ServerboundSelectKnownPacks`), so the server can leave their data out. Unlike the
/// answer, the list has no limit.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSelectKnownPacks {
    known_packs: Vec<KnownPack>,
}

impl ClientboundSelectKnownPacks {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundSelectKnownPacks> =
        &STREAM_CODEC;

    pub fn new(known_packs: Vec<KnownPack>) -> ClientboundSelectKnownPacks {
        ClientboundSelectKnownPacks { known_packs }
    }

    pub fn known_packs(&self) -> &[KnownPack] {
        &self.known_packs
    }

    /// A record's of one component: its hash, a list's, which adds each pack's to 31 times the
    /// ones before, from 1.
    pub fn hash_code(&self) -> i32 {
        self.known_packs.iter().fold(1i32, |hash, pack| {
            hash.wrapping_mul(31).wrapping_add(pack.hash_code())
        })
    }
}

impl<L: ClientConfigurationPacketListener + ?Sized> Packet<L> for ClientboundSelectKnownPacks {
    fn r#type(&self) -> &'static PacketType {
        ConfigurationPacketTypes::CLIENTBOUND_SELECT_KNOWN_PACKS
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_select_known_packs(self);
    }
}

/// A record's, with a list's `toString`: `ClientboundSelectKnownPacks[knownPacks=[minecraft:core:26.3]]`.
impl fmt::Display for ClientboundSelectKnownPacks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ClientboundSelectKnownPacks[knownPacks=[")?;
        for (index, pack) in self.known_packs.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{pack}")?;
        }
        f.write_str("]]")
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSelectKnownPacks;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::server::packs::repository::known_pack::KnownPack;

    // A record: its components.
    encode_fields!(ClientboundSelectKnownPacks {
        known_packs: Vec<KnownPack>
    });

    type Listener = dyn ClientConfigurationPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks",
        );
        golden.check_codec("STREAM_CODEC", || ClientboundSelectKnownPacks::STREAM_CODEC);
        golden.check(
            "<init>(Ljava/util/List;)V",
            ClientboundSelectKnownPacks::new,
        );
        golden.check_method(
            "knownPacks()Ljava/util/List;",
            |this: &mut ClientboundSelectKnownPacks| this.known_packs().to_vec(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundSelectKnownPacks| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/configuration/ClientConfigurationPacketListener;)V",
            |this: &mut ClientboundSelectKnownPacks, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundSelectKnownPacks, other: ClientboundSelectKnownPacks| {
                *this == other
            },
        );
        golden.check_method("hashCode()I", |this: &mut ClientboundSelectKnownPacks| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundSelectKnownPacks| this.to_string(),
        );
        golden.finish();
    }
}
