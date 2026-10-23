//! net/minecraft/network/protocol/game/ClientboundBundleDelimiterPacket.java
//!
//! Java's class has no codec: `ProtocolInfoBuilder.withBundlePacket` makes `StreamCodec.unit` of
//! the delimiter `GameProtocols` gives it.

use crate::net::minecraft::network::protocol::bundle_delimiter_packet;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

/// The start and the end of a bundle, the packets between which the client handles together.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundBundleDelimiterPacket;

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundBundleDelimiterPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_BUNDLE_DELIMITER
    }

    fn handle(&self, _listener: &mut L) {
        bundle_delimiter_packet::handle();
    }
}
