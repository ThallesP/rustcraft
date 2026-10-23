//! net/minecraft/network/protocol/game/ClientboundBundlePacket.java

use std::ops::Deref;

use crate::net::minecraft::network::protocol::bundle_packet::BundlePacket;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

/// The game protocol's bundle: packets the server sent between two delimiters, which the client
/// handles in one go on the game thread, like an entity's spawn with its data.
#[derive(Clone, Debug)]
pub struct ClientboundBundlePacket {
    bundle_packet: BundlePacket<dyn ClientGamePacketListener>,
}

impl ClientboundBundlePacket {
    pub fn new(
        packets: Vec<Box<dyn Packet<dyn ClientGamePacketListener>>>,
    ) -> ClientboundBundlePacket {
        ClientboundBundlePacket {
            bundle_packet: BundlePacket::new(packets),
        }
    }
}

impl Deref for ClientboundBundlePacket {
    type Target = BundlePacket<dyn ClientGamePacketListener>;

    fn deref(&self) -> &BundlePacket<dyn ClientGamePacketListener> {
        &self.bundle_packet
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundBundlePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_BUNDLE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_bundle_packet(self);
    }
}
