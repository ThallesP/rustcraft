//! net/minecraft/network/protocol/configuration/ClientboundRegistryDataPacket.java
//!
//! Not ported yet: `hashCode`, a record's, which hashes the registry key's identity hash, which
//! changes from run to run.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::core::registry_synchronization::PackedRegistryEntry;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;

// A registry's key, by its identifier.
static REGISTRY_KEY_STREAM_CODEC: StaticStreamCodec<ByteBuf, ResourceKey> = LazyLock::new(|| {
    Box::new(
        Identifier::STREAM_CODEC.map(ResourceKey::create_registry_key, |key: &ResourceKey| {
            key.identifier().clone()
        }),
    )
});

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundRegistryDataPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            &REGISTRY_KEY_STREAM_CODEC,
            |packet: &ClientboundRegistryDataPacket| packet.registry().clone(),
            PackedRegistryEntry::STREAM_CODEC.apply(ByteBufCodecs::list()),
            |packet: &ClientboundRegistryDataPacket| packet.entries().to_vec(),
            ClientboundRegistryDataPacket::new,
        ))
    });

/// One of the registries the server synchronizes, by its key, with its elements.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundRegistryDataPacket {
    registry: ResourceKey,
    entries: Vec<PackedRegistryEntry>,
}

impl ClientboundRegistryDataPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundRegistryDataPacket,
    > = &STREAM_CODEC;

    pub fn new(
        registry: ResourceKey,
        entries: Vec<PackedRegistryEntry>,
    ) -> ClientboundRegistryDataPacket {
        ClientboundRegistryDataPacket { registry, entries }
    }

    pub fn registry(&self) -> &ResourceKey {
        &self.registry
    }

    pub fn entries(&self) -> &[PackedRegistryEntry] {
        &self.entries
    }
}

impl<L: ClientConfigurationPacketListener + ?Sized> Packet<L> for ClientboundRegistryDataPacket {
    fn r#type(&self) -> &'static PacketType {
        ConfigurationPacketTypes::CLIENTBOUND_REGISTRY_DATA
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_registry_data(self);
    }
}

/// A record's: `ClientboundRegistryDataPacket[registry=ResourceKey[…], entries=[…, …]]`, a
/// list printing its elements between brackets.
impl fmt::Display for ClientboundRegistryDataPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries: Vec<String> = self.entries.iter().map(ToString::to_string).collect();
        write!(
            f,
            "ClientboundRegistryDataPacket[registry={}, entries=[{}]]",
            self.registry,
            entries.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundRegistryDataPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::core::registry_synchronization::PackedRegistryEntry;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::resources::resource_key::ResourceKey;

    // A record: its components.
    encode_fields!(ClientboundRegistryDataPacket {
        registry: ResourceKey,
        entries: Vec<PackedRegistryEntry>,
    });

    type Listener = dyn ClientConfigurationPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket",
        );
        golden.pending(
            "hashCode()I",
            "hashes the registry key's identity hash, which changes from run to run",
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundRegistryDataPacket| this.to_string(),
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundRegistryDataPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/resources/ResourceKey;Ljava/util/List;)V",
            ClientboundRegistryDataPacket::new,
        );
        golden.check_method(
            "registry()Lnet/minecraft/resources/ResourceKey;",
            |this: &mut ClientboundRegistryDataPacket| this.registry().clone(),
        );
        golden.check_method(
            "entries()Ljava/util/List;",
            |this: &mut ClientboundRegistryDataPacket| this.entries().to_vec(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundRegistryDataPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/configuration/ClientConfigurationPacketListener;)V",
            |this: &mut ClientboundRegistryDataPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundRegistryDataPacket, other: ClientboundRegistryDataPacket| {
                *this == other
            },
        );
        golden.finish();
    }
}
