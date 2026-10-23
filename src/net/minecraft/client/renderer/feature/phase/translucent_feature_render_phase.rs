//! net/minecraft/client/renderer/feature/phase/TranslucentFeatureRenderPhase.java
//!
//! Translucent submits, drawn farthest first, each in its own place: by their distance to the
//! camera when submitted, then in the order they came.

use crate::java::lang::float::Float;
use crate::net::minecraft::client::renderer::feature::phase::feature_render_phase::{
    FeatureRenderPhase, Output,
};
use crate::net::minecraft::client::renderer::feature::submit::submit_node::SubmitNode;

#[derive(Default)]
pub struct TranslucentFeatureRenderPhase {
    submits: Vec<SubmitNode>,
    distances: Vec<f32>,
}

impl TranslucentFeatureRenderPhase {
    /// The submits' indices, farthest first, then in order. Java's sort is unstable, but its
    /// comparison only ties an index with itself, so any sort gives this order.
    fn sort_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.submits.len()).collect();
        indices.sort_by(|&i1, &i2| {
            Float::compare(self.distances[i2], self.distances[i1]).then(i1.cmp(&i2))
        });
        indices
    }
}

impl FeatureRenderPhase for TranslucentFeatureRenderPhase {
    fn submit(&mut self, submit: SubmitNode) {
        self.distances.push(submit.distance_to_camera_sq());
        self.submits.push(submit);
    }

    fn sort_into(&mut self, output: &mut dyn Output) {
        if self.submits.is_empty() {
            return;
        }
        let indices = self.sort_indices();
        let mut submits: Vec<Option<SubmitNode>> = std::mem::take(&mut self.submits)
            .into_iter()
            .map(Some)
            .collect();
        for index in indices {
            output.accept(submits[index].take().expect("each index once"), true);
        }
        self.distances.clear();
    }

    fn is_empty(&self) -> bool {
        self.submits.is_empty()
    }
}
