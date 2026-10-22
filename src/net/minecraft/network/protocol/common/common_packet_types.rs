//! net/minecraft/network/protocol/common/CommonPacketTypes.java

use std::sync::LazyLock;

use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static CLIENTBOUND_CLEAR_DIALOG: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("clear_dialog"));
static CLIENTBOUND_CUSTOM_PAYLOAD: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("custom_payload"));
static CLIENTBOUND_CUSTOM_REPORT_DETAILS: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("custom_report_details"));
static CLIENTBOUND_DISCONNECT: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("disconnect"));
static CLIENTBOUND_KEEP_ALIVE: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("keep_alive"));
static CLIENTBOUND_PING: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("ping"));
static CLIENTBOUND_RESOURCE_PACK_POP: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("resource_pack_pop"));
static CLIENTBOUND_RESOURCE_PACK_PUSH: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("resource_pack_push"));
static CLIENTBOUND_POST_EFFECTS: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("post_effects"));
static CLIENTBOUND_SERVER_LINKS: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("server_links"));
static CLIENTBOUND_SHOW_DIALOG: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("show_dialog"));
static CLIENTBOUND_STORE_COOKIE: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("store_cookie"));
static CLIENTBOUND_TRANSFER: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("transfer"));
static CLIENTBOUND_UPDATE_TAGS: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_clientbound("update_tags"));
static SERVERBOUND_CLIENT_INFORMATION: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_serverbound("client_information"));
static SERVERBOUND_CUSTOM_PAYLOAD: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_serverbound("custom_payload"));
static SERVERBOUND_KEEP_ALIVE: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_serverbound("keep_alive"));
static SERVERBOUND_PONG: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_serverbound("pong"));
static SERVERBOUND_RESOURCE_PACK: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_serverbound("resource_pack"));
static SERVERBOUND_CUSTOM_CLICK_ACTION: LazyLock<PacketType> =
    LazyLock::new(|| CommonPacketTypes::create_serverbound("custom_click_action"));

/// The types of the packets configuration and play share.
pub struct CommonPacketTypes;

impl CommonPacketTypes {
    pub const CLIENTBOUND_CLEAR_DIALOG: &'static LazyLock<PacketType> = &CLIENTBOUND_CLEAR_DIALOG;
    pub const CLIENTBOUND_CUSTOM_PAYLOAD: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CUSTOM_PAYLOAD;
    pub const CLIENTBOUND_CUSTOM_REPORT_DETAILS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CUSTOM_REPORT_DETAILS;
    pub const CLIENTBOUND_DISCONNECT: &'static LazyLock<PacketType> = &CLIENTBOUND_DISCONNECT;
    pub const CLIENTBOUND_KEEP_ALIVE: &'static LazyLock<PacketType> = &CLIENTBOUND_KEEP_ALIVE;
    pub const CLIENTBOUND_PING: &'static LazyLock<PacketType> = &CLIENTBOUND_PING;
    pub const CLIENTBOUND_RESOURCE_PACK_POP: &'static LazyLock<PacketType> =
        &CLIENTBOUND_RESOURCE_PACK_POP;
    pub const CLIENTBOUND_RESOURCE_PACK_PUSH: &'static LazyLock<PacketType> =
        &CLIENTBOUND_RESOURCE_PACK_PUSH;
    pub const CLIENTBOUND_POST_EFFECTS: &'static LazyLock<PacketType> = &CLIENTBOUND_POST_EFFECTS;
    pub const CLIENTBOUND_SERVER_LINKS: &'static LazyLock<PacketType> = &CLIENTBOUND_SERVER_LINKS;
    pub const CLIENTBOUND_SHOW_DIALOG: &'static LazyLock<PacketType> = &CLIENTBOUND_SHOW_DIALOG;
    pub const CLIENTBOUND_STORE_COOKIE: &'static LazyLock<PacketType> = &CLIENTBOUND_STORE_COOKIE;
    pub const CLIENTBOUND_TRANSFER: &'static LazyLock<PacketType> = &CLIENTBOUND_TRANSFER;
    pub const CLIENTBOUND_UPDATE_TAGS: &'static LazyLock<PacketType> = &CLIENTBOUND_UPDATE_TAGS;
    pub const SERVERBOUND_CLIENT_INFORMATION: &'static LazyLock<PacketType> =
        &SERVERBOUND_CLIENT_INFORMATION;
    pub const SERVERBOUND_CUSTOM_PAYLOAD: &'static LazyLock<PacketType> =
        &SERVERBOUND_CUSTOM_PAYLOAD;
    pub const SERVERBOUND_KEEP_ALIVE: &'static LazyLock<PacketType> = &SERVERBOUND_KEEP_ALIVE;
    pub const SERVERBOUND_PONG: &'static LazyLock<PacketType> = &SERVERBOUND_PONG;
    pub const SERVERBOUND_RESOURCE_PACK: &'static LazyLock<PacketType> = &SERVERBOUND_RESOURCE_PACK;
    pub const SERVERBOUND_CUSTOM_CLICK_ACTION: &'static LazyLock<PacketType> =
        &SERVERBOUND_CUSTOM_CLICK_ACTION;

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
    use super::CommonPacketTypes;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.common.CommonPacketTypes");
        for (name, r#type) in [
            (
                "CLIENTBOUND_CLEAR_DIALOG",
                CommonPacketTypes::CLIENTBOUND_CLEAR_DIALOG,
            ),
            (
                "CLIENTBOUND_CUSTOM_PAYLOAD",
                CommonPacketTypes::CLIENTBOUND_CUSTOM_PAYLOAD,
            ),
            (
                "CLIENTBOUND_CUSTOM_REPORT_DETAILS",
                CommonPacketTypes::CLIENTBOUND_CUSTOM_REPORT_DETAILS,
            ),
            (
                "CLIENTBOUND_DISCONNECT",
                CommonPacketTypes::CLIENTBOUND_DISCONNECT,
            ),
            (
                "CLIENTBOUND_KEEP_ALIVE",
                CommonPacketTypes::CLIENTBOUND_KEEP_ALIVE,
            ),
            ("CLIENTBOUND_PING", CommonPacketTypes::CLIENTBOUND_PING),
            (
                "CLIENTBOUND_RESOURCE_PACK_POP",
                CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_POP,
            ),
            (
                "CLIENTBOUND_RESOURCE_PACK_PUSH",
                CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_PUSH,
            ),
            (
                "CLIENTBOUND_POST_EFFECTS",
                CommonPacketTypes::CLIENTBOUND_POST_EFFECTS,
            ),
            (
                "CLIENTBOUND_SERVER_LINKS",
                CommonPacketTypes::CLIENTBOUND_SERVER_LINKS,
            ),
            (
                "CLIENTBOUND_SHOW_DIALOG",
                CommonPacketTypes::CLIENTBOUND_SHOW_DIALOG,
            ),
            (
                "CLIENTBOUND_STORE_COOKIE",
                CommonPacketTypes::CLIENTBOUND_STORE_COOKIE,
            ),
            (
                "CLIENTBOUND_TRANSFER",
                CommonPacketTypes::CLIENTBOUND_TRANSFER,
            ),
            (
                "CLIENTBOUND_UPDATE_TAGS",
                CommonPacketTypes::CLIENTBOUND_UPDATE_TAGS,
            ),
            (
                "SERVERBOUND_CLIENT_INFORMATION",
                CommonPacketTypes::SERVERBOUND_CLIENT_INFORMATION,
            ),
            (
                "SERVERBOUND_CUSTOM_PAYLOAD",
                CommonPacketTypes::SERVERBOUND_CUSTOM_PAYLOAD,
            ),
            (
                "SERVERBOUND_KEEP_ALIVE",
                CommonPacketTypes::SERVERBOUND_KEEP_ALIVE,
            ),
            ("SERVERBOUND_PONG", CommonPacketTypes::SERVERBOUND_PONG),
            (
                "SERVERBOUND_RESOURCE_PACK",
                CommonPacketTypes::SERVERBOUND_RESOURCE_PACK,
            ),
            (
                "SERVERBOUND_CUSTOM_CLICK_ACTION",
                CommonPacketTypes::SERVERBOUND_CUSTOM_CLICK_ACTION,
            ),
        ] {
            golden.field(name, (**r#type).clone());
        }
        golden.finish();
    }
}
