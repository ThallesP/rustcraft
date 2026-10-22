//! net/minecraft/network/protocol/configuration/ServerboundSelectKnownPacks.java

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::server::packs::repository::known_pack::KnownPack;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ServerboundSelectKnownPacks> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            KnownPack::STREAM_CODEC.apply(ByteBufCodecs::list_max(64)),
            |packet: &ServerboundSelectKnownPacks| packet.known_packs().to_vec(),
            ServerboundSelectKnownPacks::new,
        ))
    });

/// The client's answer to the server's `select_known_packs`: which of the server's packs it has
/// too, at most 64, so the server leaves their data out.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundSelectKnownPacks {
    known_packs: Vec<KnownPack>,
}

impl ServerboundSelectKnownPacks {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ServerboundSelectKnownPacks> =
        &STREAM_CODEC;

    pub fn new(known_packs: Vec<KnownPack>) -> ServerboundSelectKnownPacks {
        ServerboundSelectKnownPacks { known_packs }
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

impl<L: ServerConfigurationPacketListener + ?Sized> Packet<L> for ServerboundSelectKnownPacks {
    fn r#type(&self) -> &'static PacketType {
        ConfigurationPacketTypes::SERVERBOUND_SELECT_KNOWN_PACKS
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_select_known_packs(self);
    }
}

/// A record's, with a list's `toString`: `ServerboundSelectKnownPacks[knownPacks=[minecraft:core:26.3]]`.
impl fmt::Display for ServerboundSelectKnownPacks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ServerboundSelectKnownPacks[knownPacks=[")?;
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
    use super::ServerboundSelectKnownPacks;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::server::packs::repository::known_pack::KnownPack;

    // A record: its components.
    encode_fields!(ServerboundSelectKnownPacks {
        known_packs: Vec<KnownPack>
    });

    type Listener = dyn ServerConfigurationPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks",
        );
        golden.check_codec("STREAM_CODEC", || ServerboundSelectKnownPacks::STREAM_CODEC);
        golden.check(
            "<init>(Ljava/util/List;)V",
            ServerboundSelectKnownPacks::new,
        );
        golden.check_method(
            "knownPacks()Ljava/util/List;",
            |this: &mut ServerboundSelectKnownPacks| this.known_packs().to_vec(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundSelectKnownPacks| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/configuration/ServerConfigurationPacketListener;)V",
            |this: &mut ServerboundSelectKnownPacks, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundSelectKnownPacks, other: ServerboundSelectKnownPacks| {
                *this == other
            },
        );
        golden.check_method("hashCode()I", |this: &mut ServerboundSelectKnownPacks| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundSelectKnownPacks| this.to_string(),
        );
        golden.finish();
    }
}
