//! net/minecraft/network/protocol/game/ClientboundRemoveEntitiesPacket.java
//!
//! Java's `IntList` is a `Vec<i32>`: the codec reads an `IntArrayList`, and the varargs
//! constructor wraps its array in an immutable list, which equal each other like any two lists.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::util::array_list::ArrayList;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

// A VarInt count, then each id as a VarInt.
static ID_LIST_STREAM_CODEC: StaticStreamCodec<ByteBuf, Vec<i32>> = LazyLock::new(|| {
    Box::new(
        ByteBufCodecs::VAR_INT.apply(ByteBufCodecs::collection_operation(
            ArrayList::new as fn(i32) -> Vec<i32>,
        )),
    )
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundRemoveEntitiesPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            ClientboundRemoveEntitiesPacket::ID_LIST_STREAM_CODEC,
            ClientboundRemoveEntitiesPacket::entity_ids,
            ClientboundRemoveEntitiesPacket::new,
        ))
    });

/// The entities that left the client's view, by their ids.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundRemoveEntitiesPacket {
    entity_ids: Vec<i32>,
}

impl ClientboundRemoveEntitiesPacket {
    const ID_LIST_STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Vec<i32>> =
        &ID_LIST_STREAM_CODEC;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundRemoveEntitiesPacket> =
        &STREAM_CODEC;

    pub fn new(entity_ids: Vec<i32>) -> ClientboundRemoveEntitiesPacket {
        ClientboundRemoveEntitiesPacket { entity_ids }
    }

    /// `ClientboundRemoveEntitiesPacket(int...)`.
    pub fn from_int_array(ids: Vec<i32>) -> ClientboundRemoveEntitiesPacket {
        ClientboundRemoveEntitiesPacket::new(ids)
    }

    pub fn entity_ids(&self) -> Vec<i32> {
        self.entity_ids.clone()
    }

    /// A record's: its one component's, a list's hash, each id added to 31 times the ones
    /// before, starting from 1.
    pub fn hash_code(&self) -> i32 {
        self.entity_ids
            .iter()
            .fold(1i32, |hash, id| hash.wrapping_mul(31).wrapping_add(*id))
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundRemoveEntitiesPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_REMOVE_ENTITIES
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_remove_entities(self);
    }
}

/// A record's: `ClientboundRemoveEntitiesPacket[entityIds=[1, 2]]`, the list as fastutil prints
/// it.
impl fmt::Display for ClientboundRemoveEntitiesPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ClientboundRemoveEntitiesPacket[entityIds=[")?;
        for (i, id) in self.entity_ids.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{id}")?;
        }
        f.write_str("]]")
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundRemoveEntitiesPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundRemoveEntitiesPacket {
        entity_ids: Vec<i32>,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundRemoveEntitiesPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundRemoveEntitiesPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lit/unimi/dsi/fastutil/ints/IntList;)V",
            ClientboundRemoveEntitiesPacket::new,
        );
        golden.check(
            "<init>([I)V",
            ClientboundRemoveEntitiesPacket::from_int_array,
        );
        golden.check_method(
            "entityIds()Lit/unimi/dsi/fastutil/ints/IntList;",
            |this: &mut This| this.entity_ids(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut This| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
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
}
