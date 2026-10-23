//! net/minecraft/client/model/Model.java
//!
//! A model: its root part, how it draws (`renderToBuffer`), and how it poses for a render state
//! (`setupAnim`, which resets every part's pose by default). The render type a model picks for a
//! texture (Java's `renderType` function) is the caller's so far. Only `Model.Simple`, a part that
//! doesn't animate, is drawn yet: the entities' models come with entity rendering.

use crate::com::mojang::blaze3d::vertex::pose_stack::PoseStack;
use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::net::minecraft::client::model::geom::model_part::ModelPart;

pub trait Model {
    fn root(&self) -> &ModelPart;

    fn root_mut(&mut self) -> &mut ModelPart;

    /// `setupAnim(state)`'s default: every part back to its initial pose.
    fn setup_anim(&mut self) {
        self.reset_pose();
    }

    fn reset_pose(&mut self) {
        self.root_mut().reset_all_poses();
    }

    fn render_to_buffer(
        &self,
        pose_stack: &mut PoseStack,
        buffer: &mut dyn VertexConsumer,
        light_coords: i32,
        overlay_coords: i32,
        color: i32,
    ) {
        self.root()
            .render_colored(pose_stack, buffer, light_coords, overlay_coords, color);
    }
}

/// `Model.Simple`: a part drawn as it's posed.
#[derive(Clone, Debug)]
pub struct Simple {
    root: ModelPart,
}

impl Simple {
    pub fn new(root: ModelPart) -> Simple {
        Simple { root }
    }
}

impl Model for Simple {
    fn root(&self) -> &ModelPart {
        &self.root
    }

    fn root_mut(&mut self) -> &mut ModelPart {
        &mut self.root
    }

    /// A simple model keeps its pose.
    fn setup_anim(&mut self) {}
}
