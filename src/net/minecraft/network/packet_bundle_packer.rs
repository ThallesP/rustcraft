//! net/minecraft/network/PacketBundlePacker.java
//!
//! A handler after the packet decoder (PORTING.md, "Netty handlers"): its `decode` takes the
//! decoder's packet and gives what goes on to the connection, the packet itself, nothing while a
//! bundle is being read, or the bundle once its last delimiter came. Java's packer removes itself
//! from the pipeline after a terminal packet; here it goes with the decoder it follows, which the
//! terminal packet takes out too (`Connection`).

use crate::net::minecraft::network::protocol::bundler_info::{Bundler, BundlerInfo};
use crate::net::minecraft::network::protocol::packet::Packet;

/// Gathers the packets between a protocol's bundle delimiters into one bundle packet.
pub struct PacketBundlePacker<T: ?Sized> {
    bundler_info: std::sync::Arc<dyn BundlerInfo<T>>,
    current_bundler: Option<Box<dyn Bundler<T>>>,
}

impl<T: ?Sized + 'static> PacketBundlePacker<T> {
    pub fn new(bundler_info: std::sync::Arc<dyn BundlerInfo<T>>) -> PacketBundlePacker<T> {
        PacketBundlePacker {
            bundler_info,
            current_bundler: None,
        }
    }

    pub fn decode(&mut self, msg: Box<dyn Packet<T>>, out: &mut Vec<Box<dyn Packet<T>>>) {
        if let Some(current_bundler) = self.current_bundler.as_mut() {
            PacketBundlePacker::verify_non_terminal_packet(&*msg);
            if let Some(bundle_packet) = current_bundler.add_packet(msg) {
                self.current_bundler = None;
                out.push(bundle_packet);
            }
        } else if let Some(bundler) = self.bundler_info.start_packet_bundling(&*msg) {
            PacketBundlePacker::verify_non_terminal_packet(&*msg);
            self.current_bundler = Some(bundler);
        } else {
            out.push(msg);
        }
    }

    fn verify_non_terminal_packet(msg: &dyn Packet<T>) {
        if msg.is_terminal() {
            panic!("DecoderException: Terminal message received in bundle");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::sync::Arc;

    use super::PacketBundlePacker;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::core::registry_access::RegistryAccess;
    use crate::net::minecraft::network::packet_processor::panic_message;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::game::clientbound_bundle_packet::ClientboundBundlePacket;
    use crate::net::minecraft::network::protocol::game::clientbound_chunk_batch_finished_packet::ClientboundChunkBatchFinishedPacket;
    use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
    use crate::net::minecraft::network::protocol::game::game_protocols::GameProtocols;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::network::protocol_info::ProtocolInfo;
    use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;

    type Listener = dyn ClientGamePacketListener;

    /// The game protocol as a client binds it, whose bundles the packer gathers.
    fn game_protocol() -> Box<dyn ProtocolInfo<Listener>> {
        GameProtocols::CLIENTBOUND_TEMPLATE.bind(RegistryFriendlyByteBuf::decorator(Arc::new(
            RegistryAccess::new(Vec::new()),
        )))
    }

    /// The protocol's packet of each frame, read like the decoder reads it: its id, then its
    /// codec's bytes.
    fn decode(
        protocol: &dyn ProtocolInfo<Listener>,
        frames: &[&[u8]],
    ) -> Vec<Box<dyn Packet<Listener>>> {
        frames
            .iter()
            .map(|frame| {
                let mut buffer = unpooled::buffer();
                buffer.write_bytes_array(&frame.iter().map(|&byte| byte as i8).collect::<Vec<_>>());
                let packet = protocol.codec().decode(&mut buffer);
                assert_eq!(buffer.readable_bytes(), 0);
                packet
            })
            .collect()
    }

    fn pack(
        protocol: &dyn ProtocolInfo<Listener>,
        packets: Vec<Box<dyn Packet<Listener>>>,
    ) -> Vec<Box<dyn Packet<Listener>>> {
        let mut packer = PacketBundlePacker::new(
            protocol
                .bundler_info()
                .expect("the game protocol has bundles"),
        );
        let mut out = Vec::new();
        for packet in packets {
            packer.decode(packet, &mut out);
        }
        out
    }

    fn id(r#type: &crate::net::minecraft::network::protocol::packet_type::PacketType) -> u8 {
        let mut id = None;
        GameProtocols::CLIENTBOUND_TEMPLATE
            .details()
            .list_packets(&mut |listed, network_id| {
                if listed == r#type {
                    id = Some(network_id as u8);
                }
            });
        id.expect("the game protocol has the type")
    }

    #[test]
    fn gathers_the_packets_between_two_delimiters() {
        let protocol = game_protocol();
        let delimiter = id(GamePacketTypes::CLIENTBOUND_BUNDLE_DELIMITER);
        let finished = id(GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_FINISHED);
        let start = id(GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_START);
        assert_eq!(delimiter, 0);

        let packets = decode(
            &*protocol,
            &[
                &[start],
                &[delimiter],
                &[finished, 7],
                &[start],
                &[delimiter],
                &[finished, 9],
            ],
        );
        let out = pack(&*protocol, packets);
        assert_eq!(out.len(), 3);
        assert_eq!(
            out[0].r#type(),
            &**GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_START
        );
        assert_eq!(out[1].r#type(), &**GamePacketTypes::CLIENTBOUND_BUNDLE);
        assert_eq!(
            out[2].r#type(),
            &**GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_FINISHED
        );

        let bundle = (&*out[1] as &dyn Any)
            .downcast_ref::<ClientboundBundlePacket>()
            .expect("a bundle");
        let types: Vec<_> = bundle
            .sub_packets()
            .iter()
            .map(|packet| packet.r#type())
            .collect();
        assert_eq!(
            types,
            [
                GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_FINISHED,
                GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_START,
            ]
            .map(|r#type| &**r#type)
        );
        let first = (&*bundle.sub_packets()[0] as &dyn Any)
            .downcast_ref::<ClientboundChunkBatchFinishedPacket>()
            .expect("the batch's end");
        assert_eq!(first.batch_size(), 7);
    }

    #[test]
    fn two_delimiters_make_an_empty_bundle() {
        let protocol = game_protocol();
        let delimiter = id(GamePacketTypes::CLIENTBOUND_BUNDLE_DELIMITER);
        let out = pack(
            &*protocol,
            decode(&*protocol, &[&[delimiter], &[delimiter]]),
        );
        assert_eq!(out.len(), 1);
        let bundle = (&*out[0] as &dyn Any)
            .downcast_ref::<ClientboundBundlePacket>()
            .unwrap();
        assert!(bundle.sub_packets().is_empty());
    }

    #[test]
    fn a_bundle_holds_at_most_4096_packets() {
        let protocol = game_protocol();
        let delimiter = id(GamePacketTypes::CLIENTBOUND_BUNDLE_DELIMITER);
        let start = id(GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_START);
        let (delimiter_frame, start_frame) = ([delimiter], [start]);
        let mut frames: Vec<&[u8]> = vec![&delimiter_frame];
        frames.extend(std::iter::repeat_n(&start_frame[..], 4096));
        frames.push(&delimiter_frame);
        let out = pack(&*protocol, decode(&*protocol, &frames));
        let bundle = (&*out[0] as &dyn Any)
            .downcast_ref::<ClientboundBundlePacket>()
            .unwrap();
        assert_eq!(bundle.sub_packets().len(), 4096);

        frames.insert(1, &start_frame);
        let packets = decode(&*protocol, &frames);
        let panic =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pack(&*protocol, packets)))
                .expect_err("a 4097th packet");
        assert_eq!(
            panic_message(&*panic),
            "IllegalStateException: Too many packets in a bundle"
        );
    }

    #[test]
    fn a_terminal_packet_in_a_bundle_fails() {
        let protocol = game_protocol();
        let delimiter = id(GamePacketTypes::CLIENTBOUND_BUNDLE_DELIMITER);
        let start_configuration = id(GamePacketTypes::CLIENTBOUND_START_CONFIGURATION);
        let packets = decode(&*protocol, &[&[delimiter], &[start_configuration]]);
        let panic =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pack(&*protocol, packets)))
                .expect_err("a terminal packet");
        assert_eq!(
            panic_message(&*panic),
            "DecoderException: Terminal message received in bundle"
        );

        // Outside a bundle, it goes on to the listener.
        let out = pack(&*protocol, decode(&*protocol, &[&[start_configuration]]));
        assert!(out[0].is_terminal());
    }
}
