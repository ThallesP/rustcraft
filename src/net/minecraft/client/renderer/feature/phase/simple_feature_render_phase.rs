//! net/minecraft/client/renderer/feature/phase/SimpleFeatureRenderPhase.java
//!
//! Submits that may draw in any order, kept by their feature's type, then grouped by their batch
//! key (a render type, by identity) so a batch's draws can share a buffer. Java keeps the batches
//! in a `HashMap` of the keys' identity hashes, an order that changes from run to run; the port
//! keeps them in the order their keys first came. The debug shuffle isn't ported.

use crate::net::minecraft::client::renderer::feature::feature_renderer_type::FeatureRendererType;
use crate::net::minecraft::client::renderer::feature::phase::feature_render_phase::{
    FeatureRenderPhase, Output,
};
use crate::net::minecraft::client::renderer::feature::submit::submit_node::SubmitNode;

#[derive(Default)]
pub struct SimpleFeatureRenderPhase {
    submits_by_feature: Vec<Option<FeatureSubmits>>,
}

/// `SimpleFeatureRenderPhase.FeatureSubmits`.
struct FeatureSubmits {
    feature_type: FeatureRendererType,
    unbatched: Vec<SubmitNode>,
    batches: Vec<(usize, Vec<SubmitNode>)>,
}

impl FeatureSubmits {
    fn add(&mut self, submit: SubmitNode) {
        match submit.batch_key() {
            None => self.unbatched.push(submit),
            Some(key) => match self.batches.iter_mut().find(|(batch, _)| *batch == key) {
                Some((_, batch)) => batch.push(submit),
                None => self.batches.push((key, vec![submit])),
            },
        }
    }

    fn is_empty(&self) -> bool {
        self.unbatched.is_empty() && self.batches.iter().all(|(_, submits)| submits.is_empty())
    }

    fn clear(&mut self) {
        self.unbatched.clear();
        self.batches.retain_mut(|(_, submits)| {
            if submits.is_empty() {
                return false;
            }
            submits.clear();
            true
        });
    }
}

impl SimpleFeatureRenderPhase {
    pub fn clear(&mut self) {
        for slot in &mut self.submits_by_feature {
            if let Some(submits) = slot {
                if submits.is_empty() {
                    *slot = None;
                } else {
                    submits.clear();
                }
            }
        }
    }
}

impl FeatureRenderPhase for SimpleFeatureRenderPhase {
    fn submit(&mut self, submit: SubmitNode) {
        let feature_type = submit.feature_type();
        let id = feature_type.id() as usize;
        if self.submits_by_feature.len() <= id {
            self.submits_by_feature.resize_with(id + 1, || None);
        }

        self.submits_by_feature[id]
            .get_or_insert_with(|| FeatureSubmits {
                feature_type,
                unbatched: Vec::new(),
                batches: Vec::new(),
            })
            .add(submit);
    }

    /// Each feature's unbatched submits, then its batches, as groups; then the phase is cleared
    /// like Java's `clear`, which keeps the batches that had submits (empty) for the next frame.
    fn sort_into(&mut self, output: &mut dyn Output) {
        for slot in &mut self.submits_by_feature {
            let Some(submits) = slot else {
                continue;
            };
            let was_empty = submits.is_empty();
            output.accept_feature_group(
                submits.feature_type,
                std::mem::take(&mut submits.unbatched),
                false,
            );
            let mut had_submits = Vec::with_capacity(submits.batches.len());
            for (_, batch) in &mut submits.batches {
                had_submits.push(!batch.is_empty());
                output.accept_feature_group(submits.feature_type, std::mem::take(batch), false);
            }

            if was_empty {
                *slot = None;
            } else {
                let mut had_submits = had_submits.into_iter();
                submits
                    .batches
                    .retain(|_| had_submits.next().expect("a flag for each batch"));
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.submits_by_feature
            .iter()
            .flatten()
            .all(FeatureSubmits::is_empty)
    }
}
