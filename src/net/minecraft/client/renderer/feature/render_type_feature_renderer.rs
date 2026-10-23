//! net/minecraft/client/renderer/feature/RenderTypeFeatureRenderer.java
//!
//! A feature renderer that draws its submits through render types: each group of submits it
//! prepares writes its vertices into the staged buffer, a draw for each run of one render type
//! (or, where the group may be reordered, one draw for each render type), and executing the group
//! draws them in order. Java's is an abstract class whose subclasses build a group; here it's a
//! part of each, which hands its `build_group` the group to write into.

use std::rc::Rc;

use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::net::minecraft::client::renderer::feature::feature_frame_context::FeatureFrameContext;
use crate::net::minecraft::client::renderer::rendertype::prepared_render_type::PreparedRenderType;
use crate::net::minecraft::client::renderer::rendertype::render_type::RenderType;
use crate::net::minecraft::client::renderer::staged_vertex_buffer::DrawId;

#[derive(Default)]
pub struct RenderTypeFeatureRenderer {
    groups: Vec<Group>,
}

impl RenderTypeFeatureRenderer {
    /// `prepareGroup`: a group built by `build_group`, then every group's draws' indices asked
    /// for.
    pub fn prepare_group(
        &mut self,
        context: &mut FeatureFrameContext<'_>,
        strictly_ordered: bool,
        build_group: impl FnOnce(&mut FeatureFrameContext<'_>, &mut Group),
    ) {
        let mut current_group = Group::new(!strictly_ordered);
        build_group(context, &mut current_group);
        self.groups.push(current_group);

        for group in &self.groups {
            for &draw in &group.draws {
                context
                    .staged_vertex_buffer
                    .request_index_count(draw, &mut context.sequential_buffers);
            }
        }
    }

    /// `executeGroup`: the group's draws, each with its render type. OIT's stages come with
    /// improved transparency.
    pub fn execute_group(
        &self,
        context: &mut FeatureFrameContext<'_>,
        render_pass: &mut dyn RenderPass,
        group_index: usize,
    ) {
        let group = &self.groups[group_index];
        for (draw, render_type) in group.draws.iter().zip(&group.draw_render_types) {
            let info = context.staged_vertex_buffer.get_execute_info(
                *draw,
                context.device,
                &mut context.sequential_buffers,
            );
            if let Some(info) = info {
                render_type.draw_from_buffer(&info, render_pass);
            }
        }
    }

    pub fn finish_execute(&mut self) {
        self.groups.clear();
    }
}

/// `RenderTypeFeatureRenderer.Group`.
pub struct Group {
    can_reorder: bool,
    draws: Vec<DrawId>,
    draw_render_types: Vec<PreparedRenderType>,
    last_render_type: Option<Rc<RenderType>>,
    last_draw: Option<DrawId>,
}

impl Group {
    fn new(can_reorder: bool) -> Group {
        Group {
            can_reorder,
            draws: Vec::new(),
            draw_render_types: Vec::new(),
            last_render_type: None,
            last_draw: None,
        }
    }

    /// `getVertexBuilder(renderType)`, lent to `build`: the last draw if it's of this render type
    /// and can take more, otherwise another.
    pub fn with_vertex_builder(
        &mut self,
        context: &mut FeatureFrameContext<'_>,
        render_type: &Rc<RenderType>,
        build: impl FnOnce(&mut dyn VertexConsumer),
    ) {
        let same = self
            .last_render_type
            .as_ref()
            .is_some_and(|last| Rc::ptr_eq(last, render_type));
        if self.last_draw.is_none() || !same || !render_type.can_consolidate_consecutive_geometry()
        {
            self.last_draw = Some(self.get_or_add_draw(context, render_type));
            self.last_render_type = Some(Rc::clone(render_type));
        }

        let draw = self.last_draw.expect("set above");
        context
            .staged_vertex_buffer
            .with_vertex_builder(draw, build);
    }

    fn get_or_add_draw(
        &mut self,
        context: &mut FeatureFrameContext<'_>,
        render_type: &Rc<RenderType>,
    ) -> DrawId {
        let prepared_render_type = render_type.prepare(context);
        if self.can_reorder && render_type.can_consolidate_consecutive_geometry() {
            let existing = self
                .draw_render_types
                .iter()
                .position(|prepared| *prepared == prepared_render_type);
            if let Some(existing) = existing {
                return self.draws[existing];
            }
        }

        let quad_sorting = (render_type.sort_on_upload() && !context.use_improved_transparency)
            .then(|| context.projection_type.vertex_sorting());
        let draw = context.staged_vertex_buffer.append_draw(
            render_type.format(),
            render_type.primitive_topology(),
            quad_sorting,
        );
        self.draws.push(draw);
        self.draw_render_types.push(prepared_render_type);
        draw
    }
}
