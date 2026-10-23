//! net/minecraft/client/renderer/feature/FeatureRenderDispatcher.java
//!
//! Turns a frame's submits into draws: `prepare_frame` drains the storage's phases into groups
//! (each phase's submits, sorted, in runs of one feature type that do or don't have to keep their
//! order), has each feature renderer prepare its groups into the staged vertex buffer, and uploads
//! it; `render_all_features` then draws the groups phase by phase, in Java's order. Only the
//! ported renderers' phases run; the others come with their renderers. Java keeps the groups by phase and by feature in `IdentityHashMap`s; a group is an
//! index into the frame's groups here, kept in both. Only the entity models' and the shape
//! outlines' renderers are ported.

use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::net::minecraft::client::renderer::feature::feature_frame_context::FeatureFrameContext;
use crate::net::minecraft::client::renderer::feature::feature_renderer_type::FeatureRendererType;
use crate::net::minecraft::client::renderer::feature::model_feature_renderer::ModelFeatureRenderer;
use crate::net::minecraft::client::renderer::feature::phase::feature_render_phase::Output;
use crate::net::minecraft::client::renderer::feature::shape_outline_feature_renderer::ShapeOutlineFeatureRenderer;
use crate::net::minecraft::client::renderer::feature::submit::submit_node::SubmitNode;
use crate::net::minecraft::client::renderer::submit_node_collection::Phase;
use crate::net::minecraft::client::renderer::submit_node_storage::SubmitNodeStorage;

#[derive(Default)]
pub struct FeatureRenderDispatcher {
    model_feature_renderer: ModelFeatureRenderer,
    shape_outline_feature_renderer: ShapeOutlineFeatureRenderer,
}

/// A phase of the storage: one order's. The see-through phase all orders share comes with name
/// tags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PhaseKey {
    Ordered(i32, Phase),
}

/// `FeatureRenderDispatcher.PreparedGroup`: a run of `all_submits` of one feature type.
#[derive(Clone, Copy, Debug)]
struct PreparedGroup {
    feature_group_index: usize,
    feature_type: FeatureRendererType,
    strictly_ordered: bool,
    from_inclusive: usize,
    to_inclusive: usize,
}

/// `FeatureRenderDispatcher.PreparedFrame`: the frame's submits in the order their groups take
/// them, and its groups.
#[derive(Default)]
pub struct PreparedFrame {
    all_submits: Vec<SubmitNode>,
    groups: Vec<PreparedGroup>,
    groups_by_phase: Vec<(PhaseKey, Vec<usize>)>,
    groups_by_feature: Vec<(FeatureRendererType, Vec<usize>)>,
    /// The storage's orders when the frame was prepared (`getSubmitsPerOrder().values()`).
    orders: Vec<i32>,
}

/// `FeatureRenderDispatcher.PhaseSubmitGrouper`.
struct PhaseSubmitGrouper<'a> {
    frame: &'a mut PreparedFrame,
    phase: PhaseKey,
    last_group: Option<usize>,
}

impl PhaseSubmitGrouper<'_> {
    fn add_or_extend_group(
        &mut self,
        feature_type: FeatureRendererType,
        strictly_ordered: bool,
        from_inclusive: usize,
        to_inclusive: usize,
    ) {
        if let Some(last) = self.last_group {
            let group = &mut self.frame.groups[last];
            if group.feature_type == feature_type && group.strictly_ordered == strictly_ordered {
                group.to_inclusive = to_inclusive;
                return;
            }
        }
        let feature_groups =
            PreparedFrame::groups_of(&mut self.frame.groups_by_feature, feature_type);
        let group = PreparedGroup {
            feature_group_index: feature_groups.len(),
            feature_type,
            strictly_ordered,
            from_inclusive,
            to_inclusive,
        };
        let index = self.frame.groups.len();
        feature_groups.push(index);
        self.frame.groups.push(group);
        PreparedFrame::groups_of(&mut self.frame.groups_by_phase, self.phase).push(index);
        self.last_group = Some(index);
    }
}

impl Output for PhaseSubmitGrouper<'_> {
    fn accept(&mut self, submit: SubmitNode, strictly_ordered: bool) {
        let index = self.frame.all_submits.len();
        let feature_type = submit.feature_type();
        self.frame.all_submits.push(submit);
        self.add_or_extend_group(feature_type, strictly_ordered, index, index);
    }

    /// Panics where Java throws `IllegalArgumentException` for a submit of another type.
    fn accept_feature_group(
        &mut self,
        feature_type: FeatureRendererType,
        submits: Vec<SubmitNode>,
        strictly_ordered: bool,
    ) {
        if submits.is_empty() {
            return;
        }
        for submit in &submits {
            assert!(
                submit.feature_type() == feature_type,
                "a submit was not of feature type {}",
                feature_type.name()
            );
        }
        let from_inclusive = self.frame.all_submits.len();
        self.frame.all_submits.extend(submits);
        let to_inclusive = self.frame.all_submits.len() - 1;
        self.add_or_extend_group(feature_type, strictly_ordered, from_inclusive, to_inclusive);
    }
}

impl PreparedFrame {
    fn groups_of<K: PartialEq + Copy>(
        groups: &mut Vec<(K, Vec<usize>)>,
        key: K,
    ) -> &mut Vec<usize> {
        let position = match groups.iter().position(|(listed, _)| *listed == key) {
            Some(position) => position,
            None => {
                groups.push((key, Vec::new()));
                groups.len() - 1
            }
        };
        &mut groups[position].1
    }

    fn phase_groups(&self, phase: PhaseKey) -> &[usize] {
        self.groups_by_phase
            .iter()
            .find(|(listed, _)| *listed == phase)
            .map_or(&[], |(_, groups)| groups.as_slice())
    }

    pub fn is_empty(&self) -> bool {
        self.all_submits.is_empty()
    }
}

impl FeatureRenderDispatcher {
    pub fn new() -> FeatureRenderDispatcher {
        FeatureRenderDispatcher::default()
    }

    /// `prepareFrame`: the storage's submits sorted into groups, prepared, and uploaded.
    pub fn prepare_frame(
        &mut self,
        context: &mut FeatureFrameContext<'_>,
        submit_node_storage: &mut SubmitNodeStorage,
    ) -> PreparedFrame {
        let mut frame = PreparedFrame::default();
        submit_node_storage.drain_phases(|order, phase, collection| {
            let mut grouper = PhaseSubmitGrouper {
                frame: &mut frame,
                phase: PhaseKey::Ordered(order, phase),
                last_group: None,
            };
            collection.phase(phase).sort_into(&mut grouper);
        });
        frame.orders = submit_node_storage.orders().copied().collect();

        let PreparedFrame {
            all_submits,
            groups,
            groups_by_feature,
            ..
        } = &mut frame;
        for (feature_type, feature_groups) in groups_by_feature.iter() {
            for &group in feature_groups {
                let group = groups[group];
                let submits = &mut all_submits[group.from_inclusive..=group.to_inclusive];
                match *feature_type {
                    FeatureRendererType::MODEL => self.model_feature_renderer.prepare_group(
                        context,
                        submits,
                        group.strictly_ordered,
                    ),
                    FeatureRendererType::SHAPE_OUTLINE => self
                        .shape_outline_feature_renderer
                        .prepare_group(context, submits, group.strictly_ordered),
                    _ => unreachable!("only the ported renderers make submits"),
                }
            }
        }

        context
            .staged_vertex_buffer
            .upload(context.device, &mut context.sequential_buffers);
        frame
    }

    /// `renderAllFeatures`: the solid phases, then the translucent ones and after-terrain's, of
    /// the phases ported (see-through and always-on-top come with what submits to them).
    pub fn render_all_features(
        &self,
        context: &mut FeatureFrameContext<'_>,
        render_pass: &mut dyn RenderPass,
        frame: &PreparedFrame,
    ) {
        self.execute_solid(context, render_pass, frame);
        self.execute_translucent(context, render_pass, frame);
        self.execute_translucent_after_terrain(context, render_pass, frame);
    }

    /// `PreparedFrame.executeSolid`.
    pub fn execute_solid(
        &self,
        context: &mut FeatureFrameContext<'_>,
        render_pass: &mut dyn RenderPass,
        frame: &PreparedFrame,
    ) {
        for &order in &frame.orders {
            self.execute_phase(
                context,
                render_pass,
                frame,
                PhaseKey::Ordered(order, Phase::Solid),
            );
        }
    }

    /// `PreparedFrame.executeTranslucent`: each order's translucent models (with the shadows, name
    /// tags, texts and custom geometry, not ported), then each order's shape outlines (and
    /// gizmos), then each order's water mask (after the blocks, items and breaking overlay).
    pub fn execute_translucent(
        &self,
        context: &mut FeatureFrameContext<'_>,
        render_pass: &mut dyn RenderPass,
        frame: &PreparedFrame,
    ) {
        for phase in [
            Phase::TranslucentModels,
            Phase::ShapeOutlines,
            Phase::WaterMask,
        ] {
            for &order in &frame.orders {
                self.execute_phase(context, render_pass, frame, PhaseKey::Ordered(order, phase));
            }
        }
    }

    /// `PreparedFrame.executeTranslucentAfterTerrain`.
    pub fn execute_translucent_after_terrain(
        &self,
        context: &mut FeatureFrameContext<'_>,
        render_pass: &mut dyn RenderPass,
        frame: &PreparedFrame,
    ) {
        for &order in &frame.orders {
            self.execute_phase(
                context,
                render_pass,
                frame,
                PhaseKey::Ordered(order, Phase::AfterTerrain),
            );
        }
    }

    fn execute_phase(
        &self,
        context: &mut FeatureFrameContext<'_>,
        render_pass: &mut dyn RenderPass,
        frame: &PreparedFrame,
        phase: PhaseKey,
    ) {
        for &group in frame.phase_groups(phase) {
            let group = frame.groups[group];
            render_pass.push_debug_group(group.feature_type.name());
            match group.feature_type {
                FeatureRendererType::MODEL => self.model_feature_renderer.execute_group(
                    context,
                    render_pass,
                    group.feature_group_index,
                ),
                FeatureRendererType::SHAPE_OUTLINE => self
                    .shape_outline_feature_renderer
                    .execute_group(context, render_pass, group.feature_group_index),
                _ => unreachable!("only the ported renderers make submits"),
            }
            render_pass.pop_debug_group();
        }
    }

    /// `PreparedFrame.close()`: the renderers' groups and the staged draws let go.
    pub fn close_frame(&mut self, context: &mut FeatureFrameContext<'_>, _frame: PreparedFrame) {
        self.model_feature_renderer.finish_execute();
        self.shape_outline_feature_renderer.finish_execute();
        context.staged_vertex_buffer.end_draw();
    }
}
