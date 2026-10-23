//! net/minecraft/world/entity/InterpolationHandler.java
//!
//! Java's interface, with `NO_OP` and the kinds that implement it, is an enum here. Not ported
//! yet: `interpolationTracker()`, the server's.

use crate::net::minecraft::core::position_and_rotation::PositionAndRotation;
use crate::net::minecraft::world::entity::abstract_interpolation_handler::{
    AbstractInterpolationHandler, Interpolated,
};
use crate::net::minecraft::world::entity::linear_interpolation_handler::LinearInterpolationHandler;
use crate::net::minecraft::world::entity::position_path::PositionPath;
use crate::net::minecraft::world::entity::stepped_interpolation_handler::SteppedInterpolationHandler;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// How an entity the server moves gets to where the server says.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum InterpolationHandler {
    /// `NO_OP`: straight there (`moveOrInterpolateTo` sets the position).
    #[default]
    NoOp,
    Linear(AbstractInterpolationHandler<LinearInterpolationHandler>),
    Stepped(AbstractInterpolationHandler<SteppedInterpolationHandler>),
}

impl InterpolationHandler {
    pub fn target(&self) -> Option<PositionAndRotation> {
        match self {
            InterpolationHandler::NoOp => None,
            InterpolationHandler::Linear(handler) => handler.target(),
            InterpolationHandler::Stepped(handler) => handler.target(),
        }
    }

    /// Whether it took the path: `NO_OP` doesn't, and the entity goes there itself.
    pub fn interpolate_to(
        &mut self,
        entity: &mut dyn Interpolated,
        position: Option<&PositionPath>,
        y_rot: f32,
        x_rot: f32,
        has_rotation: bool,
    ) -> bool {
        match self {
            InterpolationHandler::NoOp => false,
            InterpolationHandler::Linear(handler) => {
                handler.interpolate_to(entity, position, y_rot, x_rot, has_rotation)
            }
            InterpolationHandler::Stepped(handler) => {
                handler.interpolate_to(entity, position, y_rot, x_rot, has_rotation)
            }
        }
    }

    pub fn interpolate(&mut self, entity: &mut dyn Interpolated) {
        match self {
            InterpolationHandler::NoOp => {}
            InterpolationHandler::Linear(handler) => handler.interpolate(entity),
            InterpolationHandler::Stepped(handler) => handler.interpolate(entity),
        }
    }

    pub fn apply_predicted_movement(&mut self, entity: &dyn Interpolated, delta: &Vec3) {
        match self {
            InterpolationHandler::NoOp => {}
            InterpolationHandler::Linear(handler) => {
                handler.apply_predicted_movement(entity, delta)
            }
            InterpolationHandler::Stepped(handler) => {
                handler.apply_predicted_movement(entity, delta);
            }
        }
    }

    pub fn has_active_interpolation(&self) -> bool {
        match self {
            InterpolationHandler::NoOp => false,
            InterpolationHandler::Linear(handler) => handler.has_active_interpolation(),
            InterpolationHandler::Stepped(handler) => handler.has_active_interpolation(),
        }
    }

    pub fn cancel(&mut self) {
        match self {
            InterpolationHandler::NoOp => {}
            InterpolationHandler::Linear(handler) => handler.cancel(),
            InterpolationHandler::Stepped(handler) => handler.cancel(),
        }
    }
}
