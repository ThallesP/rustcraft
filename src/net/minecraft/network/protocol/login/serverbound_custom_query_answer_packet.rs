//! net/minecraft/network/protocol/login/ServerboundCustomQueryAnswerPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::login::custom::custom_query_answer_payload::CustomQueryAnswerPayload;
use crate::net::minecraft::network::protocol::login::custom::discarded_query_answer_payload::DiscardedQueryAnswerPayload;
use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundCustomQueryAnswerPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundCustomQueryAnswerPacket::write,
            ServerboundCustomQueryAnswerPacket::read,
        ))
    });

/// The client's answer to a server's custom query: the query's transaction id, and a payload, or
/// none when it doesn't know the query (vanilla never does).
#[derive(Debug, PartialEq)]
pub struct ServerboundCustomQueryAnswerPacket {
    transaction_id: i32,
    payload: Option<Box<dyn CustomQueryAnswerPayload>>,
}

impl ServerboundCustomQueryAnswerPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundCustomQueryAnswerPacket,
    > = &STREAM_CODEC;
    const MAX_PAYLOAD_SIZE: i32 = 1048576;

    pub fn new(
        transaction_id: i32,
        payload: Option<Box<dyn CustomQueryAnswerPayload>>,
    ) -> ServerboundCustomQueryAnswerPacket {
        ServerboundCustomQueryAnswerPacket {
            transaction_id,
            payload,
        }
    }

    /// Never reads a payload of none: `write` puts a boolean before the payload, but this skips
    /// it with the payload's bytes.
    fn read(input: &mut FriendlyByteBuf) -> ServerboundCustomQueryAnswerPacket {
        let transaction_id = input.read_var_int();
        ServerboundCustomQueryAnswerPacket::new(
            transaction_id,
            Some(ServerboundCustomQueryAnswerPacket::read_payload(
                transaction_id,
                input,
            )),
        )
    }

    fn read_payload(
        _transaction_id: i32,
        input: &mut FriendlyByteBuf,
    ) -> Box<dyn CustomQueryAnswerPayload> {
        ServerboundCustomQueryAnswerPacket::read_unknown_payload(input)
    }

    /// Skips every byte left, and panics past 1 MiB, where Java throws
    /// `IllegalArgumentException`.
    fn read_unknown_payload(input: &mut FriendlyByteBuf) -> Box<dyn CustomQueryAnswerPayload> {
        let length = input.readable_bytes();
        if (0..=ServerboundCustomQueryAnswerPacket::MAX_PAYLOAD_SIZE).contains(&length) {
            input.skip_bytes(length);
            Box::new(DiscardedQueryAnswerPayload::INSTANCE)
        } else {
            panic!("Payload may not be larger than 1048576 bytes");
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.transaction_id);
        output.write_nullable(
            self.payload.as_deref(),
            |buf: &mut FriendlyByteBuf, data: &dyn CustomQueryAnswerPayload| data.write(buf),
        );
    }

    pub fn transaction_id(&self) -> i32 {
        self.transaction_id
    }

    pub fn payload(&self) -> Option<&dyn CustomQueryAnswerPayload> {
        self.payload.as_deref()
    }

    /// A record's: each component's hash, added to 31 times the ones before, and 0 for none.
    pub fn hash_code(&self) -> i32 {
        self.transaction_id.wrapping_mul(31).wrapping_add(
            self.payload
                .as_ref()
                .map_or(0, |payload| payload.hash_code()),
        )
    }
}

impl<L: ServerLoginPacketListener + ?Sized> Packet<L> for ServerboundCustomQueryAnswerPacket {
    fn r#type(&self) -> &'static PacketType {
        LoginPacketTypes::SERVERBOUND_CUSTOM_QUERY_ANSWER
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_custom_query_packet(self);
    }
}

/// A record's: `ServerboundCustomQueryAnswerPacket[transactionId=0, payload=null]`.
impl fmt::Display for ServerboundCustomQueryAnswerPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ServerboundCustomQueryAnswerPacket[transactionId={}, payload=",
            self.transaction_id
        )?;
        match &self.payload {
            Some(payload) => write!(f, "{payload}]"),
            None => f.write_str("null]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundCustomQueryAnswerPacket;
    use crate::golden::{Encode, Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::login::custom::custom_query_answer_payload::CustomQueryAnswerPayload;
    use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ServerboundCustomQueryAnswerPacket {
        transaction_id: i32,
        payload: Option<Box<dyn CustomQueryAnswerPayload>>,
    });

    type Listener = dyn ServerLoginPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket");
        golden.check_codec("STREAM_CODEC", || {
            ServerboundCustomQueryAnswerPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(ILnet/minecraft/network/protocol/login/custom/CustomQueryAnswerPayload;)V",
            ServerboundCustomQueryAnswerPacket::new,
        );
        golden.check_method(
            "transactionId()I",
            |this: &mut ServerboundCustomQueryAnswerPacket| this.transaction_id(),
        );
        golden.check_method(
            "payload()Lnet/minecraft/network/protocol/login/custom/CustomQueryAnswerPayload;",
            |this: &mut ServerboundCustomQueryAnswerPacket| this.payload().map(Encode::encode),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundCustomQueryAnswerPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/login/ServerLoginPacketListener;)V",
            |this: &mut ServerboundCustomQueryAnswerPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundCustomQueryAnswerPacket,
             other: ServerboundCustomQueryAnswerPacket| *this == other,
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ServerboundCustomQueryAnswerPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundCustomQueryAnswerPacket| this.to_string(),
        );
        golden.finish();
    }
}
