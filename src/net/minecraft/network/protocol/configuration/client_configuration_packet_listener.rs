//! net/minecraft/network/protocol/configuration/ClientConfigurationPacketListener.java
//!
//! Not ported yet: `handleEnabledFeatures`, whose packet holds a Java `HashSet`, which comes with
//! it, and its `protocol()`, which comes with `PacketListener`'s.

use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::configuration::clientbound_code_of_conduct_packet::ClientboundCodeOfConductPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_finish_configuration_packet::ClientboundFinishConfigurationPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_registry_data_packet::ClientboundRegistryDataPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_reset_chat_packet::ClientboundResetChatPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_select_known_packs::ClientboundSelectKnownPacks;

/// The client's listener while the server configures it after login.
pub trait ClientConfigurationPacketListener: ClientCommonPacketListener {
    fn handle_code_of_conduct(&mut self, packet: &ClientboundCodeOfConductPacket);

    fn handle_configuration_finished(&mut self, packet: &ClientboundFinishConfigurationPacket);

    fn handle_registry_data(&mut self, packet: &ClientboundRegistryDataPacket);

    fn handle_select_known_packs(&mut self, packet: &ClientboundSelectKnownPacks);

    fn handle_reset_chat(&mut self, packet: &ClientboundResetChatPacket);
}

#[cfg(test)]
impl ClientConfigurationPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_code_of_conduct(&mut self, packet: &ClientboundCodeOfConductPacket) {
        self.record("handleCodeOfConduct", packet);
    }

    fn handle_configuration_finished(&mut self, packet: &ClientboundFinishConfigurationPacket) {
        self.record("handleConfigurationFinished", packet);
    }

    fn handle_registry_data(&mut self, packet: &ClientboundRegistryDataPacket) {
        self.record("handleRegistryData", packet);
    }

    fn handle_select_known_packs(&mut self, packet: &ClientboundSelectKnownPacks) {
        self.record("handleSelectKnownPacks", packet);
    }

    fn handle_reset_chat(&mut self, packet: &ClientboundResetChatPacket) {
        self.record("handleResetChat", packet);
    }
}
