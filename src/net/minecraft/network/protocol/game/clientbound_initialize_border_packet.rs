//! net/minecraft/network/protocol/game/ClientboundInitializeBorderPacket.java
//!
//! Not ported yet: its public constructor, which reads a `WorldBorder`, the world track's; the
//! client only reads the packet.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundInitializeBorderPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundInitializeBorderPacket::write,
            ClientboundInitializeBorderPacket::from_friendly_byte_buf,
        ))
    });

/// The world border, all of it: its center, its size and the size it's moving to, how long that takes, and its warnings.
#[derive(Clone, Debug)]
pub struct ClientboundInitializeBorderPacket {
    new_center_x: f64,
    new_center_z: f64,
    old_size: f64,
    new_size: f64,
    lerp_time: i64,
    new_absolute_max_size: i32,
    warning_blocks: i32,
    warning_time: i32,
}

impl ClientboundInitializeBorderPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundInitializeBorderPacket,
    > = &STREAM_CODEC;

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundInitializeBorderPacket {
        let new_center_x = input.read_double();
        let new_center_z = input.read_double();
        let old_size = input.read_double();
        let new_size = input.read_double();
        let lerp_time = input.read_var_long();
        let new_absolute_max_size = input.read_var_int();
        let warning_blocks = input.read_var_int();
        let warning_time = input.read_var_int();
        ClientboundInitializeBorderPacket {
            new_center_x,
            new_center_z,
            old_size,
            new_size,
            lerp_time,
            new_absolute_max_size,
            warning_blocks,
            warning_time,
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_double(self.new_center_x);
        output.write_double(self.new_center_z);
        output.write_double(self.old_size);
        output.write_double(self.new_size);
        output.write_var_long(self.lerp_time);
        output.write_var_int(self.new_absolute_max_size);
        output.write_var_int(self.warning_blocks);
        output.write_var_int(self.warning_time);
    }

    pub fn get_new_center_x(&self) -> f64 {
        self.new_center_x
    }

    pub fn get_new_center_z(&self) -> f64 {
        self.new_center_z
    }

    pub fn get_new_size(&self) -> f64 {
        self.new_size
    }

    pub fn get_old_size(&self) -> f64 {
        self.old_size
    }

    pub fn get_lerp_time(&self) -> i64 {
        self.lerp_time
    }

    pub fn get_new_absolute_max_size(&self) -> i32 {
        self.new_absolute_max_size
    }

    pub fn get_warning_time(&self) -> i32 {
        self.warning_time
    }

    pub fn get_warning_blocks(&self) -> i32 {
        self.warning_blocks
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundInitializeBorderPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_INITIALIZE_BORDER
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_initialize_border(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundInitializeBorderPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields it reads.
    encode_fields!(ClientboundInitializeBorderPacket {
        new_center_x: f64,
        new_center_z: f64,
        old_size: f64,
        new_size: f64,
        lerp_time: i64,
        new_absolute_max_size: i32,
        warning_blocks: i32,
        warning_time: i32
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundInitializeBorderPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundInitializeBorderPacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Lnet/minecraft/world/level/border/WorldBorder;)V",
            "reads a WorldBorder, which isn't ported",
        );
        golden.check_method("getNewCenterX()D", |this: &mut This| {
            this.get_new_center_x()
        });
        golden.check_method("getNewCenterZ()D", |this: &mut This| {
            this.get_new_center_z()
        });
        golden.check_method("getNewSize()D", |this: &mut This| this.get_new_size());
        golden.check_method("getOldSize()D", |this: &mut This| this.get_old_size());
        golden.check_method("getLerpTime()J", |this: &mut This| this.get_lerp_time());
        golden.check_method("getNewAbsoluteMaxSize()I", |this: &mut This| {
            this.get_new_absolute_max_size()
        });
        golden.check_method("getWarningTime()I", |this: &mut This| {
            this.get_warning_time()
        });
        golden.check_method("getWarningBlocks()I", |this: &mut This| {
            this.get_warning_blocks()
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
