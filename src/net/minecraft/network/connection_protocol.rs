//! net/minecraft/network/ConnectionProtocol.java

/// The protocol a connection speaks, which changes as it goes from the handshake to play.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol {
    HANDSHAKING,
    PLAY,
    STATUS,
    LOGIN,
    CONFIGURATION,
}

impl ConnectionProtocol {
    pub const VALUES: [ConnectionProtocol; 5] = [
        ConnectionProtocol::HANDSHAKING,
        ConnectionProtocol::PLAY,
        ConnectionProtocol::STATUS,
        ConnectionProtocol::LOGIN,
        ConnectionProtocol::CONFIGURATION,
    ];

    pub fn values() -> [ConnectionProtocol; 5] {
        ConnectionProtocol::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> ConnectionProtocol {
        match name {
            "HANDSHAKING" => ConnectionProtocol::HANDSHAKING,
            "PLAY" => ConnectionProtocol::PLAY,
            "STATUS" => ConnectionProtocol::STATUS,
            "LOGIN" => ConnectionProtocol::LOGIN,
            "CONFIGURATION" => ConnectionProtocol::CONFIGURATION,
            _ => panic!("IllegalArgumentException: no ConnectionProtocol constant {name}"),
        }
    }

    /// The name packet reports and logs use, like `handshake`.
    pub fn id(&self) -> &'static str {
        match self {
            ConnectionProtocol::HANDSHAKING => "handshake",
            ConnectionProtocol::PLAY => "play",
            ConnectionProtocol::STATUS => "status",
            ConnectionProtocol::LOGIN => "login",
            ConnectionProtocol::CONFIGURATION => "configuration",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectionProtocol;
    use crate::golden::{Golden, encode_names};

    encode_names!(ConnectionProtocol {
        HANDSHAKING,
        PLAY,
        STATUS,
        LOGIN,
        CONFIGURATION
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.ConnectionProtocol");
        for protocol in ConnectionProtocol::VALUES {
            golden.field(&format!("{protocol:?}"), protocol);
        }
        golden.check(
            "values()[Lnet/minecraft/network/ConnectionProtocol;",
            ConnectionProtocol::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/network/ConnectionProtocol;",
            |name: String| ConnectionProtocol::value_of(&name),
        );
        golden.check_method("id()Ljava/lang/String;", |this: &mut ConnectionProtocol| {
            this.id().to_owned()
        });
        golden.finish();
    }
}
