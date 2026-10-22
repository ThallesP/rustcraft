//! net/minecraft/network/protocol/common/ServerboundClientInformationPacket.java
//!
//! Not ported: `hashCode`, a record's, which hashes its information's, and so its enum constants'
//! identity hashes, which change from run to run. Nothing hashes a packet.

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::server::level::client_information::ClientInformation;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundClientInformationPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundClientInformationPacket::write,
            ServerboundClientInformationPacket::from_friendly_byte_buf,
        ))
    });

/// The client's options the server needs, which it sends in configuration and whenever they
/// change in play.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundClientInformationPacket {
    information: ClientInformation,
}

impl ServerboundClientInformationPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundClientInformationPacket,
    > = &STREAM_CODEC;

    pub fn new(information: ClientInformation) -> ServerboundClientInformationPacket {
        ServerboundClientInformationPacket { information }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ServerboundClientInformationPacket {
        ServerboundClientInformationPacket::new(ClientInformation::from_friendly_byte_buf(input))
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        self.information.write(output);
    }

    pub fn information(&self) -> &ClientInformation {
        &self.information
    }
}

impl<L: ServerCommonPacketListener + ?Sized> Packet<L> for ServerboundClientInformationPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::SERVERBOUND_CLIENT_INFORMATION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_client_information(self);
    }
}

/// A record's: `ServerboundClientInformationPacket[information=ClientInformation[…]]`.
impl fmt::Display for ServerboundClientInformationPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ServerboundClientInformationPacket[information={}]",
            self.information
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundClientInformationPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::server::level::client_information::ClientInformation;

    // A record: its components.
    encode_fields!(ServerboundClientInformationPacket {
        information: ClientInformation
    });

    type Listener = dyn ServerCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.common.ServerboundClientInformationPacket",
        );
        golden.pending(
            "hashCode()I",
            "hashes an enum constant's identity hash, which changes from run to run",
        );
        golden.check_codec("STREAM_CODEC", || {
            ServerboundClientInformationPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/server/level/ClientInformation;)V",
            ServerboundClientInformationPacket::new,
        );
        golden.check_method(
            "information()Lnet/minecraft/server/level/ClientInformation;",
            |this: &mut ServerboundClientInformationPacket| this.information().clone(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundClientInformationPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ServerCommonPacketListener;)V",
            |this: &mut ServerboundClientInformationPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundClientInformationPacket,
             other: ServerboundClientInformationPacket| *this == other,
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundClientInformationPacket| this.to_string(),
        );
        golden.finish();
    }
}
