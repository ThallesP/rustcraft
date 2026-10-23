//! net/minecraft/client/renderer/SectionBufferBuilderPack.java
//!
//! A buffer for each of a section's layers. The port's buffers free each result as it's built
//! (`ByteBufferBuilder`), so `clearAll` and `discardAll` have no results left to drop.

use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;

pub struct SectionBufferBuilderPack {
    buffers: [ByteBufferBuilder; 3],
}

impl Default for SectionBufferBuilderPack {
    fn default() -> SectionBufferBuilderPack {
        SectionBufferBuilderPack::new()
    }
}

impl SectionBufferBuilderPack {
    pub fn total_buffers_size() -> i32 {
        ChunkSectionLayer::VALUES
            .iter()
            .map(|layer| layer.buffer_size())
            .sum()
    }

    pub fn new() -> SectionBufferBuilderPack {
        SectionBufferBuilderPack {
            buffers: ChunkSectionLayer::VALUES
                .map(|layer| ByteBufferBuilder::new(layer.buffer_size())),
        }
    }

    pub fn buffer(&mut self, layer: ChunkSectionLayer) -> &mut ByteBufferBuilder {
        &mut self.buffers[layer as usize]
    }

    /// Every layer's buffer at once, by the layer's ordinal, for builders that write into
    /// several.
    pub fn buffers_mut(&mut self) -> [&mut ByteBufferBuilder; 3] {
        self.buffers.each_mut()
    }
}
