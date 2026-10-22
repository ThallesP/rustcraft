//! net/minecraft/network/protocol/login/ClientboundLoginFinishedPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::authlib::game_profile::GameProfile;
use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::core::uuid_util::UUIDUtil;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundLoginFinishedPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            ByteBufCodecs::GAME_PROFILE,
            |packet: &ClientboundLoginFinishedPacket| packet.game_profile().clone(),
            UUIDUtil::STREAM_CODEC,
            ClientboundLoginFinishedPacket::session_id,
            ClientboundLoginFinishedPacket::new,
        ))
    });

/// The server's last login packet: the player's profile as the server sees it, and a session id
/// for telemetry. It ends the login, and the client moves on to configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundLoginFinishedPacket {
    game_profile: GameProfile,
    session_id: UUID,
}

impl ClientboundLoginFinishedPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundLoginFinishedPacket> =
        &STREAM_CODEC;

    pub fn new(game_profile: GameProfile, session_id: UUID) -> ClientboundLoginFinishedPacket {
        ClientboundLoginFinishedPacket {
            game_profile,
            session_id,
        }
    }

    pub fn game_profile(&self) -> &GameProfile {
        &self.game_profile
    }

    pub fn session_id(&self) -> UUID {
        self.session_id
    }

    /// A record's: each component's hash, added to 31 times the ones before.
    pub fn hash_code(&self) -> i32 {
        self.game_profile
            .hash_code()
            .wrapping_mul(31)
            .wrapping_add(self.session_id.hash_code())
    }
}

impl<L: ClientLoginPacketListener + ?Sized> Packet<L> for ClientboundLoginFinishedPacket {
    fn r#type(&self) -> &'static PacketType {
        LoginPacketTypes::CLIENTBOUND_LOGIN_FINISHED
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_login_finished(self);
    }

    fn is_terminal(&self) -> bool {
        true
    }
}

/// A record's: `ClientboundLoginFinishedPacket[gameProfile=GameProfile[…], sessionId=…]`.
impl fmt::Display for ClientboundLoginFinishedPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundLoginFinishedPacket[gameProfile={}, sessionId={}]",
            self.game_profile, self.session_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundLoginFinishedPacket;
    use crate::com::mojang::authlib::game_profile::GameProfile;
    use crate::golden::{Golden, encode_fields};
    use crate::java::util::uuid::UUID;
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A record: its components.
    encode_fields!(ClientboundLoginFinishedPacket {
        game_profile: GameProfile,
        session_id: UUID,
    });

    type Listener = dyn ClientLoginPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundLoginFinishedPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lcom/mojang/authlib/GameProfile;Ljava/util/UUID;)V",
            ClientboundLoginFinishedPacket::new,
        );
        golden.check_method(
            "gameProfile()Lcom/mojang/authlib/GameProfile;",
            |this: &mut ClientboundLoginFinishedPacket| this.game_profile().clone(),
        );
        golden.check_method(
            "sessionId()Ljava/util/UUID;",
            |this: &mut ClientboundLoginFinishedPacket| this.session_id(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundLoginFinishedPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/login/ClientLoginPacketListener;)V",
            |this: &mut ClientboundLoginFinishedPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.check_method(
            "isTerminal()Z",
            |this: &mut ClientboundLoginFinishedPacket| Packet::<Listener>::is_terminal(this),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundLoginFinishedPacket, other: ClientboundLoginFinishedPacket| {
                *this == other
            },
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ClientboundLoginFinishedPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundLoginFinishedPacket| this.to_string(),
        );
        golden.finish();
    }
}
