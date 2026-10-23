//! net/minecraft/network/protocol/status/ClientboundStatusResponsePacket.java
//!
//! Its status is the `ServerStatus` stand-in, the JSON's text: Java's codec reads the text
//! (`ByteBufCodecs.lenientJson(32767)`: a string of at most 32767 units), parses it with Gson,
//! and decodes the status from that with `ServerStatus.CODEC`; this one reads and writes the
//! text, and leaves the rest for when text components and DataFixerUpper's codecs come.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::protocol::status::client_status_packet_listener::ClientStatusPacketListener;
use crate::net::minecraft::network::protocol::status::server_status::ServerStatus;
use crate::net::minecraft::network::protocol::status::status_packet_types::StatusPacketTypes;
use crate::net::minecraft::network::utf8_string::Utf8String;

/// `lenientJson`'s maximum, in UTF-16 units.
const MAX_JSON_LENGTH: i32 = 32767;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundStatusResponsePacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::of(
            |output: &mut ByteBuf, packet: &ClientboundStatusResponsePacket| {
                Utf8String::write(output, packet.status.json(), MAX_JSON_LENGTH);
            },
            |input: &mut ByteBuf| {
                ClientboundStatusResponsePacket::new(ServerStatus::new(Utf8String::read(
                    input,
                    MAX_JSON_LENGTH,
                )))
            },
        ))
    });

/// The server's answer to a status request: what the server list shows of it.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundStatusResponsePacket {
    status: ServerStatus,
}

impl ClientboundStatusResponsePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundStatusResponsePacket> =
        &STREAM_CODEC;

    pub fn new(status: ServerStatus) -> ClientboundStatusResponsePacket {
        ClientboundStatusResponsePacket { status }
    }

    pub fn status(&self) -> &ServerStatus {
        &self.status
    }
}

impl<L: ClientStatusPacketListener + ?Sized> Packet<L> for ClientboundStatusResponsePacket {
    fn r#type(&self) -> &'static PacketType {
        StatusPacketTypes::CLIENTBOUND_STATUS_RESPONSE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_status_response(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundStatusResponsePacket;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::status::server_status::ServerStatus;

    /// The status response vanilla's server sent the oracle client's ping on the walk scenario's
    /// world (`vanilla/26.3/oracle/walk-ping/capture.jsonl`): its JSON behind its length.
    const VANILLA_STATUS_RESPONSE: &str = "6d7b226465736372697074696f6e223a2241204d696e65637261667420536572766572222c22706c6179657273223a7b226d6178223a32302c226f6e6c696e65223a307d2c2276657273696f6e223a7b226e616d65223a2232362e33222c2270726f746f636f6c223a3737377d7d";

    fn bytes(hex: &str) -> Vec<i8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as i8)
            .collect()
    }

    #[test]
    fn reads_and_writes_vanilla_s_status_response() {
        let mut buffer = unpooled::buffer();
        buffer.write_bytes_array(&bytes(VANILLA_STATUS_RESPONSE));
        let packet = ClientboundStatusResponsePacket::STREAM_CODEC.decode(&mut buffer);
        assert_eq!(buffer.readable_bytes(), 0);
        assert_eq!(
            packet.status(),
            &ServerStatus::new(
                r#"{"description":"A Minecraft Server","players":{"max":20,"online":0},"version":{"name":"26.3","protocol":777}}"#
                    .to_string()
            )
        );

        let mut output = unpooled::buffer();
        ClientboundStatusResponsePacket::STREAM_CODEC.encode(&mut output, &packet);
        let mut written = vec![0; output.readable_bytes() as usize];
        output.read_bytes_array(&mut written);
        assert_eq!(written, bytes(VANILLA_STATUS_RESPONSE));
    }
}
