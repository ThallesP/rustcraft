//! net/minecraft/network/protocol/game/ClientboundPlayerAbilitiesPacket.java
//!
//! Not ported yet: its public constructor, which reads a player's `Abilities`, the player track's;
//! the client only reads the packet.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundPlayerAbilitiesPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundPlayerAbilitiesPacket::write,
            ClientboundPlayerAbilitiesPacket::from_friendly_byte_buf,
        ))
    });

/// What the player may do (fly, build without using up blocks, not be hurt), and its flying and
/// walking speeds.
#[derive(Clone, Debug)]
pub struct ClientboundPlayerAbilitiesPacket {
    invulnerable: bool,
    is_flying: bool,
    can_fly: bool,
    instabuild: bool,
    flying_speed: f32,
    walking_speed: f32,
}

impl ClientboundPlayerAbilitiesPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundPlayerAbilitiesPacket,
    > = &STREAM_CODEC;
    const FLAG_INVULNERABLE: i8 = 1;
    const FLAG_FLYING: i8 = 2;
    const FLAG_CAN_FLY: i8 = 4;
    const FLAG_INSTABUILD: i8 = 8;

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundPlayerAbilitiesPacket {
        let bitfield = input.read_byte();
        ClientboundPlayerAbilitiesPacket {
            invulnerable: (bitfield & ClientboundPlayerAbilitiesPacket::FLAG_INVULNERABLE) != 0,
            is_flying: (bitfield & ClientboundPlayerAbilitiesPacket::FLAG_FLYING) != 0,
            can_fly: (bitfield & ClientboundPlayerAbilitiesPacket::FLAG_CAN_FLY) != 0,
            instabuild: (bitfield & ClientboundPlayerAbilitiesPacket::FLAG_INSTABUILD) != 0,
            flying_speed: input.read_float(),
            walking_speed: input.read_float(),
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        let mut bitfield: i8 = 0;
        if self.invulnerable {
            bitfield |= ClientboundPlayerAbilitiesPacket::FLAG_INVULNERABLE;
        }
        if self.is_flying {
            bitfield |= ClientboundPlayerAbilitiesPacket::FLAG_FLYING;
        }
        if self.can_fly {
            bitfield |= ClientboundPlayerAbilitiesPacket::FLAG_CAN_FLY;
        }
        if self.instabuild {
            bitfield |= ClientboundPlayerAbilitiesPacket::FLAG_INSTABUILD;
        }
        output.write_byte(i32::from(bitfield));
        output.write_float(self.flying_speed);
        output.write_float(self.walking_speed);
    }

    pub fn is_invulnerable(&self) -> bool {
        self.invulnerable
    }

    pub fn is_flying(&self) -> bool {
        self.is_flying
    }

    pub fn can_fly(&self) -> bool {
        self.can_fly
    }

    pub fn can_instabuild(&self) -> bool {
        self.instabuild
    }

    pub fn get_flying_speed(&self) -> f32 {
        self.flying_speed
    }

    pub fn get_walking_speed(&self) -> f32 {
        self.walking_speed
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundPlayerAbilitiesPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_PLAYER_ABILITIES
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_player_abilities(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundPlayerAbilitiesPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // Its fields, in the order it declares them.
    encode_fields!(ClientboundPlayerAbilitiesPacket {
        invulnerable: bool,
        is_flying: bool,
        can_fly: bool,
        instabuild: bool,
        flying_speed: f32,
        walking_speed: f32,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundPlayerAbilitiesPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundPlayerAbilitiesPacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Lnet/minecraft/world/entity/player/Abilities;)V",
            "reads a player's Abilities, which aren't ported",
        );
        golden.check_method("isInvulnerable()Z", |this: &mut This| {
            this.is_invulnerable()
        });
        golden.check_method("isFlying()Z", |this: &mut This| this.is_flying());
        golden.check_method("canFly()Z", |this: &mut This| this.can_fly());
        golden.check_method("canInstabuild()Z", |this: &mut This| this.can_instabuild());
        golden.check_method("getFlyingSpeed()F", |this: &mut This| {
            this.get_flying_speed()
        });
        golden.check_method("getWalkingSpeed()F", |this: &mut This| {
            this.get_walking_speed()
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut This| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
