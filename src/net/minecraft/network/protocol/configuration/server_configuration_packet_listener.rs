//! net/minecraft/network/protocol/configuration/ServerConfigurationPacketListener.java
//!
//! Not ported yet: its `protocol()`, which comes with `PacketListener`'s.

use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use crate::net::minecraft::network::protocol::configuration::serverbound_accept_code_of_conduct_packet::ServerboundAcceptCodeOfConductPacket;
use crate::net::minecraft::network::protocol::configuration::serverbound_finish_configuration_packet::ServerboundFinishConfigurationPacket;
use crate::net::minecraft::network::protocol::configuration::serverbound_select_known_packs::ServerboundSelectKnownPacks;

/// The server's listener while it configures a client that has logged in.
pub trait ServerConfigurationPacketListener: ServerCommonPacketListener {
    fn handle_configuration_finished(&mut self, packet: &ServerboundFinishConfigurationPacket);

    fn handle_select_known_packs(&mut self, packet: &ServerboundSelectKnownPacks);

    fn handle_accept_code_of_conduct(&mut self, packet: &ServerboundAcceptCodeOfConductPacket);
}

#[cfg(test)]
impl ServerConfigurationPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_configuration_finished(&mut self, packet: &ServerboundFinishConfigurationPacket) {
        self.record("handleConfigurationFinished", packet);
    }

    fn handle_select_known_packs(&mut self, packet: &ServerboundSelectKnownPacks) {
        self.record("handleSelectKnownPacks", packet);
    }

    fn handle_accept_code_of_conduct(&mut self, packet: &ServerboundAcceptCodeOfConductPacket) {
        self.record("handleAcceptCodeOfConduct", packet);
    }
}
