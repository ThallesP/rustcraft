//! net/minecraft/network/protocol/common/ClientboundCustomReportDetailsPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::util::hash_map::HashMap;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

const MAX_DETAIL_KEY_LENGTH: i32 = 128;
const MAX_DETAIL_VALUE_LENGTH: i32 = 4096;
const MAX_DETAIL_COUNT: i32 = 32;

static DETAILS_STREAM_CODEC: StaticStreamCodec<ByteBuf, HashMap<String, String>> =
    LazyLock::new(|| {
        Box::new(ByteBufCodecs::map_max(
            HashMap::from_initial_capacity,
            ByteBufCodecs::string_utf8(MAX_DETAIL_KEY_LENGTH),
            ByteBufCodecs::string_utf8(MAX_DETAIL_VALUE_LENGTH),
            MAX_DETAIL_COUNT,
        ))
    });

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundCustomReportDetailsPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            &DETAILS_STREAM_CODEC,
            |packet: &ClientboundCustomReportDetailsPacket| packet.details().clone(),
            ClientboundCustomReportDetailsPacket::new,
        ))
    });

/// Details a server adds to the client's crash reports, by name: up to 32, each name up to 128
/// characters and each value up to 4096. The codec reads them into a `HashMap` and writes them in
/// its order, which its table decides. A vanilla server never sends it.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundCustomReportDetailsPacket {
    details: HashMap<String, String>,
}

impl ClientboundCustomReportDetailsPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        ByteBuf,
        ClientboundCustomReportDetailsPacket,
    > = &STREAM_CODEC;

    pub fn new(details: HashMap<String, String>) -> ClientboundCustomReportDetailsPacket {
        ClientboundCustomReportDetailsPacket { details }
    }

    pub fn details(&self) -> &HashMap<String, String> {
        &self.details
    }

    /// A record's of one component: its hash, a map's, the sum of each name's hash xor its
    /// value's.
    pub fn hash_code(&self) -> i32 {
        self.details.hash_code()
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundCustomReportDetailsPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_CUSTOM_REPORT_DETAILS
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_custom_report_details(self);
    }
}

/// A record's, with a map's `toString`: `ClientboundCustomReportDetailsPacket[details={Server=Example}]`.
impl fmt::Display for ClientboundCustomReportDetailsPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundCustomReportDetailsPacket[details={}]",
            self.details
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundCustomReportDetailsPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::java::util::hash_map::HashMap;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ClientboundCustomReportDetailsPacket {
        details: HashMap<String, String>
    });

    type Listener = dyn ClientCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket",
        );
        golden.check_codec("STREAM_CODEC", || {
            ClientboundCustomReportDetailsPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Ljava/util/Map;)V",
            ClientboundCustomReportDetailsPacket::new,
        );
        golden.check_method(
            "details()Ljava/util/Map;",
            |this: &mut ClientboundCustomReportDetailsPacket| this.details().clone(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundCustomReportDetailsPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut ClientboundCustomReportDetailsPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundCustomReportDetailsPacket,
             other: ClientboundCustomReportDetailsPacket| *this == other,
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ClientboundCustomReportDetailsPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundCustomReportDetailsPacket| this.to_string(),
        );
        golden.finish();
    }
}
