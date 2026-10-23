//! net/minecraft/client/renderer/feature/phase/FeatureRenderPhase.java
//!
//! A part of a frame's submits that draws together, and how they're ordered when the frame is
//! prepared (`sortInto`), which hands them on, out of the phase.

use crate::net::minecraft::client::renderer::feature::feature_renderer_type::FeatureRendererType;
use crate::net::minecraft::client::renderer::feature::submit::submit_node::SubmitNode;

pub trait FeatureRenderPhase {
    fn submit(&mut self, submit: SubmitNode);

    fn sort_into(&mut self, output: &mut dyn Output);

    fn is_empty(&self) -> bool;
}

/// `FeatureRenderPhase.Output`: where sorted submits go, each with whether the ones it groups
/// with must keep their order.
pub trait Output {
    fn accept(&mut self, submit: SubmitNode, strictly_ordered: bool);

    /// Panics where Java throws `IllegalArgumentException` for a submit of another type.
    fn accept_feature_group(
        &mut self,
        feature_type: FeatureRendererType,
        submits: Vec<SubmitNode>,
        strictly_ordered: bool,
    ) {
        for submit in submits {
            assert!(
                submit.feature_type() == feature_type,
                "a submit was not of feature type {}",
                feature_type.name()
            );
            self.accept(submit, strictly_ordered);
        }
    }
}
