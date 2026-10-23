//! net/minecraft/world/entity/AbstractInterpolationHandler.java
//!
//! Java's handler holds its entity; here the entity is handed to each call, as what a handler
//! asks of it and its level (`Interpolated`), and the kinds (`LinearInterpolationHandler`,
//! `SteppedInterpolationHandler`) are what `InterpolationKind` asks of them.

use crate::net::minecraft::core::position_and_rotation::PositionAndRotation;
use crate::net::minecraft::world::entity::position_path::PositionPath;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// What a handler moves, and asks its level: `AbstractInterpolationHandler.entity`.
pub trait Interpolated {
    fn position(&self) -> Vec3;

    fn get_y_rot(&self) -> f32;

    fn get_x_rot(&self) -> f32;

    /// `setPos(Vec3)`.
    fn set_pos(&mut self, position: &Vec3);

    /// `setRot(float, float)`: each rotation `% 360`.
    fn set_rot(&mut self, y_rot: f32, x_rot: f32);

    /// `snapTo(Vec3, float, float)`.
    fn snap_to(&mut self, position: &Vec3, y_rot: f32, x_rot: f32);

    /// `level().noCollision(entity, makeBoundingBox(position))`.
    fn no_collision_at(&self, position: &Vec3) -> bool;

    /// `level().getRelativeTickSpeed()`.
    fn relative_tick_speed(&self) -> f32;

    /// `storePositionAndRotation()`.
    fn store_position_and_rotation(&self) -> PositionAndRotation {
        PositionAndRotation::of(self.position(), self.get_y_rot(), self.get_x_rot())
    }

    /// `onInterpolationStart(InterpolationHandler)`, which no class the port has overrides.
    fn on_interpolation_start(&mut self) {}
}

/// What a kind of handler adds to the shared one: its target and how it gets there.
pub trait InterpolationKind {
    /// `interpolationData()`: where it's going.
    fn interpolation_data(&self) -> &PositionAndRotation;

    /// `interpolationData().addDelta`, which a kind with steps moves them all by.
    fn add_delta(&mut self, delta: &Vec3);

    /// `interpolationData().addRotation`.
    fn add_rotation(&mut self, y_rot: f32, x_rot: f32);

    fn start_interpolating(
        &mut self,
        entity: &dyn Interpolated,
        position: &PositionPath,
        y_rot: f32,
        x_rot: f32,
        interpolation_steps: i32,
    );

    fn do_interpolate(&mut self, entity: &mut dyn Interpolated, interpolation_steps: i32);

    fn has_active_interpolation(&self) -> bool;

    fn cancel(&mut self);
}

/// The handler a kind shares the rest with.
#[derive(Clone, Debug, PartialEq)]
pub struct AbstractInterpolationHandler<K> {
    pub(crate) kind: K,
    interpolation_steps: i32,
    last_position_and_rotation: PositionAndRotation,
}

impl<K: InterpolationKind> AbstractInterpolationHandler<K> {
    pub fn new(kind: K, interpolation_steps: i32) -> AbstractInterpolationHandler<K> {
        AbstractInterpolationHandler {
            kind,
            interpolation_steps,
            last_position_and_rotation: PositionAndRotation::default(),
        }
    }

    pub fn set_interpolation_steps(&mut self, interpolation_steps: i32) {
        self.interpolation_steps = interpolation_steps;
    }

    pub fn target(&self) -> Option<PositionAndRotation> {
        self.kind
            .has_active_interpolation()
            .then(|| *self.kind.interpolation_data())
    }

    /// `interpolateTo(PositionPath, float, float, boolean)`: from where it's going, or where it
    /// is, to the path, keeping the rotation without one.
    pub fn interpolate_to(
        &mut self,
        entity: &mut dyn Interpolated,
        position: Option<&PositionPath>,
        y_rot: f32,
        x_rot: f32,
        has_rotation: bool,
    ) -> bool {
        let current = if self.kind.has_active_interpolation() {
            *self.kind.interpolation_data()
        } else {
            entity.store_position_and_rotation()
        };
        let position = position
            .cloned()
            .unwrap_or_else(|| PositionPath::of(current.position()));
        self.interpolate_to_path(
            entity,
            &position,
            if has_rotation { y_rot } else { current.y_rot() },
            if has_rotation { x_rot } else { current.x_rot() },
        );
        true
    }

    fn interpolate_to_path(
        &mut self,
        entity: &mut dyn Interpolated,
        position: &PositionPath,
        y_rot: f32,
        x_rot: f32,
    ) {
        if self.interpolation_steps == 0 {
            entity.snap_to(&position.end_position(), y_rot, x_rot);
            self.kind.cancel();
        } else if !self.kind.has_active_interpolation()
            || !self
                .kind
                .interpolation_data()
                .is(&position.end_position(), y_rot, x_rot)
        {
            self.kind
                .start_interpolating(entity, position, y_rot, x_rot, self.interpolation_steps);
            self.set_last_position_and_rotation(entity);
            entity.on_interpolation_start();
        }
    }

    pub fn apply_predicted_movement(&mut self, entity: &dyn Interpolated, delta: &Vec3) {
        self.adjust_interpolation_position(entity, delta);
        self.last_position_and_rotation.add_delta(delta);
    }

    pub fn interpolate(&mut self, entity: &mut dyn Interpolated) {
        if !self.kind.has_active_interpolation() {
            self.kind.cancel();
        } else {
            self.adjust_interpolation_target_from_deltas(entity);
            self.kind.do_interpolate(entity, self.interpolation_steps);
            self.set_last_position_and_rotation(entity);
        }
    }

    pub fn has_active_interpolation(&self) -> bool {
        self.kind.has_active_interpolation()
    }

    pub fn cancel(&mut self) {
        self.kind.cancel();
    }

    /// The target moved too, unless that would put the entity in something.
    fn adjust_interpolation_position(&mut self, entity: &dyn Interpolated, delta: &Vec3) {
        let adjusted_position = self.kind.interpolation_data().position().add(delta);
        if entity.no_collision_at(&adjusted_position) {
            self.kind.add_delta(delta);
        }
    }

    /// What the entity did itself since the last step (it moved, or turned) moves the target too.
    fn adjust_interpolation_target_from_deltas(&mut self, entity: &dyn Interpolated) {
        let delta_since_last_interpolation = entity
            .position()
            .subtract(&self.last_position_and_rotation.position());
        if delta_since_last_interpolation.length_sqr() > f64::from(1.0E-5f32) {
            self.adjust_interpolation_position(entity, &delta_since_last_interpolation);
        }

        let delta_y_rot = entity.get_y_rot() - self.last_position_and_rotation.y_rot();
        let delta_x_rot = entity.get_x_rot() - self.last_position_and_rotation.x_rot();
        self.kind.add_rotation(delta_y_rot, delta_x_rot);
    }

    fn set_last_position_and_rotation(&mut self, entity: &dyn Interpolated) {
        self.last_position_and_rotation.set(
            entity.position(),
            entity.get_y_rot(),
            entity.get_x_rot(),
        );
    }
}
