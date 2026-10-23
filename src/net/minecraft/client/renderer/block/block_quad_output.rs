//! net/minecraft/client/renderer/block/BlockQuadOutput.java

use crate::com::mojang::blaze3d::vertex::quad_instance::QuadInstance;
use crate::net::minecraft::client::resources::model::geometry::baked_quad::BakedQuad;

/// Where a block's quads go, at its place in the section.
pub trait BlockQuadOutput {
    fn put(&mut self, x: f32, y: f32, z: f32, quad: &BakedQuad, instance: &QuadInstance);
}
