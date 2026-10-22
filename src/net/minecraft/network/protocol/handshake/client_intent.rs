//! net/minecraft/network/protocol/handshake/ClientIntent.java

use std::fmt;

/// What a client connects for, which the handshake's last field says: the server's status, to
/// log in, or to log in after a transfer from another server.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClientIntent {
    STATUS,
    LOGIN,
    TRANSFER,
}

impl ClientIntent {
    const STATUS_ID: i32 = 1;
    const LOGIN_ID: i32 = 2;
    const TRANSFER_ID: i32 = 3;

    pub const VALUES: [ClientIntent; 3] = [
        ClientIntent::STATUS,
        ClientIntent::LOGIN,
        ClientIntent::TRANSFER,
    ];

    pub fn values() -> [ClientIntent; 3] {
        ClientIntent::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> ClientIntent {
        match name {
            "STATUS" => ClientIntent::STATUS,
            "LOGIN" => ClientIntent::LOGIN,
            "TRANSFER" => ClientIntent::TRANSFER,
            _ => panic!("IllegalArgumentException: no ClientIntent constant {name}"),
        }
    }

    /// Panics on an unknown id, where Java throws `IllegalArgumentException`.
    pub fn by_id(id: i32) -> ClientIntent {
        match id {
            ClientIntent::STATUS_ID => ClientIntent::STATUS,
            ClientIntent::LOGIN_ID => ClientIntent::LOGIN,
            ClientIntent::TRANSFER_ID => ClientIntent::TRANSFER,
            _ => panic!("IllegalArgumentException: Unknown connection intent: {id}"),
        }
    }

    pub fn id(&self) -> i32 {
        match self {
            ClientIntent::STATUS => ClientIntent::STATUS_ID,
            ClientIntent::LOGIN => ClientIntent::LOGIN_ID,
            ClientIntent::TRANSFER => ClientIntent::TRANSFER_ID,
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for ClientIntent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientIntent;
    use crate::golden::{Golden, encode_names};

    encode_names!(ClientIntent {
        STATUS,
        LOGIN,
        TRANSFER
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.handshake.ClientIntent");
        for intent in ClientIntent::VALUES {
            golden.field(&format!("{intent:?}"), intent);
        }
        golden.check(
            "values()[Lnet/minecraft/network/protocol/handshake/ClientIntent;",
            ClientIntent::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/network/protocol/handshake/ClientIntent;",
            |name: String| ClientIntent::value_of(&name),
        );
        golden.check(
            "byId(I)Lnet/minecraft/network/protocol/handshake/ClientIntent;",
            ClientIntent::by_id,
        );
        golden.check_method("id()I", |this: &mut ClientIntent| this.id());
        golden.finish();
    }
}
