//! net/minecraft/network/protocol/PacketFlow.java

/// Which way a packet goes: to the server or to the client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PacketFlow {
    SERVERBOUND,
    CLIENTBOUND,
}

impl PacketFlow {
    pub const VALUES: [PacketFlow; 2] = [PacketFlow::SERVERBOUND, PacketFlow::CLIENTBOUND];

    pub fn values() -> [PacketFlow; 2] {
        PacketFlow::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> PacketFlow {
        match name {
            "SERVERBOUND" => PacketFlow::SERVERBOUND,
            "CLIENTBOUND" => PacketFlow::CLIENTBOUND,
            _ => panic!("IllegalArgumentException: no PacketFlow constant {name}"),
        }
    }

    pub fn get_opposite(&self) -> PacketFlow {
        if *self == PacketFlow::CLIENTBOUND {
            PacketFlow::SERVERBOUND
        } else {
            PacketFlow::CLIENTBOUND
        }
    }

    /// The name a `PacketType` prints first, like `serverbound`.
    pub fn id(&self) -> &'static str {
        match self {
            PacketFlow::SERVERBOUND => "serverbound",
            PacketFlow::CLIENTBOUND => "clientbound",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PacketFlow;
    use crate::golden::{Golden, encode_names};

    encode_names!(PacketFlow {
        SERVERBOUND,
        CLIENTBOUND
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.PacketFlow");
        for flow in PacketFlow::VALUES {
            golden.field(&format!("{flow:?}"), flow);
        }
        golden.check(
            "values()[Lnet/minecraft/network/protocol/PacketFlow;",
            PacketFlow::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/network/protocol/PacketFlow;",
            |name: String| PacketFlow::value_of(&name),
        );
        golden.check_method(
            "getOpposite()Lnet/minecraft/network/protocol/PacketFlow;",
            |this: &mut PacketFlow| this.get_opposite(),
        );
        golden.check_method("id()Ljava/lang/String;", |this: &mut PacketFlow| {
            this.id().to_owned()
        });
        golden.finish();
    }
}
