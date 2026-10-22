//! net/minecraft/network/protocol/configuration/ConfigurationProtocols.java
//!
//! Not ported yet: `CLIENTBOUND_TEMPLATE` and `CLIENTBOUND`, whose packets include the registry
//! data, the tags and the dialogs, which need the registries.

use std::sync::{Arc, LazyLock};

use crate::net::minecraft::network::connection_protocol::ConnectionProtocol;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::common::serverbound_client_information_packet::ServerboundClientInformationPacket;
use crate::net::minecraft::network::protocol::common::serverbound_custom_click_action_packet::ServerboundCustomClickActionPacket;
use crate::net::minecraft::network::protocol::common::serverbound_custom_payload_packet::ServerboundCustomPayloadPacket;
use crate::net::minecraft::network::protocol::common::serverbound_keep_alive_packet::ServerboundKeepAlivePacket;
use crate::net::minecraft::network::protocol::common::serverbound_pong_packet::ServerboundPongPacket;
use crate::net::minecraft::network::protocol::common::serverbound_resource_pack_packet::ServerboundResourcePackPacket;
use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
use crate::net::minecraft::network::protocol::configuration::serverbound_accept_code_of_conduct_packet::ServerboundAcceptCodeOfConductPacket;
use crate::net::minecraft::network::protocol::configuration::serverbound_finish_configuration_packet::ServerboundFinishConfigurationPacket;
use crate::net::minecraft::network::protocol::configuration::serverbound_select_known_packs::ServerboundSelectKnownPacks;
use crate::net::minecraft::network::protocol::cookie::cookie_packet_types::CookiePacketTypes;
use crate::net::minecraft::network::protocol::cookie::serverbound_cookie_response_packet::ServerboundCookieResponsePacket;
use crate::net::minecraft::network::protocol::protocol_info_builder::ProtocolInfoBuilder;
use crate::net::minecraft::network::protocol::simple_unbound_protocol::StaticSimpleUnboundProtocol;
use crate::net::minecraft::network::protocol_info::StaticProtocolInfo;

static SERVERBOUND_TEMPLATE: StaticSimpleUnboundProtocol<
    dyn ServerConfigurationPacketListener,
    FriendlyByteBuf,
> = LazyLock::new(|| {
    ProtocolInfoBuilder::serverbound_protocol(ConnectionProtocol::CONFIGURATION, |builder| {
        builder
            .add_packet(
                CommonPacketTypes::SERVERBOUND_CLIENT_INFORMATION,
                ServerboundClientInformationPacket::STREAM_CODEC,
            )
            .add_packet(
                CookiePacketTypes::SERVERBOUND_COOKIE_RESPONSE,
                ServerboundCookieResponsePacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_CUSTOM_PAYLOAD,
                ServerboundCustomPayloadPacket::STREAM_CODEC,
            )
            .add_packet(
                ConfigurationPacketTypes::SERVERBOUND_FINISH_CONFIGURATION,
                ServerboundFinishConfigurationPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_KEEP_ALIVE,
                ServerboundKeepAlivePacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_PONG,
                ServerboundPongPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_RESOURCE_PACK,
                ServerboundResourcePackPacket::STREAM_CODEC,
            )
            .add_packet(
                ConfigurationPacketTypes::SERVERBOUND_SELECT_KNOWN_PACKS,
                ServerboundSelectKnownPacks::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_CUSTOM_CLICK_ACTION,
                ServerboundCustomClickActionPacket::STREAM_CODEC,
            )
            .add_packet(
                ConfigurationPacketTypes::SERVERBOUND_ACCEPT_CODE_OF_CONDUCT,
                ServerboundAcceptCodeOfConductPacket::STREAM_CODEC,
            );
    })
});

static SERVERBOUND: StaticProtocolInfo<dyn ServerConfigurationPacketListener> =
    LazyLock::new(|| SERVERBOUND_TEMPLATE.bind(Arc::new(FriendlyByteBuf::new)));

/// Configuration: after login, the server sends the registries, tags and feature flags, and the
/// client answers with its settings, brand and known packs, until the server finishes it and both
/// move on to play.
pub struct ConfigurationProtocols;

impl ConfigurationProtocols {
    pub const SERVERBOUND_TEMPLATE: &'static StaticSimpleUnboundProtocol<
        dyn ServerConfigurationPacketListener,
        FriendlyByteBuf,
    > = &SERVERBOUND_TEMPLATE;
    pub const SERVERBOUND: &'static StaticProtocolInfo<dyn ServerConfigurationPacketListener> =
        &SERVERBOUND;
}

#[cfg(test)]
mod tests {
    use super::ConfigurationProtocols;
    use crate::golden::{Golden, encode_packets};
    use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
    use crate::net::minecraft::network::protocol::common::serverbound_client_information_packet::ServerboundClientInformationPacket;
    use crate::net::minecraft::network::protocol::common::serverbound_custom_click_action_packet::ServerboundCustomClickActionPacket;
    use crate::net::minecraft::network::protocol::common::serverbound_custom_payload_packet::ServerboundCustomPayloadPacket;
    use crate::net::minecraft::network::protocol::common::serverbound_keep_alive_packet::ServerboundKeepAlivePacket;
    use crate::net::minecraft::network::protocol::common::serverbound_pong_packet::ServerboundPongPacket;
    use crate::net::minecraft::network::protocol::common::serverbound_resource_pack_packet::ServerboundResourcePackPacket;
    use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
    use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
    use crate::net::minecraft::network::protocol::configuration::serverbound_accept_code_of_conduct_packet::ServerboundAcceptCodeOfConductPacket;
    use crate::net::minecraft::network::protocol::configuration::serverbound_finish_configuration_packet::ServerboundFinishConfigurationPacket;
    use crate::net::minecraft::network::protocol::configuration::serverbound_select_known_packs::ServerboundSelectKnownPacks;
    use crate::net::minecraft::network::protocol::cookie::cookie_packet_types::CookiePacketTypes;
    use crate::net::minecraft::network::protocol::cookie::serverbound_cookie_response_packet::ServerboundCookieResponsePacket;
    use crate::net::minecraft::network::protocol_info::tests::listed;

    encode_packets!(dyn ServerConfigurationPacketListener {
        CommonPacketTypes::SERVERBOUND_CLIENT_INFORMATION => ServerboundClientInformationPacket,
        CookiePacketTypes::SERVERBOUND_COOKIE_RESPONSE => ServerboundCookieResponsePacket,
        CommonPacketTypes::SERVERBOUND_CUSTOM_PAYLOAD => ServerboundCustomPayloadPacket,
        ConfigurationPacketTypes::SERVERBOUND_FINISH_CONFIGURATION => ServerboundFinishConfigurationPacket,
        CommonPacketTypes::SERVERBOUND_KEEP_ALIVE => ServerboundKeepAlivePacket,
        CommonPacketTypes::SERVERBOUND_PONG => ServerboundPongPacket,
        CommonPacketTypes::SERVERBOUND_RESOURCE_PACK => ServerboundResourcePackPacket,
        ConfigurationPacketTypes::SERVERBOUND_SELECT_KNOWN_PACKS => ServerboundSelectKnownPacks,
        CommonPacketTypes::SERVERBOUND_CUSTOM_CLICK_ACTION => ServerboundCustomClickActionPacket,
        ConfigurationPacketTypes::SERVERBOUND_ACCEPT_CODE_OF_CONDUCT => ServerboundAcceptCodeOfConductPacket,
    });

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.configuration.ConfigurationProtocols");
        let clientbound = "its packets include the registry data, the tags and the dialogs, \
            which need the registries";
        golden.pending("CLIENTBOUND_TEMPLATE", clientbound);
        golden.pending("CLIENTBOUND", clientbound);
        golden.pending("CLIENTBOUND.codec()", clientbound);
        golden.field(
            "SERVERBOUND_TEMPLATE",
            listed(ConfigurationProtocols::SERVERBOUND_TEMPLATE.details()),
        );
        golden.field(
            "SERVERBOUND",
            (
                ConfigurationProtocols::SERVERBOUND.id(),
                ConfigurationProtocols::SERVERBOUND.flow(),
            ),
        );
        golden.check_codec("SERVERBOUND.codec()", || {
            ConfigurationProtocols::SERVERBOUND.codec()
        });
        golden.finish();
    }
}
