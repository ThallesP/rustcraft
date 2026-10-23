//! net/minecraft/world/entity/SteppedInterpolationHandler.java
//!
//! `create` gives the server a `SteppedInterpolationTracker`, which records the steps its moves
//! send; the port only makes handlers for the client's entities.

use std::collections::VecDeque;

use crate::net::minecraft::core::position_and_rotation::PositionAndRotation;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::abstract_interpolation_handler::{
    AbstractInterpolationHandler, Interpolated, InterpolationKind,
};
use crate::net::minecraft::world::entity::position_path::PositionPath;
use crate::net::minecraft::world::entity::position_step::PositionStep;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// Through the steps the server's moves send, each over its ticks, faster while it's behind.
#[derive(Clone, Debug, PartialEq)]
pub struct SteppedInterpolationHandler {
    interpolation_data: InterpolationData,
}

/// `SteppedInterpolationHandler.InterpolationData`: the target (its `Mutable`), the steps left,
/// where the last step ended, and how fast it goes.
#[derive(Clone, Debug, PartialEq)]
struct InterpolationData {
    target: PositionAndRotation,
    remaining_steps: VecDeque<Step>,
    last_step_pos_rot: PositionAndRotation,
    current_step_ticks: f32,
    remaining_ticks: f32,
    interpolation_speed: f32,
}

/// `SteppedInterpolationHandler.Step`: where a step goes, over so many ticks.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Step {
    pos_rot: PositionAndRotation,
    tick_offset: i32,
}

impl SteppedInterpolationHandler {
    /// A living entity's: over its type's update interval.
    pub fn create(
        update_interval: i32,
    ) -> AbstractInterpolationHandler<SteppedInterpolationHandler> {
        AbstractInterpolationHandler::new(
            SteppedInterpolationHandler {
                interpolation_data: InterpolationData {
                    target: PositionAndRotation::default(),
                    remaining_steps: VecDeque::new(),
                    last_step_pos_rot: PositionAndRotation::default(),
                    current_step_ticks: 0.0,
                    remaining_ticks: 0.0,
                    interpolation_speed: 1.0,
                },
            },
            update_interval,
        )
    }
}

impl InterpolationKind for SteppedInterpolationHandler {
    fn interpolation_data(&self) -> &PositionAndRotation {
        &self.interpolation_data.target
    }

    fn add_delta(&mut self, delta: &Vec3) {
        self.interpolation_data.add_delta(delta);
    }

    fn add_rotation(&mut self, y_rot: f32, x_rot: f32) {
        self.interpolation_data.add_rotation(y_rot, x_rot);
    }

    fn start_interpolating(
        &mut self,
        entity: &dyn Interpolated,
        position: &PositionPath,
        y_rot: f32,
        x_rot: f32,
        interpolation_steps: i32,
    ) {
        let data = &mut self.interpolation_data;
        if data.remaining_steps.is_empty() {
            data.set_starting_point(entity.position(), entity.get_y_rot(), entity.get_x_rot());
        }

        let end_position = position.end_position();
        if end_position == data.target.position() {
            data.add_step(end_position, y_rot, x_rot, interpolation_steps);
        } else {
            data.add_steps(position, y_rot, x_rot, interpolation_steps);
        }

        data.target.set(end_position, y_rot, x_rot);
    }

    fn do_interpolate(&mut self, entity: &mut dyn Interpolated, interpolation_steps: i32) {
        let target = self.interpolation_data.get_new_position_and_rotation();
        entity.set_pos(&target.position());
        entity.set_rot(target.y_rot(), target.x_rot());
        let tick = entity.relative_tick_speed();
        self.interpolation_data.advance(tick, interpolation_steps);
    }

    fn has_active_interpolation(&self) -> bool {
        !self.interpolation_data.remaining_steps.is_empty()
    }

    fn cancel(&mut self) {
        self.interpolation_data.reset();
    }
}

impl InterpolationData {
    fn advance(&mut self, mut ticks: f32, interpolation_steps: i32) {
        let target_speed = (self.remaining_ticks / interpolation_steps as f32).max(1.0);
        self.interpolation_speed = Mth::lerp_f32(
            1.0 / interpolation_steps as f32,
            self.interpolation_speed,
            target_speed,
        );
        if ticks * self.interpolation_speed < self.remaining_ticks {
            ticks *= self.interpolation_speed;
        } else {
            ticks = self.remaining_ticks;
            self.interpolation_speed = 1.0;
        }

        self.current_step_ticks += ticks;
        self.remaining_ticks -= ticks;
    }

    fn reset(&mut self) {
        self.remaining_steps.clear();
        self.remaining_ticks = 0.0;
        self.interpolation_speed = 1.0;
    }

    fn add_delta(&mut self, delta: &Vec3) {
        self.target.add_delta(delta);
        for step in &mut self.remaining_steps {
            step.pos_rot.add_delta(delta);
        }
        self.last_step_pos_rot.add_delta(delta);
    }

    fn add_rotation(&mut self, y_rot: f32, x_rot: f32) {
        self.target.add_rotation(y_rot, x_rot);
        for step in &mut self.remaining_steps {
            step.pos_rot.add_rotation(y_rot, x_rot);
        }
        self.last_step_pos_rot.add_rotation(y_rot, x_rot);
    }

    fn set_starting_point(&mut self, position: Vec3, y_rot: f32, x_rot: f32) {
        self.last_step_pos_rot.set(position, y_rot, x_rot);
        self.current_step_ticks = 1.0;
    }

    fn add_step(&mut self, position: Vec3, y_rot: f32, x_rot: f32, interpolation_steps: i32) {
        self.remaining_steps.push_back(Step {
            pos_rot: PositionAndRotation::of(position, y_rot, x_rot),
            tick_offset: interpolation_steps,
        });
        self.remaining_ticks += interpolation_steps as f32;
    }

    fn add_steps(
        &mut self,
        position: &PositionPath,
        y_rot: f32,
        x_rot: f32,
        interpolation_steps: i32,
    ) {
        match position {
            PositionPath::Linear { end_position } => {
                self.add_step(*end_position, y_rot, x_rot, interpolation_steps);
            }
            PositionPath::Stepped { steps, .. } => {
                if y_rot == self.target.y_rot() && x_rot == self.target.x_rot() {
                    for step in steps {
                        self.add_step(step.position(), y_rot, x_rot, step.tick_offset());
                    }
                    return;
                }

                let total_interpolation_ticks = InterpolationData::get_interpolation_ticks(steps);
                let mut offset = 0i32;
                for step in steps {
                    offset = i32::wrapping_add(offset, step.tick_offset());
                    let a = offset as f32 / total_interpolation_ticks as f32;
                    self.add_step(
                        step.position(),
                        Mth::rot_lerp_f32(a, self.target.y_rot(), y_rot),
                        Mth::lerp_f32(a, self.target.x_rot(), x_rot),
                        step.tick_offset(),
                    );
                }
            }
        }
    }

    fn get_interpolation_ticks(steps: &[PositionStep]) -> i32 {
        steps
            .iter()
            .fold(0i32, |ticks, step| ticks.wrapping_add(step.tick_offset()))
    }

    /// Where the step it's in has got to, past the steps it's done with; the target when none is
    /// left.
    fn get_new_position_and_rotation(&mut self) -> PositionAndRotation {
        while let Some(step) = self.remaining_steps.front().copied() {
            let offset = step.tick_offset;
            if self.current_step_ticks < offset as f32 {
                let a = self.current_step_ticks / offset as f32;
                return PositionAndRotation::of(
                    self.last_step_pos_rot
                        .position()
                        .lerp(&step.pos_rot.position(), f64::from(a)),
                    Mth::rot_lerp_f32(a, self.last_step_pos_rot.y_rot(), step.pos_rot.y_rot()),
                    Mth::lerp_f32(a, self.last_step_pos_rot.x_rot(), step.pos_rot.x_rot()),
                );
            }

            self.current_step_ticks -= offset as f32;
            self.last_step_pos_rot.set(
                step.pos_rot.position(),
                step.pos_rot.y_rot(),
                step.pos_rot.x_rot(),
            );
            self.remaining_steps.pop_front();
        }

        self.target
    }
}
