//! net/minecraft/network/protocol/login/ClientboundCustomQueryPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
use crate::net::minecraft::network::protocol::login::custom::custom_query_payload::CustomQueryPayload;
use crate::net::minecraft::network::protocol::login::custom::discarded_query_payload::DiscardedQueryPayload;
use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundCustomQueryPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundCustomQueryPacket::write,
            ClientboundCustomQueryPacket::from_friendly_byte_buf,
        ))
    });

/// A server's custom query while a client logs in, which vanilla's never sends (mods and proxies
/// do): a transaction id the answer repeats, and a payload. The client answers with a
/// `ServerboundCustomQueryAnswerPacket`.
#[derive(Debug)]
pub struct ClientboundCustomQueryPacket {
    transaction_id: i32,
    payload: Box<dyn CustomQueryPayload>,
}

impl ClientboundCustomQueryPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundCustomQueryPacket,
    > = &STREAM_CODEC;
    const MAX_PAYLOAD_SIZE: i32 = 1048576;

    pub fn new(
        transaction_id: i32,
        payload: Box<dyn CustomQueryPayload>,
    ) -> ClientboundCustomQueryPacket {
        ClientboundCustomQueryPacket {
            transaction_id,
            payload,
        }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundCustomQueryPacket {
        let transaction_id = input.read_var_int();
        let identifier = input.read_identifier();
        ClientboundCustomQueryPacket::new(
            transaction_id,
            ClientboundCustomQueryPacket::read_payload(identifier, input),
        )
    }

    fn read_payload(
        identifier: Identifier,
        input: &mut FriendlyByteBuf,
    ) -> Box<dyn CustomQueryPayload> {
        Box::new(ClientboundCustomQueryPacket::read_unknown_payload(
            identifier, input,
        ))
    }

    /// Skips every byte left, and panics past 1 MiB, where Java throws
    /// `IllegalArgumentException`.
    fn read_unknown_payload(
        identifier: Identifier,
        input: &mut FriendlyByteBuf,
    ) -> DiscardedQueryPayload {
        let length = input.readable_bytes();
        if (0..=ClientboundCustomQueryPacket::MAX_PAYLOAD_SIZE).contains(&length) {
            input.skip_bytes(length);
            DiscardedQueryPayload::new(identifier)
        } else {
            panic!("Payload may not be larger than 1048576 bytes");
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.transaction_id);
        output.write_identifier(self.payload.id());
        self.payload.write(output);
    }

    pub fn transaction_id(&self) -> i32 {
        self.transaction_id
    }

    pub fn payload(&self) -> &dyn CustomQueryPayload {
        &*self.payload
    }

    /// A record's: each component's hash, added to 31 times the ones before.
    pub fn hash_code(&self) -> i32 {
        self.transaction_id
            .wrapping_mul(31)
            .wrapping_add(self.payload.hash_code())
    }
}

/// A record's: each component, the payload with its `equals`.
impl PartialEq for ClientboundCustomQueryPacket {
    fn eq(&self, other: &ClientboundCustomQueryPacket) -> bool {
        self.transaction_id == other.transaction_id && self.payload.equals(&*other.payload)
    }
}

impl<L: ClientLoginPacketListener + ?Sized> Packet<L> for ClientboundCustomQueryPacket {
    fn r#type(&self) -> &'static PacketType {
        LoginPacketTypes::CLIENTBOUND_CUSTOM_QUERY
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_custom_query(self);
    }
}

/// A record's: `ClientboundCustomQueryPacket[transactionId=0, payload=DiscardedQueryPayload[…]]`.
impl fmt::Display for ClientboundCustomQueryPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundCustomQueryPacket[transactionId={}, payload={}]",
            self.transaction_id, self.payload
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundCustomQueryPacket;
    use crate::golden::{Encode, Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
    use crate::net::minecraft::network::protocol::login::custom::custom_query_payload::CustomQueryPayload;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ClientboundCustomQueryPacket {
        transaction_id: i32,
        payload: Box<dyn CustomQueryPayload>,
    });

    type Listener = dyn ClientLoginPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.login.ClientboundCustomQueryPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundCustomQueryPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(ILnet/minecraft/network/protocol/login/custom/CustomQueryPayload;)V",
            ClientboundCustomQueryPacket::new,
        );
        golden.check_method(
            "transactionId()I",
            |this: &mut ClientboundCustomQueryPacket| this.transaction_id(),
        );
        golden.check_method(
            "payload()Lnet/minecraft/network/protocol/login/custom/CustomQueryPayload;",
            |this: &mut ClientboundCustomQueryPacket| this.payload().encode(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundCustomQueryPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/login/ClientLoginPacketListener;)V",
            |this: &mut ClientboundCustomQueryPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundCustomQueryPacket, other: ClientboundCustomQueryPacket| {
                *this == other
            },
        );
        golden.check_method("hashCode()I", |this: &mut ClientboundCustomQueryPacket| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundCustomQueryPacket| this.to_string(),
        );
        golden.finish();
    }
}
