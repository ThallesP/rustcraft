//! net/minecraft/network/protocol/game/ClientboundSetEntityDataPacket.java
//!
//! Its record's `hashCode` and `toString` hash and print its values' serializers by identity
//! (`DataValue`), so they aren't ported.
//!
//! A value whose serializer's type isn't ported yet skips the rest of the frame
//! (`EntityDataSerializer`), so `unpack` stops after it: the packet holds the values before it,
//! and that one as `Unported`. Java reads them all.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::network::syncher::entity_data_serializer::EntityDataValue;
use crate::net::minecraft::network::syncher::synched_entity_data::DataValue;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundSetEntityDataPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetEntityDataPacket::write,
            ClientboundSetEntityDataPacket::from_registry_friendly_byte_buf,
        ))
    });

/// An entity's data the server changed, or all of it that isn't the default when the entity
/// comes into view.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetEntityDataPacket {
    id: i32,
    packed_items: Vec<DataValue>,
}

impl ClientboundSetEntityDataPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundSetEntityDataPacket,
    > = &STREAM_CODEC;
    /// The id that ends the values.
    pub const EOF_MARKER: i32 = 255;

    pub fn new(id: i32, packed_items: Vec<DataValue>) -> ClientboundSetEntityDataPacket {
        ClientboundSetEntityDataPacket { id, packed_items }
    }

    fn from_registry_friendly_byte_buf(
        input: &mut RegistryFriendlyByteBuf,
    ) -> ClientboundSetEntityDataPacket {
        let id = input.read_var_int();
        ClientboundSetEntityDataPacket::new(id, ClientboundSetEntityDataPacket::unpack(input))
    }

    fn pack(items: &[DataValue], output: &mut RegistryFriendlyByteBuf) {
        for item in items {
            item.write(output);
        }

        output.write_byte(ClientboundSetEntityDataPacket::EOF_MARKER);
    }

    fn unpack(input: &mut RegistryFriendlyByteBuf) -> Vec<DataValue> {
        let mut result = Vec::new();

        loop {
            let id = i32::from(input.read_unsigned_byte());
            if id == ClientboundSetEntityDataPacket::EOF_MARKER {
                break;
            }
            let item = DataValue::read(input, id);
            let unported = matches!(item.value(), EntityDataValue::Unported(_));
            result.push(item);
            if unported {
                break;
            }
        }

        result
    }

    fn write(&self, output: &mut RegistryFriendlyByteBuf) {
        output.write_var_int(self.id);
        ClientboundSetEntityDataPacket::pack(&self.packed_items, output);
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn packed_items(&self) -> Vec<DataValue> {
        self.packed_items.clone()
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetEntityDataPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_ENTITY_DATA
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_entity_data(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetEntityDataPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::core::registry_access;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
    use crate::net::minecraft::network::syncher::entity_data_serializer::{
        EntityDataSerializer, EntityDataValue,
    };
    use crate::net::minecraft::network::syncher::synched_entity_data::DataValue;

    encode_fields!(ClientboundSetEntityDataPacket {
        id: i32,
        packed_items: Vec<DataValue>,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetEntityDataPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket");
        golden.pending("hashCode()I", "hashes the serializers' identity hashes");
        golden.pending(
            "toString()Ljava/lang/String;",
            "prints the serializers' identities",
        );
        golden.field("EOF_MARKER", ClientboundSetEntityDataPacket::EOF_MARKER);
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetEntityDataPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(ILjava/util/List;)V",
            ClientboundSetEntityDataPacket::new,
        );
        golden.check_method("id()I", |this: &mut This| this.id());
        golden.check_method("packedItems()Ljava/util/List;", |this: &mut This| {
            this.packed_items()
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
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

    /// A value whose type isn't ported (a custom name, an optional component) ends the values
    /// read, and the rest of the frame is skipped, so the packet decodes to its last byte.
    #[test]
    fn stops_at_an_unported_serializer() {
        let mut buffer = unpooled::buffer();
        // Entity 7: its flags, then its custom name (an optional component, serializer 6) and
        // its pose, which the component's unknown length hides.
        for byte in [7, 0, 0, 0x08, 2, 6, 1, 0x08, 0, 1, 0x41, 6, 20, 5, 0xff] {
            buffer.write_byte(byte);
        }
        let mut input = RegistryFriendlyByteBuf::new(buffer, registry_access::tests::golden());
        let packet = ClientboundSetEntityDataPacket::STREAM_CODEC.decode(&mut input);
        assert_eq!(input.readable_bytes(), 0);
        assert_eq!(
            packet.packed_items(),
            vec![
                DataValue::new(0, EntityDataValue::Byte(0x08)),
                DataValue::new(
                    2,
                    EntityDataValue::Unported(EntityDataSerializer::OPTIONAL_COMPONENT)
                ),
            ]
        );
    }
}
