//! net/minecraft/client/renderer/feature/ModelFeatureRenderer.java
//!
//! Entity models' submits: each model, posed for its state (`setupAnim`), written at its pose into
//! its render type's draw. Only `Model.Simple` submits are made so far (`submitModelPart`); the
//! texture remapping (`UvMapping`), the breaking overlay's decal (`CrumblingOverlay`,
//! `SheetedDecalTextureGenerator`) and the outlines come with what submits them.

use std::rc::Rc;

use crate::com::mojang::blaze3d::vertex::pose_stack::{Pose, PoseStack};
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::net::minecraft::client::model::model::{Model, Simple};
use crate::net::minecraft::client::renderer::feature::feature_frame_context::FeatureFrameContext;
use crate::net::minecraft::client::renderer::feature::render_type_feature_renderer::{
    Group, RenderTypeFeatureRenderer,
};
use crate::net::minecraft::client::renderer::feature::submit::submit_node::SubmitNode;
use crate::net::minecraft::client::renderer::rendertype::render_type::RenderType;

/// `ModelFeatureRenderer.Submit`: a model as it was posed when submitted (Java holds the model
/// itself, which nothing changes before the frame draws it).
pub struct ModelSubmit {
    pub render_type: Rc<RenderType>,
    pub pose: Pose,
    pub model: Simple,
    pub light_coords: i32,
    pub overlay_coords: i32,
    pub tinted_color: i32,
}

#[derive(Default)]
pub struct ModelFeatureRenderer {
    base: RenderTypeFeatureRenderer,
    pose_stack: PoseStack,
}

impl ModelFeatureRenderer {
    pub fn prepare_group(
        &mut self,
        context: &mut FeatureFrameContext<'_>,
        submits: &mut [SubmitNode],
        strictly_ordered: bool,
    ) {
        let pose_stack = &mut self.pose_stack;
        self.base
            .prepare_group(context, strictly_ordered, |context, group| {
                for submit in submits {
                    let SubmitNode::Model(submit) = submit;
                    ModelFeatureRenderer::prepare_model(pose_stack, context, group, submit);
                }
            });
    }

    fn prepare_model(
        pose_stack: &mut PoseStack,
        context: &mut FeatureFrameContext<'_>,
        group: &mut Group,
        submit: &mut ModelSubmit,
    ) {
        pose_stack.last_mut().set(&submit.pose);
        submit.model.setup_anim();
        let model = &submit.model;
        group.with_vertex_builder(context, &submit.render_type, |buffer| {
            model.render_to_buffer(
                pose_stack,
                buffer,
                submit.light_coords,
                submit.overlay_coords,
                submit.tinted_color,
            );
        });
    }

    pub fn execute_group(
        &self,
        context: &mut FeatureFrameContext<'_>,
        render_pass: &mut dyn RenderPass,
        group_index: usize,
    ) {
        self.base.execute_group(context, render_pass, group_index);
    }

    pub fn finish_execute(&mut self) {
        self.base.finish_execute();
    }
}
