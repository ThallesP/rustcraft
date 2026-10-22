//! net/minecraft/network/protocol/common/ServerboundResourcePackPacket.java
//!
//! Not ported: `hashCode`, a record's, which hashes its action's identity hash, and so changes
//! from run to run. Nothing hashes a packet.

use std::fmt;
use std::sync::LazyLock;

use crate::java::lang::r#enum::Enum;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundResourcePackPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundResourcePackPacket::write,
            ServerboundResourcePackPacket::from_friendly_byte_buf,
        ))
    });

/// What the client did with a resource pack the server pushed, by the pack's id.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundResourcePackPacket {
    id: UUID,
    action: Action,
}

impl ServerboundResourcePackPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundResourcePackPacket,
    > = &STREAM_CODEC;

    pub fn new(id: UUID, action: Action) -> ServerboundResourcePackPacket {
        ServerboundResourcePackPacket { id, action }
    }

    /// The action by its ordinal, which panics past the last, where Java throws
    /// `ArrayIndexOutOfBoundsException`.
    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ServerboundResourcePackPacket {
        let id = input.read_uuid();
        ServerboundResourcePackPacket::new(id, input.read_enum::<Action>())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_uuid(&self.id);
        output.write_enum(&self.action);
    }

    pub fn id(&self) -> UUID {
        self.id
    }

    pub fn action(&self) -> Action {
        self.action
    }
}

impl<L: ServerCommonPacketListener + ?Sized> Packet<L> for ServerboundResourcePackPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::SERVERBOUND_RESOURCE_PACK
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_resource_pack_response(self);
    }
}

/// A record's: `ServerboundResourcePackPacket[id=…, action=ACCEPTED]`.
impl fmt::Display for ServerboundResourcePackPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ServerboundResourcePackPacket[id={}, action={}]",
            self.id, self.action
        )
    }
}

/// `ServerboundResourcePackPacket.Action`: how far a pack got. Accepting and downloading it come
/// before one of the others.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    SUCCESSFULLY_LOADED,
    DECLINED,
    FAILED_DOWNLOAD,
    ACCEPTED,
    DOWNLOADED,
    INVALID_URL,
    FAILED_RELOAD,
    DISCARDED,
}

impl Action {
    pub const VALUES: [Action; 8] = [
        Action::SUCCESSFULLY_LOADED,
        Action::DECLINED,
        Action::FAILED_DOWNLOAD,
        Action::ACCEPTED,
        Action::DOWNLOADED,
        Action::INVALID_URL,
        Action::FAILED_RELOAD,
        Action::DISCARDED,
    ];

    pub fn values() -> [Action; 8] {
        Action::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Action {
        match name {
            "SUCCESSFULLY_LOADED" => Action::SUCCESSFULLY_LOADED,
            "DECLINED" => Action::DECLINED,
            "FAILED_DOWNLOAD" => Action::FAILED_DOWNLOAD,
            "ACCEPTED" => Action::ACCEPTED,
            "DOWNLOADED" => Action::DOWNLOADED,
            "INVALID_URL" => Action::INVALID_URL,
            "FAILED_RELOAD" => Action::FAILED_RELOAD,
            "DISCARDED" => Action::DISCARDED,
            _ => panic!("IllegalArgumentException: no Action constant {name}"),
        }
    }

    /// Whether the pack is done with, loaded or not.
    pub fn is_terminal(&self) -> bool {
        *self != Action::ACCEPTED && *self != Action::DOWNLOADED
    }
}

impl Enum for Action {
    const ENUM_CONSTANTS: &'static [Action] = &Action::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, ServerboundResourcePackPacket};
    use crate::golden::{Golden, encode_fields, encode_names};
    use crate::java::util::uuid::UUID;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ServerboundResourcePackPacket {
        id: UUID,
        action: Action,
    });

    encode_names!(Action {
        SUCCESSFULLY_LOADED,
        DECLINED,
        FAILED_DOWNLOAD,
        ACCEPTED,
        DOWNLOADED,
        INVALID_URL,
        FAILED_RELOAD,
        DISCARDED
    });

    type Listener = dyn ServerCommonPacketListener;

    #[test]
    fn serverbound_resource_pack_packet_matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ServerboundResourcePackPacket");
        golden.pending(
            "hashCode()I",
            "hashes an enum constant's identity hash, which changes from run to run",
        );
        golden.check_codec("STREAM_CODEC", || {
            ServerboundResourcePackPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Ljava/util/UUID;Lnet/minecraft/network/protocol/common/ServerboundResourcePackPacket$Action;)V",
            ServerboundResourcePackPacket::new,
        );
        golden.check_method(
            "id()Ljava/util/UUID;",
            |this: &mut ServerboundResourcePackPacket| this.id(),
        );
        golden.check_method(
            "action()Lnet/minecraft/network/protocol/common/ServerboundResourcePackPacket$Action;",
            |this: &mut ServerboundResourcePackPacket| this.action(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundResourcePackPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ServerCommonPacketListener;)V",
            |this: &mut ServerboundResourcePackPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundResourcePackPacket, other: ServerboundResourcePackPacket| {
                *this == other
            },
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundResourcePackPacket| this.to_string(),
        );
        golden.finish();
    }

    #[test]
    fn action_matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.common.ServerboundResourcePackPacket$Action",
        );
        for action in Action::VALUES {
            golden.field(&format!("{action:?}"), action);
        }
        golden.check(
            "values()[Lnet/minecraft/network/protocol/common/ServerboundResourcePackPacket$Action;",
            Action::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/network/protocol/common/ServerboundResourcePackPacket$Action;",
            |name: String| Action::value_of(&name),
        );
        golden.check_method("isTerminal()Z", |this: &mut Action| this.is_terminal());
        golden.finish();
    }
}
