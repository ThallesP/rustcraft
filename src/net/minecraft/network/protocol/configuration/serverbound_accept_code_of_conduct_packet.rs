//! net/minecraft/network/protocol/configuration/ServerboundAcceptCodeOfConductPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::configuration::configuration_packet_types::ConfigurationPacketTypes;
use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ServerboundAcceptCodeOfConductPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::unit(
            ServerboundAcceptCodeOfConductPacket::INSTANCE,
        ))
    });

/// The client accepting the code of conduct a server asked it to with a
/// `ClientboundCodeOfConductPacket`. It has no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundAcceptCodeOfConductPacket;

impl ServerboundAcceptCodeOfConductPacket {
    pub const INSTANCE: ServerboundAcceptCodeOfConductPacket = ServerboundAcceptCodeOfConductPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        ServerboundAcceptCodeOfConductPacket,
    > = &STREAM_CODEC;

    pub fn new() -> ServerboundAcceptCodeOfConductPacket {
        ServerboundAcceptCodeOfConductPacket
    }

    /// A record's of no components: 0.
    pub fn hash_code(&self) -> i32 {
        0
    }
}

impl Default for ServerboundAcceptCodeOfConductPacket {
    fn default() -> ServerboundAcceptCodeOfConductPacket {
        ServerboundAcceptCodeOfConductPacket::new()
    }
}

impl<L: ServerConfigurationPacketListener + ?Sized> Packet<L>
    for ServerboundAcceptCodeOfConductPacket
{
    fn r#type(&self) -> &'static PacketType {
        ConfigurationPacketTypes::SERVERBOUND_ACCEPT_CODE_OF_CONDUCT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_accept_code_of_conduct(self);
    }
}

/// A record's: `ServerboundAcceptCodeOfConductPacket[]`.
impl fmt::Display for ServerboundAcceptCodeOfConductPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ServerboundAcceptCodeOfConductPacket[]")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ServerboundAcceptCodeOfConductPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record of no components: [].
    impl Decode for ServerboundAcceptCodeOfConductPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ServerboundAcceptCodeOfConductPacket::INSTANCE
        }
    }

    impl Encode for ServerboundAcceptCodeOfConductPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ServerConfigurationPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket",
        );
        golden.field("INSTANCE", ServerboundAcceptCodeOfConductPacket::INSTANCE);
        golden.check_codec("STREAM_CODEC", || {
            ServerboundAcceptCodeOfConductPacket::STREAM_CODEC
        });
        golden.check("<init>()V", ServerboundAcceptCodeOfConductPacket::new);
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundAcceptCodeOfConductPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/configuration/ServerConfigurationPacketListener;)V",
            |this: &mut ServerboundAcceptCodeOfConductPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundAcceptCodeOfConductPacket,
             other: ServerboundAcceptCodeOfConductPacket| *this == other,
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ServerboundAcceptCodeOfConductPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundAcceptCodeOfConductPacket| this.to_string(),
        );
        golden.finish();
    }
}
