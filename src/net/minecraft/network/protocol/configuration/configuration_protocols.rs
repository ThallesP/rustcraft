//! net/minecraft/network/protocol/configuration/ConfigurationProtocols.java
//!
//! Not ported yet: the clientbound `disconnect`, `resource_pack_push` and `server_links`, which
//! hold text components, and `update_tags` and `show_dialog`, which need the registries. The
//! clientbound protocol has the `Unported` stand-in for each: the client answers none of them
//! during a join (a resource pack push would get a `resource_pack` answer, which comes with it).

use std::sync::{Arc, LazyLock};

use crate::net::minecraft::network::connection_protocol::ConnectionProtocol;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::clientbound_disconnect_packet::ClientboundDisconnectPacket;
use crate::net::minecraft::network::protocol::common::clientbound_clear_dialog_packet::ClientboundClearDialogPacket;
use crate::net::minecraft::network::protocol::common::clientbound_custom_payload_packet::ClientboundCustomPayloadPacket;
use crate::net::minecraft::network::protocol::common::clientbound_custom_report_details_packet::ClientboundCustomReportDetailsPacket;
use crate::net::minecraft::network::protocol::common::clientbound_keep_alive_packet::ClientboundKeepAlivePacket;
use crate::net::minecraft::network::protocol::common::clientbound_ping_packet::ClientboundPingPacket;
use crate::net::minecraft::network::protocol::common::clientbound_post_effects_packet::ClientboundPostEffectsPacket;
use crate::net::minecraft::network::protocol::common::clientbound_resource_pack_pop_packet::ClientboundResourcePackPopPacket;
use crate::net::minecraft::network::protocol::common::clientbound_store_cookie_packet::ClientboundStoreCookiePacket;
use crate::net::minecraft::network::protocol::common::clientbound_transfer_packet::ClientboundTransferPacket;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::common::serverbound_client_information_packet::ServerboundClientInformationPacket;
use crate::net::minecraft::network::protocol::common::serverbound_custom_click_action_packet::ServerboundCustomClickActionPacket;
use crate::net::minecraft::network::protocol::common::serverbound_custom_payload_packet::ServerboundCustomPayloadPacket;
use crate::net::minecraft::network::protocol::common::serverbound_keep_alive_packet::ServerboundKeepAlivePacket;
use crate::net::minecraft::network::protocol::common::serverbound_pong_packet::ServerboundPongPacket;
use crate::net::minecraft::network::protocol::common::serverbound_resource_pack_packet::ServerboundResourcePackPacket;
use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
use crate::net::minecraft::network::protocol::configuration::clientbound_code_of_conduct_packet::ClientboundCodeOfConductPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_finish_configuration_packet::ClientboundFinishConfigurationPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_registry_data_packet::ClientboundRegistryDataPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_reset_chat_packet::ClientboundResetChatPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_select_known_packs::ClientboundSelectKnownPacks;
use crate::net::minecraft::network::protocol::configuration::clientbound_update_enabled_features_packet::ClientboundUpdateEnabledFeaturesPacket;
use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
use crate::net::minecraft::network::protocol::configuration::serverbound_accept_code_of_conduct_packet::ServerboundAcceptCodeOfConductPacket;
use crate::net::minecraft::network::protocol::configuration::serverbound_finish_configuration_packet::ServerboundFinishConfigurationPacket;
use crate::net::minecraft::network::protocol::configuration::serverbound_select_known_packs::ServerboundSelectKnownPacks;
use crate::net::minecraft::network::protocol::cookie::clientbound_cookie_request_packet::ClientboundCookieRequestPacket;
use crate::net::minecraft::network::protocol::cookie::cookie_packet_types::CookiePacketTypes;
use crate::net::minecraft::network::protocol::cookie::serverbound_cookie_response_packet::ServerboundCookieResponsePacket;
use crate::net::minecraft::network::protocol::protocol_info_builder::ProtocolInfoBuilder;
use crate::net::minecraft::network::protocol::simple_unbound_protocol::StaticSimpleUnboundProtocol;
use crate::net::minecraft::network::protocol::unported::Unported;
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

static CLIENTBOUND_TEMPLATE: StaticSimpleUnboundProtocol<
    dyn ClientConfigurationPacketListener,
    FriendlyByteBuf,
> = LazyLock::new(|| {
    ProtocolInfoBuilder::clientbound_protocol(ConnectionProtocol::CONFIGURATION, |builder| {
        builder
            .add_packet(
                CookiePacketTypes::CLIENTBOUND_COOKIE_REQUEST,
                ClientboundCookieRequestPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_CUSTOM_PAYLOAD,
                ClientboundCustomPayloadPacket::CONFIG_STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_DISCONNECT,
                ClientboundDisconnectPacket::STREAM_CODEC,
            )
            .add_packet(
                ConfigurationPacketTypes::CLIENTBOUND_FINISH_CONFIGURATION,
                ClientboundFinishConfigurationPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_KEEP_ALIVE,
                ClientboundKeepAlivePacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_PING,
                ClientboundPingPacket::STREAM_CODEC,
            )
            .add_packet(
                ConfigurationPacketTypes::CLIENTBOUND_RESET_CHAT,
                ClientboundResetChatPacket::STREAM_CODEC,
            )
            .add_packet(
                ConfigurationPacketTypes::CLIENTBOUND_REGISTRY_DATA,
                ClientboundRegistryDataPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_POP,
                ClientboundResourcePackPopPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_PUSH,
                Unported::codec(CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_PUSH),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_POST_EFFECTS,
                ClientboundPostEffectsPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_STORE_COOKIE,
                ClientboundStoreCookiePacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_TRANSFER,
                ClientboundTransferPacket::STREAM_CODEC,
            )
            .add_packet(
                ConfigurationPacketTypes::CLIENTBOUND_UPDATE_ENABLED_FEATURES,
                ClientboundUpdateEnabledFeaturesPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_UPDATE_TAGS,
                Unported::codec(CommonPacketTypes::CLIENTBOUND_UPDATE_TAGS),
            )
            .add_packet(
                ConfigurationPacketTypes::CLIENTBOUND_SELECT_KNOWN_PACKS,
                ClientboundSelectKnownPacks::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_CUSTOM_REPORT_DETAILS,
                ClientboundCustomReportDetailsPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_SERVER_LINKS,
                Unported::codec(CommonPacketTypes::CLIENTBOUND_SERVER_LINKS),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_CLEAR_DIALOG,
                ClientboundClearDialogPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_SHOW_DIALOG,
                Unported::codec(CommonPacketTypes::CLIENTBOUND_SHOW_DIALOG),
            )
            .add_packet(
                ConfigurationPacketTypes::CLIENTBOUND_CODE_OF_CONDUCT,
                ClientboundCodeOfConductPacket::STREAM_CODEC,
            );
    })
});

static CLIENTBOUND: StaticProtocolInfo<dyn ClientConfigurationPacketListener> =
    LazyLock::new(|| CLIENTBOUND_TEMPLATE.bind(Arc::new(FriendlyByteBuf::new)));

/// Configuration: after login, the server sends the registries, tags and feature flags, and the
/// client answers with its settings, brand and known packs, until the server finishes it and both
/// move on to play.
pub struct ConfigurationProtocols;

impl ConfigurationProtocols {
    pub const CLIENTBOUND_TEMPLATE: &'static StaticSimpleUnboundProtocol<
        dyn ClientConfigurationPacketListener,
        FriendlyByteBuf,
    > = &CLIENTBOUND_TEMPLATE;
    pub const CLIENTBOUND: &'static StaticProtocolInfo<dyn ClientConfigurationPacketListener> =
        &CLIENTBOUND;
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
        golden.field(
            "CLIENTBOUND_TEMPLATE",
            listed(ConfigurationProtocols::CLIENTBOUND_TEMPLATE.details()),
        );
        golden.field(
            "CLIENTBOUND",
            (
                ConfigurationProtocols::CLIENTBOUND.id(),
                ConfigurationProtocols::CLIENTBOUND.flow(),
            ),
        );
        golden.pending(
            "CLIENTBOUND.codec()",
            "disconnect, resource_pack_push and server_links hold text components, and \
             update_tags and show_dialog need the registries",
        );
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
