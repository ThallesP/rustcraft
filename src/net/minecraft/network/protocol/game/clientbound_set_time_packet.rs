//! net/minecraft/network/protocol/game/ClientboundSetTimePacket.java
//!
//! Not ported yet: the record's `toString`, which prints the clocks' floats with Java's
//! `Float.toString`. Its `hashCode` hashes the holders' identity hashes, which change from run to
//! run.

use std::sync::LazyLock;

use crate::java::util::hash_map::{self, HashMap};
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::world::clock::clock_network_state::ClockNetworkState;
use crate::net::minecraft::world::clock::world_clock::WorldClock;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundSetTimePacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            ByteBufCodecs::LONG,
            ClientboundSetTimePacket::game_time,
            ByteBufCodecs::map(
                ClockUpdates::from_initial_capacity,
                WorldClock::STREAM_CODEC,
                ClockNetworkState::STREAM_CODEC.cast::<RegistryFriendlyByteBuf>(),
            ),
            ClientboundSetTimePacket::clock_updates,
            ClientboundSetTimePacket::new,
        ))
    });

/// Java's `Map<Holder<WorldClock>, ClockNetworkState>`, a `HashMap` (its codec's `HashMap::new`).
/// Its holders hash by identity in Java, so its order changes from run to run; here they hash
/// their keys (`Holder`). Nothing reads the order: each update is its own clock's.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClockUpdates(HashMap<Holder, ClockNetworkState>);

impl ClockUpdates {
    pub fn new() -> ClockUpdates {
        ClockUpdates(HashMap::new())
    }

    /// `new HashMap<>(initialCapacity)`, what the codec makes.
    pub fn from_initial_capacity(initial_capacity: i32) -> ClockUpdates {
        ClockUpdates(HashMap::from_initial_capacity(initial_capacity))
    }

    pub fn put(&mut self, clock: Holder, state: ClockNetworkState) -> Option<ClockNetworkState> {
        self.0.put(clock, state)
    }

    pub fn size(&self) -> i32 {
        self.0.size()
    }

    pub fn iter(&self) -> hash_map::Iter<'_, Holder, ClockNetworkState> {
        self.0.iter()
    }
}

impl Extend<(Holder, ClockNetworkState)> for ClockUpdates {
    fn extend<I: IntoIterator<Item = (Holder, ClockNetworkState)>>(&mut self, entries: I) {
        self.0.extend(entries);
    }
}

impl<'a> IntoIterator for &'a ClockUpdates {
    type Item = (&'a Holder, &'a ClockNetworkState);
    type IntoIter = hash_map::Iter<'a, Holder, ClockNetworkState>;

    fn into_iter(self) -> hash_map::Iter<'a, Holder, ClockNetworkState> {
        self.0.iter()
    }
}

/// The game's time, and the clocks the server changed: every one when the player joins, then
/// the ones whose rate or ticks a command changed.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetTimePacket {
    game_time: i64,
    clock_updates: ClockUpdates,
}

impl ClientboundSetTimePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundSetTimePacket,
    > = &STREAM_CODEC;

    pub fn new(game_time: i64, clock_updates: ClockUpdates) -> ClientboundSetTimePacket {
        ClientboundSetTimePacket {
            game_time,
            clock_updates,
        }
    }

    pub fn game_time(&self) -> i64 {
        self.game_time
    }

    pub fn clock_updates(&self) -> ClockUpdates {
        self.clock_updates.clone()
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetTimePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_TIME
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_time(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{ClientboundSetTimePacket, ClockUpdates};
    use crate::golden::{Decode, Encode, Golden, elements, encode_fields};
    use crate::java::util::hash_map::HashMap;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::clock::clock_network_state::ClockNetworkState;

    // A HashMap's [capacity, [[key, value], …]], the entries sorted by key, since Java's order
    // changes from run to run (TESTING.md).
    impl Decode for ClockUpdates {
        fn decode(value: &Value) -> Self {
            ClockUpdates(HashMap::decode(value))
        }
    }

    impl ClockUpdates {
        fn sorted(&self) -> Vec<(&Holder, &ClockNetworkState)> {
            let mut entries: Vec<_> = self.0.iter().collect();
            entries.sort_by(|(a, _), (b, _)| {
                let key = |holder: &Holder| holder.unwrap_key().cloned();
                key(a)
                    .map(|key| key.identifier().clone())
                    .cmp(&key(b).map(|key| key.identifier().clone()))
            });
            entries
        }
    }

    impl Encode for ClockUpdates {
        fn encode(&self) -> Value {
            let entries: Vec<Value> = self
                .sorted()
                .into_iter()
                .map(|(key, value)| Value::from(vec![key.encode(), value.encode()]))
                .collect();
            Value::from(vec![Value::from(self.0.capacity()), Value::from(entries)])
        }

        fn matches(&self, expected: &Value) -> bool {
            let Some([capacity, entries]) = expected.as_array().map(Vec::as_slice) else {
                return false;
            };
            let Some(entries) = entries.as_array() else {
                return false;
            };
            self.0.capacity().matches(capacity)
                && entries.len() == self.0.size() as usize
                && self
                    .sorted()
                    .into_iter()
                    .zip(entries)
                    .all(|((key, value), entry)| {
                        let [expected_key, expected_value] = elements(entry);
                        key.matches(expected_key) && value.matches(expected_value)
                    })
        }
    }

    encode_fields!(ClientboundSetTimePacket {
        game_time: i64,
        clock_updates: ClockUpdates,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetTimePacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.ClientboundSetTimePacket");
        golden.pending("hashCode()I", "hashes the holders' identity hashes");
        golden.pending(
            "toString()Ljava/lang/String;",
            "waits for Java's Float.toString",
        );
        golden.check_codec("STREAM_CODEC", || ClientboundSetTimePacket::STREAM_CODEC);
        golden.check("<init>(JLjava/util/Map;)V", ClientboundSetTimePacket::new);
        golden.check_method("gameTime()J", |this: &mut This| this.game_time());
        golden.check_method("clockUpdates()Ljava/util/Map;", |this: &mut This| {
            this.clock_updates()
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
}
