//! net/minecraft/network/protocol/configuration/ClientboundCodeOfConductPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::object::HashCode;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundCodeOfConductPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            ByteBufCodecs::STRING_UTF8,
            |packet: &ClientboundCodeOfConductPacket| packet.code_of_conduct().to_owned(),
            ClientboundCodeOfConductPacket::new,
        ))
    });

/// A server's code of conduct, which the client shows and answers with a
/// `ServerboundAcceptCodeOfConductPacket` once the player accepts it.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundCodeOfConductPacket {
    code_of_conduct: String,
}

impl ClientboundCodeOfConductPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundCodeOfConductPacket> =
        &STREAM_CODEC;

    pub fn new(code_of_conduct: String) -> ClientboundCodeOfConductPacket {
        ClientboundCodeOfConductPacket { code_of_conduct }
    }

    pub fn code_of_conduct(&self) -> &str {
        &self.code_of_conduct
    }

    /// A record's of one component: its hash.
    pub fn hash_code(&self) -> i32 {
        self.code_of_conduct.hash_code()
    }
}

impl<L: ClientConfigurationPacketListener + ?Sized> Packet<L> for ClientboundCodeOfConductPacket {
    fn r#type(&self) -> &'static PacketType {
        ConfigurationPacketTypes::CLIENTBOUND_CODE_OF_CONDUCT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_code_of_conduct(self);
    }
}

/// A record's: `ClientboundCodeOfConductPacket[codeOfConduct=…]`.
impl fmt::Display for ClientboundCodeOfConductPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundCodeOfConductPacket[codeOfConduct={}]",
            self.code_of_conduct
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundCodeOfConductPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ClientboundCodeOfConductPacket {
        code_of_conduct: String
    });

    type Listener = dyn ClientConfigurationPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundCodeOfConductPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Ljava/lang/String;)V",
            ClientboundCodeOfConductPacket::new,
        );
        golden.check_method(
            "codeOfConduct()Ljava/lang/String;",
            |this: &mut ClientboundCodeOfConductPacket| this.code_of_conduct().to_owned(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundCodeOfConductPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/configuration/ClientConfigurationPacketListener;)V",
            |this: &mut ClientboundCodeOfConductPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundCodeOfConductPacket, other: ClientboundCodeOfConductPacket| {
                *this == other
            },
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ClientboundCodeOfConductPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundCodeOfConductPacket| this.to_string(),
        );
        golden.finish();
    }
}
