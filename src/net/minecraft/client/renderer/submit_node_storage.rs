//! net/minecraft/client/renderer/SubmitNodeStorage.java, with `SubmitNodeCollector` and
//! `OrderedSubmitNodeCollector`.
//!
//! A frame's submits by their order (lower orders draw first in each phase); the collector's
//! methods submit at order 0. Only `submitModelPart` is ported (`submitModel` of a
//! `Model.Simple`): the other kinds of submit come with what makes them, and so do the
//! see-through phase (the name tags') and improved transparency.

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::com::mojang::blaze3d::vertex::pose_stack::PoseStack;
use crate::net::minecraft::client::model::geom::model_part::ModelPart;
use crate::net::minecraft::client::model::model::Simple;
use crate::net::minecraft::client::renderer::rendertype::render_type::RenderType;
use crate::net::minecraft::client::renderer::submit_node_collection::{
    Phase, SubmitNodeCollection,
};

#[derive(Default)]
pub struct SubmitNodeStorage {
    submits_per_order: BTreeMap<i32, SubmitNodeCollection>,
}

impl SubmitNodeStorage {
    pub fn new() -> SubmitNodeStorage {
        SubmitNodeStorage::default()
    }

    pub fn order(&mut self, order: i32) -> &mut SubmitNodeCollection {
        self.submits_per_order.entry(order).or_default()
    }

    /// `submitModelPart(modelPart, poseStack, renderType, lightCoords, overlayCoords, uvMapping)`
    /// without a texture remapping: the part as it's posed now, white.
    pub fn submit_model_part(
        &mut self,
        model_part: &ModelPart,
        pose_stack: &PoseStack,
        render_type: &Rc<RenderType>,
        light_coords: i32,
        overlay_coords: i32,
    ) {
        self.order(0).submit_model(
            Simple::new(model_part.clone()),
            pose_stack,
            render_type,
            light_coords,
            overlay_coords,
            -1,
        );
    }

    pub fn orders(&self) -> impl Iterator<Item = &i32> {
        self.submits_per_order.keys()
    }

    /// `drainPhases`: each order's phases that have submits, in order, and the orders that had
    /// none dropped.
    pub fn drain_phases(
        &mut self,
        mut consumer: impl FnMut(i32, Phase, &mut SubmitNodeCollection),
    ) {
        self.submits_per_order.retain(|&order, collection| {
            let mut empty = true;
            for phase in Phase::ALL {
                if !collection.phase(phase).is_empty() {
                    consumer(order, phase, collection);
                    empty = false;
                }
            }
            !empty
        });
    }
}
