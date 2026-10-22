//! net/minecraft/network/protocol/common/ServerCommonPacketListener.java
//!
//! Not ported yet: `handleCustomClickAction`, which comes with `ServerboundCustomClickActionPacket`
//! (its payload is an NBT tag).

use crate::net::minecraft::network::protocol::common::serverbound_client_information_packet::ServerboundClientInformationPacket;
use crate::net::minecraft::network::protocol::common::serverbound_custom_payload_packet::ServerboundCustomPayloadPacket;
use crate::net::minecraft::network::protocol::common::serverbound_keep_alive_packet::ServerboundKeepAlivePacket;
use crate::net::minecraft::network::protocol::common::serverbound_pong_packet::ServerboundPongPacket;
use crate::net::minecraft::network::protocol::common::serverbound_resource_pack_packet::ServerboundResourcePackPacket;
use crate::net::minecraft::network::protocol::cookie::server_cookie_packet_listener::ServerCookiePacketListener;

/// A server's listener in the protocols after login (configuration and play), for the packets
/// both have.
pub trait ServerCommonPacketListener: ServerCookiePacketListener {
    fn handle_keep_alive(&mut self, packet: &ServerboundKeepAlivePacket);

    fn handle_pong(&mut self, serverbound_pong_packet: &ServerboundPongPacket);

    fn handle_custom_payload(&mut self, packet: &ServerboundCustomPayloadPacket);

    fn handle_resource_pack_response(&mut self, packet: &ServerboundResourcePackPacket);

    fn handle_client_information(&mut self, packet: &ServerboundClientInformationPacket);
}

#[cfg(test)]
impl ServerCommonPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_keep_alive(&mut self, packet: &ServerboundKeepAlivePacket) {
        self.record("handleKeepAlive", packet);
    }

    fn handle_pong(&mut self, serverbound_pong_packet: &ServerboundPongPacket) {
        self.record("handlePong", serverbound_pong_packet);
    }

    fn handle_custom_payload(&mut self, packet: &ServerboundCustomPayloadPacket) {
        self.record("handleCustomPayload", packet);
    }

    fn handle_resource_pack_response(&mut self, packet: &ServerboundResourcePackPacket) {
        self.record("handleResourcePackResponse", packet);
    }

    fn handle_client_information(&mut self, packet: &ServerboundClientInformationPacket) {
        self.record("handleClientInformation", packet);
    }
}
