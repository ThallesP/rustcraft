//! net/minecraft/world/entity/LinearInterpolationHandler.java
//!
//! `create` gives the server no handler (`NO_OP`); the port only makes them for the client's
//! entities.

use crate::net::minecraft::core::position_and_rotation::PositionAndRotation;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::abstract_interpolation_handler::{
    AbstractInterpolationHandler, Interpolated, InterpolationKind,
};
use crate::net::minecraft::world::entity::position_path::PositionPath;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// Straight to the target, a fraction of the way left each tick, over so many steps.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LinearInterpolationHandler {
    interpolation_data: PositionAndRotation,
    remaining_steps: i32,
}

impl LinearInterpolationHandler {
    pub const DEFAULT_INTERPOLATION_STEPS: i32 = 3;

    pub fn create(
        interpolation_steps: i32,
    ) -> AbstractInterpolationHandler<LinearInterpolationHandler> {
        AbstractInterpolationHandler::new(
            LinearInterpolationHandler::default(),
            interpolation_steps,
        )
    }
}

impl InterpolationKind for LinearInterpolationHandler {
    fn interpolation_data(&self) -> &PositionAndRotation {
        &self.interpolation_data
    }

    fn add_delta(&mut self, delta: &Vec3) {
        self.interpolation_data.add_delta(delta);
    }

    fn add_rotation(&mut self, y_rot: f32, x_rot: f32) {
        self.interpolation_data.add_rotation(y_rot, x_rot);
    }

    fn start_interpolating(
        &mut self,
        _entity: &dyn Interpolated,
        position: &PositionPath,
        y_rot: f32,
        x_rot: f32,
        interpolation_steps: i32,
    ) {
        self.interpolation_data
            .set(position.end_position(), y_rot, x_rot);
        self.remaining_steps = interpolation_steps;
    }

    fn do_interpolate(&mut self, entity: &mut dyn Interpolated, _interpolation_steps: i32) {
        let alpha = 1.0 / f64::from(self.remaining_steps);
        let new_position = entity
            .position()
            .lerp(&self.interpolation_data.position(), alpha);
        let new_y_rot = Mth::rot_lerp_f64(
            alpha,
            f64::from(entity.get_y_rot()),
            f64::from(self.interpolation_data.y_rot()),
        ) as f32;
        let new_x_rot = Mth::lerp_f64(
            alpha,
            f64::from(entity.get_x_rot()),
            f64::from(self.interpolation_data.x_rot()),
        ) as f32;
        entity.set_pos(&new_position);
        entity.set_rot(new_y_rot, new_x_rot);
        self.remaining_steps -= 1;
    }

    fn has_active_interpolation(&self) -> bool {
        self.remaining_steps > 0
    }

    fn cancel(&mut self) {
        self.remaining_steps = 0;
    }
}
