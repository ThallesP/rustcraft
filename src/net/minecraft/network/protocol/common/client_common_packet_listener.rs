//! net/minecraft/network/protocol/common/ClientCommonPacketListener.java
//!
//! Not ported yet: `handleDisconnect`, `handleResourcePackPush` and `handleServerLinks`, whose
//! packets hold text components, and `handleUpdateTags` and `handleShowDialog`, whose packets
//! need the registries. They come with their packets.

use crate::net::minecraft::network::protocol::common::clientbound_clear_dialog_packet::ClientboundClearDialogPacket;
use crate::net::minecraft::network::protocol::common::clientbound_custom_payload_packet::ClientboundCustomPayloadPacket;
use crate::net::minecraft::network::protocol::common::clientbound_custom_report_details_packet::ClientboundCustomReportDetailsPacket;
use crate::net::minecraft::network::protocol::common::clientbound_disconnect_packet::ClientboundDisconnectPacket;
use crate::net::minecraft::network::protocol::common::clientbound_keep_alive_packet::ClientboundKeepAlivePacket;
use crate::net::minecraft::network::protocol::common::clientbound_ping_packet::ClientboundPingPacket;
use crate::net::minecraft::network::protocol::common::clientbound_post_effects_packet::ClientboundPostEffectsPacket;
use crate::net::minecraft::network::protocol::common::clientbound_resource_pack_pop_packet::ClientboundResourcePackPopPacket;
use crate::net::minecraft::network::protocol::common::clientbound_resource_pack_push_packet::ClientboundResourcePackPushPacket;
use crate::net::minecraft::network::protocol::common::clientbound_server_links_packet::ClientboundServerLinksPacket;
use crate::net::minecraft::network::protocol::common::clientbound_store_cookie_packet::ClientboundStoreCookiePacket;
use crate::net::minecraft::network::protocol::common::clientbound_transfer_packet::ClientboundTransferPacket;
use crate::net::minecraft::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener;

/// A client's listener in the protocols after login (configuration and play), for the packets
/// both have.
pub trait ClientCommonPacketListener: ClientCookiePacketListener {
    fn handle_keep_alive(&mut self, packet: &ClientboundKeepAlivePacket);

    fn handle_ping(&mut self, packet: &ClientboundPingPacket);

    fn handle_custom_payload(&mut self, packet: &ClientboundCustomPayloadPacket);

    fn handle_resource_pack_push(&mut self, packet: &ClientboundResourcePackPushPacket);

    fn handle_resource_pack_pop(&mut self, packet: &ClientboundResourcePackPopPacket);

    fn handle_server_links(&mut self, packet: &ClientboundServerLinksPacket);

    fn handle_post_effects(&mut self, packet: &ClientboundPostEffectsPacket);

    fn handle_store_cookie(&mut self, packet: &ClientboundStoreCookiePacket);

    fn handle_transfer(&mut self, packet: &ClientboundTransferPacket);

    fn handle_disconnect(&mut self, packet: &ClientboundDisconnectPacket);

    fn handle_custom_report_details(&mut self, packet: &ClientboundCustomReportDetailsPacket);

    fn handle_clear_dialog(&mut self, packet: &ClientboundClearDialogPacket);
}

#[cfg(test)]
impl ClientCommonPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_keep_alive(&mut self, packet: &ClientboundKeepAlivePacket) {
        self.record("handleKeepAlive", packet);
    }

    fn handle_ping(&mut self, packet: &ClientboundPingPacket) {
        self.record("handlePing", packet);
    }

    fn handle_custom_payload(&mut self, packet: &ClientboundCustomPayloadPacket) {
        self.record("handleCustomPayload", packet);
    }

    fn handle_resource_pack_push(&mut self, packet: &ClientboundResourcePackPushPacket) {
        self.record("handleResourcePackPush", packet);
    }

    fn handle_resource_pack_pop(&mut self, packet: &ClientboundResourcePackPopPacket) {
        self.record("handleResourcePackPop", packet);
    }

    fn handle_server_links(&mut self, packet: &ClientboundServerLinksPacket) {
        self.record("handleServerLinks", packet);
    }

    fn handle_post_effects(&mut self, packet: &ClientboundPostEffectsPacket) {
        self.record("handlePostEffects", packet);
    }

    fn handle_store_cookie(&mut self, packet: &ClientboundStoreCookiePacket) {
        self.record("handleStoreCookie", packet);
    }

    fn handle_transfer(&mut self, packet: &ClientboundTransferPacket) {
        self.record("handleTransfer", packet);
    }

    fn handle_disconnect(&mut self, packet: &ClientboundDisconnectPacket) {
        self.record("handleDisconnect", packet);
    }

    fn handle_custom_report_details(&mut self, packet: &ClientboundCustomReportDetailsPacket) {
        self.record("handleCustomReportDetails", packet);
    }

    fn handle_clear_dialog(&mut self, packet: &ClientboundClearDialogPacket) {
        self.record("handleClearDialog", packet);
    }
}
