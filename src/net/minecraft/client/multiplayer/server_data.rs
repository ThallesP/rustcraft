//! net/minecraft/client/multiplayer/ServerData.java
//!
//! Not ported yet: the accepted codes of conduct, the icon, the state,
//! saving the server list, and the parts of the status a ping fills in that are text components
//! (`status`, `motd`, `version`, `playerList`) or come from the status's parts (`players`,
//! `protocol`): `ServerStatus` holds its JSON until text components come, and so does
//! `server_status`, which stands in for them.

use crate::net::minecraft::network::protocol::status::server_status::ServerStatus;

/// A server the player joins, as the server list has it.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerData {
    pub name: String,
    pub ip: String,
    /// The round trip of the last ping, in milliseconds.
    pub ping: i64,
    /// The status the last ping got, standing in for the fields Java fills in from it.
    pub server_status: Option<ServerStatus>,
    pack_status: ServerPackStatus,
    r#type: Type,
}

impl ServerData {
    pub fn new(name: String, ip: String, r#type: Type) -> ServerData {
        ServerData {
            name,
            ip,
            ping: 0,
            server_status: None,
            pack_status: ServerPackStatus::PROMPT,
            r#type,
        }
    }

    pub fn get_resource_pack_status(&self) -> ServerPackStatus {
        self.pack_status
    }

    pub fn set_resource_pack_status(&mut self, pack_status: ServerPackStatus) {
        self.pack_status = pack_status;
    }

    pub fn r#type(&self) -> Type {
        self.r#type
    }

    pub fn is_lan(&self) -> bool {
        self.r#type == Type::LAN
    }

    pub fn is_realm(&self) -> bool {
        self.r#type == Type::REALM
    }

    /// Whether the player accepted this code of conduct before. The server list isn't ported,
    /// so nothing has been accepted.
    pub fn has_accepted_code_of_conduct(&self, _code_of_conduct: &str) -> bool {
        false
    }
}

/// `ServerData.ServerPackStatus`: what the player chose for the server's resource packs.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerPackStatus {
    ENABLED,
    DISABLED,
    PROMPT,
}

/// `ServerData.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    LAN,
    REALM,
    OTHER,
}
