//! net/minecraft/client/renderer/SubmitNodeCollection.java
//!
//! One order's submits, by the phase they draw in. Only entity models are submitted so far
//! (`submitModel`), so only their phases are here: the solid models, the translucent ones, the
//! water mask and the outlines; the shadows', texts', blocks', items', particles', gizmos' and
//! the other phases come with what submits to them. Improved transparency's single phase
//! (`oitTranslucent`) comes with it.

use crate::com::mojang::blaze3d::vertex::pose_stack::PoseStack;
use crate::net::minecraft::client::model::model::Simple;
use crate::net::minecraft::client::renderer::feature::model_feature_renderer::ModelSubmit;
use crate::net::minecraft::client::renderer::feature::phase::feature_render_phase::FeatureRenderPhase;
use crate::net::minecraft::client::renderer::feature::phase::simple_feature_render_phase::SimpleFeatureRenderPhase;
use crate::net::minecraft::client::renderer::feature::phase::translucent_feature_render_phase::TranslucentFeatureRenderPhase;
use crate::net::minecraft::client::renderer::feature::submit::submit_node::SubmitNode;
use crate::net::minecraft::client::renderer::rendertype::render_type::RenderType;
use std::rc::Rc;

/// A phase of a collection, which a prepared frame keeps its groups by (Java's phases by
/// identity).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Solid,
    TranslucentModels,
    WaterMask,
    Outline,
}

impl Phase {
    /// `allPhases()`'s order, without improved transparency.
    pub const ALL: [Phase; 4] = [
        Phase::Solid,
        Phase::TranslucentModels,
        Phase::WaterMask,
        Phase::Outline,
    ];
}

#[derive(Default)]
pub struct SubmitNodeCollection {
    pub solid: SimpleFeatureRenderPhase,
    pub water_mask: SimpleFeatureRenderPhase,
    pub outline: SimpleFeatureRenderPhase,
    pub translucent_models: TranslucentFeatureRenderPhase,
}

impl SubmitNodeCollection {
    pub fn phase(&mut self, phase: Phase) -> &mut dyn FeatureRenderPhase {
        match phase {
            Phase::Solid => &mut self.solid,
            Phase::TranslucentModels => &mut self.translucent_models,
            Phase::WaterMask => &mut self.water_mask,
            Phase::Outline => &mut self.outline,
        }
    }

    /// `submitModel` of a simple model: into the solid phase, or the translucent one if its render
    /// type blends. The water mask's render type and the outlines (an outline color) come with
    /// what uses them.
    #[allow(clippy::too_many_arguments)]
    pub fn submit_model(
        &mut self,
        model: Simple,
        pose_stack: &PoseStack,
        render_type: &Rc<RenderType>,
        light_coords: i32,
        overlay_coords: i32,
        tinted_color: i32,
    ) {
        let pose = pose_stack.last().clone();
        if !render_type.is_outline() {
            let submit = SubmitNode::Model(ModelSubmit {
                render_type: Rc::clone(render_type),
                pose,
                model,
                light_coords,
                overlay_coords,
                tinted_color,
            });
            if !render_type.force_solid_model_phase() && render_type.has_blending() {
                self.translucent_models.submit(submit);
            } else {
                self.solid.submit(submit);
            }
        }
    }
}
