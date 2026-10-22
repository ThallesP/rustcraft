//! net/minecraft/client/multiplayer/ServerData.java
//!
//! Not ported yet: everything but the name, address and type: the status a ping fills in, the
//! resource pack setting, the accepted codes of conduct, and saving the server list.

/// A server the player joins, as the server list has it.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerData {
    pub name: String,
    pub ip: String,
    r#type: Type,
}

impl ServerData {
    pub fn new(name: String, ip: String, r#type: Type) -> ServerData {
        ServerData { name, ip, r#type }
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
