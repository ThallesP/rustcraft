//! No Java counterpart: the stand-in for a packet the port can't decode yet (GOAL.md).
//!
//! A protocol lists every one of vanilla's packet types in vanilla's order, so ids match, and a
//! type whose class isn't ported yet gets this codec: it reads past the rest of the frame, which
//! holds nothing but the packet, and its packet's `handle` does nothing. That's only right for a
//! packet vanilla answers nothing to, so every packet vanilla answers gets ported instead, and
//! README.md's feature table counts the ones left. Writing one panics: the port never sends a
//! packet it hasn't ported. How many of each type the client skipped is counted (`counts`), which
//! the client prints as it exits, for the soak's report of what to port next.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
use crate::net::minecraft::network::packet_processor::lock;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

/// How many packets of each type the client skipped, by the type's id.
static COUNTS: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());

/// A packet of a type the port can't decode yet, whose bytes were skipped.
#[derive(Clone, Debug)]
pub struct Unported {
    r#type: &'static PacketType,
    length: i32,
}

impl Unported {
    /// The codec standing in for `type`'s. Every protocol's buffer wraps a `ByteBuf`.
    pub fn codec(r#type: &'static PacketType) -> UnportedCodec {
        UnportedCodec { r#type }
    }

    /// How many bytes the packet had after its id.
    pub fn length(&self) -> i32 {
        self.length
    }

    /// The types the client skipped so far, each with how many, the most first.
    pub fn counts() -> Vec<(String, u64)> {
        let mut counts: Vec<(String, u64)> = lock(&COUNTS)
            .iter()
            .map(|(r#type, count)| (r#type.clone(), *count))
            .collect();
        counts.sort_by(|(a, a_count), (b, b_count)| b_count.cmp(a_count).then(a.cmp(b)));
        counts
    }
}

impl<L: ?Sized> Packet<L> for Unported {
    fn r#type(&self) -> &'static PacketType {
        self.r#type
    }

    fn handle(&self, _listener: &mut L) {}
}

/// What `Unported::codec` makes.
#[derive(Clone)]
pub struct UnportedCodec {
    r#type: &'static PacketType,
}

impl StreamCodec for UnportedCodec {
    type Buffer = ByteBuf;
    type Value = Unported;

    fn decode(&self, input: &mut ByteBuf) -> Unported {
        let length = input.readable_bytes();
        input.skip_bytes(length);
        *lock(&COUNTS)
            .entry(self.r#type.id().to_string())
            .or_insert(0) += 1;
        Unported {
            r#type: self.r#type,
            length,
        }
    }

    fn encode(&self, _output: &mut ByteBuf, value: &Unported) {
        panic!("{} isn't ported, so it can't be written", value.r#type);
    }
}

#[cfg(test)]
mod tests {
    use super::Unported;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
    use crate::net::minecraft::network::protocol::packet::Packet;

    #[test]
    fn skips_the_rest_of_the_frame() {
        let mut buffer = unpooled::buffer();
        buffer.write_bytes_array(&[1, 2, 3, 4, 5]);
        buffer.read_byte();
        let packet = Unported::codec(CommonPacketTypes::CLIENTBOUND_DISCONNECT).decode(&mut buffer);
        assert_eq!(buffer.readable_bytes(), 0);
        assert_eq!(packet.length(), 4);
        assert_eq!(
            Packet::<()>::r#type(&packet),
            &**CommonPacketTypes::CLIENTBOUND_DISCONNECT
        );
    }
}
