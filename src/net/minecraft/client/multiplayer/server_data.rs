//! net/minecraft/client/multiplayer/ServerData.java
//!
//! Not ported yet: the resource pack setting, the accepted codes of conduct, the icon, the state,
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
    r#type: Type,
}

impl ServerData {
    pub fn new(name: String, ip: String, r#type: Type) -> ServerData {
        ServerData {
            name,
            ip,
            ping: 0,
            server_status: None,
            r#type,
        }
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

/// `ServerData.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    LAN,
    REALM,
    OTHER,
}
