//! net/minecraft/network/protocol/game/ClientboundUpdateAttributesPacket.java
//!
//! Not ported yet: the public constructor, which reads an entity's `AttributeInstance`s, which the
//! player track ports with `AttributeMap`; the server's side of the packet. The snapshot's record
//! `hashCode` hashes its holder's identity hash, which changes from run to run, and its
//! `toString` prints the base with Java's `Double.toString`.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::double::Double;
use crate::java::util::array_list::ArrayList;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::entity::ai::attributes::attribute_modifier::{
    AttributeModifier, Operation,
};

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundUpdateAttributesPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            ByteBufCodecs::VAR_INT,
            ClientboundUpdateAttributesPacket::get_entity_id,
            AttributeSnapshot::STREAM_CODEC.apply(ByteBufCodecs::list_max(128)),
            ClientboundUpdateAttributesPacket::get_values,
            ClientboundUpdateAttributesPacket::from_values,
        ))
    });

static MODIFIER_STREAM_CODEC: StaticStreamCodec<ByteBuf, AttributeModifier> = LazyLock::new(|| {
    Box::new(stream_codec::composite3(
        Identifier::STREAM_CODEC,
        |modifier: &AttributeModifier| modifier.id().clone(),
        ByteBufCodecs::DOUBLE,
        AttributeModifier::amount,
        Operation::STREAM_CODEC,
        AttributeModifier::operation,
        AttributeModifier::new,
    ))
});

static SNAPSHOT_STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, AttributeSnapshot> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite3(
            ByteBufCodecs::holder_registry(Registries::ATTRIBUTE),
            AttributeSnapshot::attribute,
            ByteBufCodecs::DOUBLE,
            AttributeSnapshot::base,
            AttributeSnapshot::MODIFIER_STREAM_CODEC
                .apply(ByteBufCodecs::collection_operation(
                    ArrayList::new as fn(i32) -> Vec<AttributeModifier>,
                ))
                .cast::<RegistryFriendlyByteBuf>(),
            AttributeSnapshot::modifiers,
            AttributeSnapshot::new,
        ))
    });

/// An entity's attributes the server changed: each one's base value and modifiers.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundUpdateAttributesPacket {
    entity_id: i32,
    attributes: Vec<AttributeSnapshot>,
}

impl ClientboundUpdateAttributesPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundUpdateAttributesPacket,
    > = &STREAM_CODEC;

    /// The private constructor the codec makes it with.
    fn from_values(
        entity_id: i32,
        attributes: Vec<AttributeSnapshot>,
    ) -> ClientboundUpdateAttributesPacket {
        ClientboundUpdateAttributesPacket {
            entity_id,
            attributes,
        }
    }

    pub fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    pub fn get_values(&self) -> Vec<AttributeSnapshot> {
        self.attributes.clone()
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundUpdateAttributesPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_UPDATE_ATTRIBUTES
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_update_attributes(self);
    }
}

/// `ClientboundUpdateAttributesPacket.AttributeSnapshot`: an attribute, its base value and its
/// modifiers, which the client sets on the entity's own (`handleUpdateAttributes`).
#[derive(Clone, Debug)]
pub struct AttributeSnapshot {
    attribute: Holder,
    base: f64,
    modifiers: Vec<AttributeModifier>,
}

impl AttributeSnapshot {
    pub const MODIFIER_STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, AttributeModifier> =
        &MODIFIER_STREAM_CODEC;
    pub const STREAM_CODEC: &'static StaticStreamCodec<RegistryFriendlyByteBuf, AttributeSnapshot> =
        &SNAPSHOT_STREAM_CODEC;

    pub fn new(
        attribute: Holder,
        base: f64,
        modifiers: Vec<AttributeModifier>,
    ) -> AttributeSnapshot {
        AttributeSnapshot {
            attribute,
            base,
            modifiers,
        }
    }

    pub fn attribute(&self) -> Holder {
        self.attribute.clone()
    }

    pub fn base(&self) -> f64 {
        self.base
    }

    pub fn modifiers(&self) -> Vec<AttributeModifier> {
        self.modifiers.clone()
    }
}

/// A record's: the base compares by its bits, NaNs made one (`Double.compare`), and the
/// modifiers as lists.
impl PartialEq for AttributeSnapshot {
    fn eq(&self, other: &AttributeSnapshot) -> bool {
        self.attribute == other.attribute
            && Double::double_to_long_bits(self.base) == Double::double_to_long_bits(other.base)
            && self.modifiers == other.modifiers
    }
}

#[cfg(test)]
mod tests {
    use super::{AttributeSnapshot, ClientboundUpdateAttributesPacket};
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::entity::ai::attributes::attribute_modifier::AttributeModifier;

    // The fields the codec's constructor sets.
    encode_fields!(ClientboundUpdateAttributesPacket {
        entity_id: i32,
        attributes: Vec<AttributeSnapshot>,
    });
    encode_fields!(AttributeSnapshot {
        attribute: Holder,
        base: f64,
        modifiers: Vec<AttributeModifier>,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundUpdateAttributesPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket");
        golden.pending(
            "<init>(ILjava/util/Collection;)V",
            "reads an entity's AttributeInstances, the player track's",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundUpdateAttributesPacket::STREAM_CODEC
        });
        golden.check_method("getEntityId()I", |this: &mut This| this.get_entity_id());
        golden.check_method("getValues()Ljava/util/List;", |this: &mut This| {
            this.get_values()
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut This| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }

    #[test]
    fn snapshot_matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket$AttributeSnapshot",
        );
        golden.pending("hashCode()I", "hashes the holder's identity hash");
        golden.pending(
            "toString()Ljava/lang/String;",
            "waits for Java's Double.toString",
        );
        golden.check_codec("MODIFIER_STREAM_CODEC", || {
            AttributeSnapshot::MODIFIER_STREAM_CODEC
        });
        golden.check_codec("STREAM_CODEC", || AttributeSnapshot::STREAM_CODEC);
        golden.check(
            "<init>(Lnet/minecraft/core/Holder;DLjava/util/Collection;)V",
            AttributeSnapshot::new,
        );
        golden.check_method(
            "attribute()Lnet/minecraft/core/Holder;",
            |this: &mut AttributeSnapshot| this.attribute(),
        );
        golden.check_method("base()D", |this: &mut AttributeSnapshot| this.base());
        golden.check_method(
            "modifiers()Ljava/util/Collection;",
            |this: &mut AttributeSnapshot| this.modifiers(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut AttributeSnapshot, other: AttributeSnapshot| *this == other,
        );
        golden.finish();
    }
}
