//! net/minecraft/network/protocol/configuration/ClientboundUpdateEnabledFeaturesPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::util::hash_set::HashSet;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundUpdateEnabledFeaturesPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            Identifier::STREAM_CODEC.apply(ByteBufCodecs::collection_operation(
                HashSet::from_initial_capacity,
            )),
            |packet: &ClientboundUpdateEnabledFeaturesPacket| packet.features().clone(),
            ClientboundUpdateEnabledFeaturesPacket::new,
        ))
    });

/// The feature flags the server's world has on (`minecraft:vanilla`, and experiments like
/// `minecraft:trade_rebalance`), by their identifiers. The codec reads them into a `HashSet` and
/// writes them in its order, which its table decides.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundUpdateEnabledFeaturesPacket {
    features: HashSet<Identifier>,
}

impl ClientboundUpdateEnabledFeaturesPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        ClientboundUpdateEnabledFeaturesPacket,
    > = &STREAM_CODEC;

    pub fn new(features: HashSet<Identifier>) -> ClientboundUpdateEnabledFeaturesPacket {
        ClientboundUpdateEnabledFeaturesPacket { features }
    }

    pub fn features(&self) -> &HashSet<Identifier> {
        &self.features
    }

    /// A record's of one component: its hash, a set's, the sum of its identifiers'.
    pub fn hash_code(&self) -> i32 {
        self.features.hash_code()
    }
}

impl<L: ClientConfigurationPacketListener + ?Sized> Packet<L>
    for ClientboundUpdateEnabledFeaturesPacket
{
    fn r#type(&self) -> &'static PacketType {
        ConfigurationPacketTypes::CLIENTBOUND_UPDATE_ENABLED_FEATURES
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_enabled_features(self);
    }
}

/// A record's, with a set's `toString`: `ClientboundUpdateEnabledFeaturesPacket[features=[minecraft:vanilla]]`.
impl fmt::Display for ClientboundUpdateEnabledFeaturesPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundUpdateEnabledFeaturesPacket[features={}]",
            self.features
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundUpdateEnabledFeaturesPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::java::util::hash_set::HashSet;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components.
    encode_fields!(ClientboundUpdateEnabledFeaturesPacket {
        features: HashSet<Identifier>
    });

    type Listener = dyn ClientConfigurationPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundUpdateEnabledFeaturesPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Ljava/util/Set;)V",
            ClientboundUpdateEnabledFeaturesPacket::new,
        );
        golden.check_method(
            "features()Ljava/util/Set;",
            |this: &mut ClientboundUpdateEnabledFeaturesPacket| this.features().clone(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundUpdateEnabledFeaturesPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/configuration/ClientConfigurationPacketListener;)V",
            |this: &mut ClientboundUpdateEnabledFeaturesPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundUpdateEnabledFeaturesPacket,
             other: ClientboundUpdateEnabledFeaturesPacket| *this == other,
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ClientboundUpdateEnabledFeaturesPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundUpdateEnabledFeaturesPacket| this.to_string(),
        );
        golden.finish();
    }
}
