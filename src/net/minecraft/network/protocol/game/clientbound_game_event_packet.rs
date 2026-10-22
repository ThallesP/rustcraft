//! net/minecraft/network/protocol/game/ClientboundGameEventPacket.java
//!
//! Java's `Type` is a class whose constructor registers each instance by its id, and the packet
//! reads a type by looking its id up there, finding null for an id no constant has. Java compares
//! types by reference, and each id has one constant, so here a type is its id, compared by it,
//! and the lookup is the constants' ids. A packet with an unknown type holds none, and writing it
//! panics, where Java throws `NullPointerException`.

use std::sync::LazyLock;

use crate::java::lang::float::Float;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundGameEventPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundGameEventPacket::write,
            ClientboundGameEventPacket::from_friendly_byte_buf,
        ))
    });

/// Something happening to the player or the world, by its type, with a parameter.
#[derive(Clone, Debug)]
pub struct ClientboundGameEventPacket {
    event: Option<Type>,
    param: f32,
}

impl ClientboundGameEventPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundGameEventPacket,
    > = &STREAM_CODEC;
    pub const NO_RESPAWN_BLOCK_AVAILABLE: Type = Type::new(0);
    pub const START_RAINING: Type = Type::new(1);
    pub const STOP_RAINING: Type = Type::new(2);
    pub const CHANGE_GAME_MODE: Type = Type::new(3);
    pub const WIN_GAME: Type = Type::new(4);
    pub const DEMO_EVENT: Type = Type::new(5);
    pub const PLAY_ARROW_HIT_SOUND: Type = Type::new(6);
    pub const RAIN_LEVEL_CHANGE: Type = Type::new(7);
    pub const THUNDER_LEVEL_CHANGE: Type = Type::new(8);
    pub const PUFFER_FISH_STING: Type = Type::new(9);
    pub const GUARDIAN_ELDER_EFFECT: Type = Type::new(10);
    pub const IMMEDIATE_RESPAWN: Type = Type::new(11);
    pub const LIMITED_CRAFTING: Type = Type::new(12);
    pub const LEVEL_CHUNKS_LOAD_START: Type = Type::new(13);
    pub const DEMO_PARAM_INTRO: i32 = 0;
    pub const DEMO_PARAM_HINT_1: i32 = 101;
    pub const DEMO_PARAM_HINT_2: i32 = 102;
    pub const DEMO_PARAM_HINT_3: i32 = 103;
    pub const DEMO_PARAM_HINT_4: i32 = 104;

    pub fn new(event: Type, param: f32) -> ClientboundGameEventPacket {
        ClientboundGameEventPacket {
            event: Some(event),
            param,
        }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundGameEventPacket {
        let event = Type::by_id(i32::from(input.read_unsigned_byte()));
        let param = input.read_float();
        ClientboundGameEventPacket { event, param }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        let event = self
            .event
            .expect("NullPointerException: the packet has no event type");
        output.write_byte(event.id);
        output.write_float(self.param);
    }

    pub fn get_event(&self) -> Option<Type> {
        self.event
    }

    pub fn get_param(&self) -> f32 {
        self.param
    }
}

/// Java's class compares by reference, which is what it would be if each packet shared the
/// constants: the same types and the same bits of the parameter don't make two packets equal in
/// Java, but the port has no reference to compare, and nothing compares packets.
impl PartialEq for ClientboundGameEventPacket {
    fn eq(&self, other: &ClientboundGameEventPacket) -> bool {
        self.event == other.event
            && Float::float_to_int_bits(self.param) == Float::float_to_int_bits(other.param)
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundGameEventPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_GAME_EVENT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_game_event(self);
    }
}

/// `ClientboundGameEventPacket.Type`: a kind of game event, by its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Type {
    id: i32,
}

impl Type {
    pub const fn new(id: i32) -> Type {
        Type { id }
    }

    /// `TYPES.get(id)`: the constant with the id, if one has it.
    fn by_id(id: i32) -> Option<Type> {
        (0..=13).contains(&id).then_some(Type::new(id))
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{ClientboundGameEventPacket, Type};
    use crate::golden::{Decode, Encode, Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // A type is its id.
    impl Decode for Type {
        fn decode(value: &Value) -> Self {
            Type::new(i32::decode(value))
        }
    }

    impl Encode for Type {
        fn encode(&self) -> Value {
            self.id.encode()
        }
    }

    // The fields its public constructor sets.
    encode_fields!(ClientboundGameEventPacket {
        event: Option<Type>,
        param: f32,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundGameEventPacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.ClientboundGameEventPacket");
        golden.check_codec("STREAM_CODEC", || ClientboundGameEventPacket::STREAM_CODEC);
        for (name, r#type) in [
            (
                "NO_RESPAWN_BLOCK_AVAILABLE",
                This::NO_RESPAWN_BLOCK_AVAILABLE,
            ),
            ("START_RAINING", This::START_RAINING),
            ("STOP_RAINING", This::STOP_RAINING),
            ("CHANGE_GAME_MODE", This::CHANGE_GAME_MODE),
            ("WIN_GAME", This::WIN_GAME),
            ("DEMO_EVENT", This::DEMO_EVENT),
            ("PLAY_ARROW_HIT_SOUND", This::PLAY_ARROW_HIT_SOUND),
            ("RAIN_LEVEL_CHANGE", This::RAIN_LEVEL_CHANGE),
            ("THUNDER_LEVEL_CHANGE", This::THUNDER_LEVEL_CHANGE),
            ("PUFFER_FISH_STING", This::PUFFER_FISH_STING),
            ("GUARDIAN_ELDER_EFFECT", This::GUARDIAN_ELDER_EFFECT),
            ("IMMEDIATE_RESPAWN", This::IMMEDIATE_RESPAWN),
            ("LIMITED_CRAFTING", This::LIMITED_CRAFTING),
            ("LEVEL_CHUNKS_LOAD_START", This::LEVEL_CHUNKS_LOAD_START),
        ] {
            golden.field(name, r#type);
        }
        for (name, value) in [
            ("DEMO_PARAM_INTRO", This::DEMO_PARAM_INTRO),
            ("DEMO_PARAM_HINT_1", This::DEMO_PARAM_HINT_1),
            ("DEMO_PARAM_HINT_2", This::DEMO_PARAM_HINT_2),
            ("DEMO_PARAM_HINT_3", This::DEMO_PARAM_HINT_3),
            ("DEMO_PARAM_HINT_4", This::DEMO_PARAM_HINT_4),
        ] {
            golden.field(name, value);
        }
        golden.check(
            "<init>(Lnet/minecraft/network/protocol/game/ClientboundGameEventPacket$Type;F)V",
            ClientboundGameEventPacket::new,
        );
        golden.check_method(
            "getEvent()Lnet/minecraft/network/protocol/game/ClientboundGameEventPacket$Type;",
            |this: &mut This| this.get_event(),
        );
        golden.check_method("getParam()F", |this: &mut This| this.get_param());
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
