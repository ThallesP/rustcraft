//! net/minecraft/network/protocol/game/ClientboundLightUpdatePacketData.java
//!
//! Not ported yet: the constructor making the data from a light engine, the server's. A record
//! whose components hold arrays, which its `equals`, `hashCode` and `toString` take by identity,
//! so they aren't ported either; the client only reads the data.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::util::bit_set::BitSet;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};

// A light layer's bytes: 2048 of them, or fewer, which `DataLayer`'s constructor refuses.
static DATA_LAYER_STREAM_CODEC: StaticStreamCodec<ByteBuf, Vec<i8>> =
    LazyLock::new(|| Box::new(ByteBufCodecs::byte_array(2048)));

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundLightUpdatePacketData> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite6(
            ByteBufCodecs::BIT_SET,
            |data: &ClientboundLightUpdatePacketData| data.sky_y_mask.clone(),
            ByteBufCodecs::BIT_SET,
            |data: &ClientboundLightUpdatePacketData| data.block_y_mask.clone(),
            ByteBufCodecs::BIT_SET,
            |data: &ClientboundLightUpdatePacketData| data.empty_sky_y_mask.clone(),
            ByteBufCodecs::BIT_SET,
            |data: &ClientboundLightUpdatePacketData| data.empty_block_y_mask.clone(),
            (&DATA_LAYER_STREAM_CODEC).apply(ByteBufCodecs::list()),
            |data: &ClientboundLightUpdatePacketData| data.sky_updates.clone(),
            (&DATA_LAYER_STREAM_CODEC).apply(ByteBufCodecs::list()),
            |data: &ClientboundLightUpdatePacketData| data.block_updates.clone(),
            ClientboundLightUpdatePacketData::new,
        ))
    });

/// A chunk's light, a bit for each of its light sections (one more than its sections below and
/// above): which have sky and block light that's sent, as a layer each, and which have none.
#[derive(Clone, Debug)]
pub struct ClientboundLightUpdatePacketData {
    sky_y_mask: BitSet,
    block_y_mask: BitSet,
    empty_sky_y_mask: BitSet,
    empty_block_y_mask: BitSet,
    sky_updates: Vec<Vec<i8>>,
    block_updates: Vec<Vec<i8>>,
}

impl ClientboundLightUpdatePacketData {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundLightUpdatePacketData> =
        &STREAM_CODEC;

    pub fn new(
        sky_y_mask: BitSet,
        block_y_mask: BitSet,
        empty_sky_y_mask: BitSet,
        empty_block_y_mask: BitSet,
        sky_updates: Vec<Vec<i8>>,
        block_updates: Vec<Vec<i8>>,
    ) -> ClientboundLightUpdatePacketData {
        ClientboundLightUpdatePacketData {
            sky_y_mask,
            block_y_mask,
            empty_sky_y_mask,
            empty_block_y_mask,
            sky_updates,
            block_updates,
        }
    }

    pub fn sky_y_mask(&self) -> &BitSet {
        &self.sky_y_mask
    }

    pub fn block_y_mask(&self) -> &BitSet {
        &self.block_y_mask
    }

    pub fn empty_sky_y_mask(&self) -> &BitSet {
        &self.empty_sky_y_mask
    }

    pub fn empty_block_y_mask(&self) -> &BitSet {
        &self.empty_block_y_mask
    }

    pub fn sky_updates(&self) -> &[Vec<i8>] {
        &self.sky_updates
    }

    pub fn block_updates(&self) -> &[Vec<i8>] {
        &self.block_updates
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::{Value, json};

    use super::ClientboundLightUpdatePacketData;
    use crate::golden::{Encode, hex};

    /// `[skyYMask, blockYMask, emptySkyYMask, emptyBlockYMask, skyUpdates, blockUpdates]`, the
    /// masks as their words and the layers as hex.
    pub(crate) fn encode_light(light: &ClientboundLightUpdatePacketData) -> Value {
        let layers = |layers: &[Vec<i8>]| layers.iter().map(|layer| hex(layer)).collect::<Vec<_>>();
        json!([
            light.sky_y_mask().encode(),
            light.block_y_mask().encode(),
            light.empty_sky_y_mask().encode(),
            light.empty_block_y_mask().encode(),
            layers(light.sky_updates()),
            layers(light.block_updates())
        ])
    }
}
