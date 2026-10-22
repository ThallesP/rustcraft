//! net/minecraft/network/protocol/configuration/ConfigurationPacketTypes.java

use std::sync::LazyLock;

use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static CLIENTBOUND_CODE_OF_CONDUCT: LazyLock<PacketType> =
    LazyLock::new(|| ConfigurationPacketTypes::create_clientbound("code_of_conduct"));
static CLIENTBOUND_FINISH_CONFIGURATION: LazyLock<PacketType> =
    LazyLock::new(|| ConfigurationPacketTypes::create_clientbound("finish_configuration"));
static CLIENTBOUND_REGISTRY_DATA: LazyLock<PacketType> =
    LazyLock::new(|| ConfigurationPacketTypes::create_clientbound("registry_data"));
static CLIENTBOUND_RESET_CHAT: LazyLock<PacketType> =
    LazyLock::new(|| ConfigurationPacketTypes::create_clientbound("reset_chat"));
static CLIENTBOUND_SELECT_KNOWN_PACKS: LazyLock<PacketType> =
    LazyLock::new(|| ConfigurationPacketTypes::create_clientbound("select_known_packs"));
static CLIENTBOUND_UPDATE_ENABLED_FEATURES: LazyLock<PacketType> =
    LazyLock::new(|| ConfigurationPacketTypes::create_clientbound("update_enabled_features"));
static SERVERBOUND_ACCEPT_CODE_OF_CONDUCT: LazyLock<PacketType> =
    LazyLock::new(|| ConfigurationPacketTypes::create_serverbound("accept_code_of_conduct"));
static SERVERBOUND_FINISH_CONFIGURATION: LazyLock<PacketType> =
    LazyLock::new(|| ConfigurationPacketTypes::create_serverbound("finish_configuration"));
static SERVERBOUND_SELECT_KNOWN_PACKS: LazyLock<PacketType> =
    LazyLock::new(|| ConfigurationPacketTypes::create_serverbound("select_known_packs"));

/// The types of the packets only the configuration protocol has. The ones it shares with play
/// are `CommonPacketTypes`' and `CookiePacketTypes`'.
pub struct ConfigurationPacketTypes;

impl ConfigurationPacketTypes {
    pub const CLIENTBOUND_CODE_OF_CONDUCT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CODE_OF_CONDUCT;
    pub const CLIENTBOUND_FINISH_CONFIGURATION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_FINISH_CONFIGURATION;
    pub const CLIENTBOUND_REGISTRY_DATA: &'static LazyLock<PacketType> = &CLIENTBOUND_REGISTRY_DATA;
    pub const CLIENTBOUND_RESET_CHAT: &'static LazyLock<PacketType> = &CLIENTBOUND_RESET_CHAT;
    pub const CLIENTBOUND_SELECT_KNOWN_PACKS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SELECT_KNOWN_PACKS;
    pub const CLIENTBOUND_UPDATE_ENABLED_FEATURES: &'static LazyLock<PacketType> =
        &CLIENTBOUND_UPDATE_ENABLED_FEATURES;
    pub const SERVERBOUND_ACCEPT_CODE_OF_CONDUCT: &'static LazyLock<PacketType> =
        &SERVERBOUND_ACCEPT_CODE_OF_CONDUCT;
    pub const SERVERBOUND_FINISH_CONFIGURATION: &'static LazyLock<PacketType> =
        &SERVERBOUND_FINISH_CONFIGURATION;
    pub const SERVERBOUND_SELECT_KNOWN_PACKS: &'static LazyLock<PacketType> =
        &SERVERBOUND_SELECT_KNOWN_PACKS;

    fn create_clientbound(id: &str) -> PacketType {
        PacketType::new(
            PacketFlow::CLIENTBOUND,
            Identifier::with_default_namespace(id),
        )
    }

    fn create_serverbound(id: &str) -> PacketType {
        PacketType::new(
            PacketFlow::SERVERBOUND,
            Identifier::with_default_namespace(id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigurationPacketTypes;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.configuration.ConfigurationPacketTypes");
        for (name, r#type) in [
            (
                "CLIENTBOUND_CODE_OF_CONDUCT",
                ConfigurationPacketTypes::CLIENTBOUND_CODE_OF_CONDUCT,
            ),
            (
                "CLIENTBOUND_FINISH_CONFIGURATION",
                ConfigurationPacketTypes::CLIENTBOUND_FINISH_CONFIGURATION,
            ),
            (
                "CLIENTBOUND_REGISTRY_DATA",
                ConfigurationPacketTypes::CLIENTBOUND_REGISTRY_DATA,
            ),
            (
                "CLIENTBOUND_RESET_CHAT",
                ConfigurationPacketTypes::CLIENTBOUND_RESET_CHAT,
            ),
            (
                "CLIENTBOUND_SELECT_KNOWN_PACKS",
                ConfigurationPacketTypes::CLIENTBOUND_SELECT_KNOWN_PACKS,
            ),
            (
                "CLIENTBOUND_UPDATE_ENABLED_FEATURES",
                ConfigurationPacketTypes::CLIENTBOUND_UPDATE_ENABLED_FEATURES,
            ),
            (
                "SERVERBOUND_ACCEPT_CODE_OF_CONDUCT",
                ConfigurationPacketTypes::SERVERBOUND_ACCEPT_CODE_OF_CONDUCT,
            ),
            (
                "SERVERBOUND_FINISH_CONFIGURATION",
                ConfigurationPacketTypes::SERVERBOUND_FINISH_CONFIGURATION,
            ),
            (
                "SERVERBOUND_SELECT_KNOWN_PACKS",
                ConfigurationPacketTypes::SERVERBOUND_SELECT_KNOWN_PACKS,
            ),
        ] {
            golden.field(name, (**r#type).clone());
        }
        golden.finish();
    }
}
