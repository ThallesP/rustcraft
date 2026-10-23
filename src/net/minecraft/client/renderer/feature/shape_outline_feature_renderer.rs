//! net/minecraft/client/renderer/feature/ShapeOutlineFeatureRenderer.java
//!
//! Voxel shapes' outlines: each edge a line, from the pose, in the submit's color and width, its
//! normal the edge's direction.

use crate::com::mojang::blaze3d::vertex::pose_stack::Pose;
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::net::minecraft::client::renderer::feature::feature_frame_context::FeatureFrameContext;
use crate::net::minecraft::client::renderer::feature::render_type_feature_renderer::RenderTypeFeatureRenderer;
use crate::net::minecraft::client::renderer::feature::submit::submit_node::SubmitNode;
use crate::net::minecraft::client::renderer::rendertype::render_type::RenderType;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::org::joml::vector3f::Vector3f;
use std::rc::Rc;

/// `ShapeOutlineFeatureRenderer.Submit`.
pub struct ShapeOutlineSubmit {
    pub pose: Pose,
    pub shape: VoxelShape,
    pub render_type: Rc<RenderType>,
    pub color: i32,
    pub width: f32,
}

#[derive(Default)]
pub struct ShapeOutlineFeatureRenderer {
    base: RenderTypeFeatureRenderer,
}

impl ShapeOutlineFeatureRenderer {
    pub fn prepare_group(
        &mut self,
        context: &mut FeatureFrameContext<'_>,
        submits: &mut [SubmitNode],
        strictly_ordered: bool,
    ) {
        self.base
            .prepare_group(context, strictly_ordered, |context, group| {
                let mut normal = Vector3f::new();
                for submit in submits.iter() {
                    let SubmitNode::ShapeOutline(submit) = submit else {
                        unreachable!("a group holds one feature's submits");
                    };
                    let pose = &submit.pose;
                    let color = submit.color;
                    let width = submit.width;
                    group.with_vertex_builder(context, &submit.render_type, |builder| {
                        submit.shape.for_all_edges(&mut |x1, y1, z1, x2, y2, z2| {
                            normal
                                .set_xyz((x2 - x1) as f32, (y2 - y1) as f32, (z2 - z1) as f32)
                                .normalize();
                            for (x, y, z) in [(x1, y1, z1), (x2, y2, z2)] {
                                builder.add_vertex_matrix(
                                    pose.pose(),
                                    x as f32,
                                    y as f32,
                                    z as f32,
                                );
                                builder.set_color(color);
                                let mut transformed = Vector3f::new();
                                pose.transform_normal(&normal, &mut transformed);
                                builder.set_normal(transformed.x, transformed.y, transformed.z);
                                builder.set_line_width(width);
                            }
                        });
                    });
                }
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
