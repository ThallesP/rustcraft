//! net/minecraft/network/protocol/game/ClientboundBundleDelimiterPacket.java
//!
//! Java's `BundleDelimiterPacket.handle` throws, since the bundle packer takes every delimiter out
//! of the stream first. The packer isn't ported yet (`Connection`), so the delimiter reaches the
//! listener, which does nothing with it, and a bundle's packets are handled one by one.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

/// `ProtocolInfoBuilder.withBundlePacket`'s codec of the delimiter, `StreamCodec.unit`.
static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundBundleDelimiterPacket> =
    LazyLock::new(|| Box::new(stream_codec::unit(ClientboundBundleDelimiterPacket)));

/// The start and the end of a bundle, the packets between which the client handles together.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundBundleDelimiterPacket;

impl ClientboundBundleDelimiterPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundBundleDelimiterPacket> =
        &STREAM_CODEC;
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundBundleDelimiterPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_BUNDLE_DELIMITER
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_bundle_delimiter(self);
    }
}
