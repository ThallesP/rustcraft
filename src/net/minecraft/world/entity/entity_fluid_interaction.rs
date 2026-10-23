//! net/minecraft/world/entity/EntityFluidInteraction.java
//!
//! What an entity knows about the fluids it's in, worked out once a tick (`update`): for each
//! fluid type its box reaches (`Tracker`), how high above its feet the fluid goes and whether its
//! eyes are in it, and for each fluid tag with a current (water and lava), the flows of the
//! fluid's blocks it's in (`CurrentAccumulator`), which push it (`applyCurrentTo`).
//!
//! A fluid type is Java's `Holder<Fluid>`, the fluid itself here, so still and flowing water have
//! a tracker each, like in Java. Java's map of accumulators is a `Reference2ObjectArrayMap` over
//! `Set.of(WATER, LAVA)`, whose order only decides which tag a fluid in both would push with; no
//! fluid is in both.

// Java's negated comparisons are kept as they are: they differ from the plain ones for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use crate::java::lang::math::Math;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::tags::fluid_tags::FluidTagKey;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// The fluids an entity is in.
#[derive(Clone, Debug)]
pub struct EntityFluidInteraction {
    fluid_trackers: Vec<Tracker>,
    current_accumulators: Vec<(FluidTagKey, CurrentAccumulator)>,
}

impl EntityFluidInteraction {
    pub fn new(fluids_with_current: &[FluidTagKey]) -> EntityFluidInteraction {
        EntityFluidInteraction {
            fluid_trackers: Vec::new(),
            current_accumulators: fluids_with_current
                .iter()
                .map(|fluid| (*fluid, CurrentAccumulator::new()))
                .collect(),
        }
    }

    /// `update(Entity, boolean)`: the fluids in the entity's fluid interaction box, their heights
    /// over its feet, whether its eyes are under one, and (unless `ignore_current`) their flows.
    /// Whether it's in any fluid.
    pub fn update(
        &mut self,
        entity: &(impl EntityClass + ?Sized),
        level: &dyn Level,
        ignore_current: bool,
    ) -> bool {
        self.fluid_trackers.retain_mut(|tracker| !tracker.reset());
        for (_, current) in &mut self.current_accumulators {
            current.reset();
        }
        let Some(r#box) = entity.get_fluid_interaction_box() else {
            return false;
        };

        let x0 = Mth::floor_f64(r#box.min_x);
        let y0 = Mth::floor_f64(r#box.min_y);
        let z0 = Mth::floor_f64(r#box.min_z);
        let x1 = Mth::ceil_f64(r#box.max_x) - 1;
        let y1 = Mth::ceil_f64(r#box.max_y) - 1;
        let z1 = Mth::ceil_f64(r#box.max_z) - 1;
        if !has_fluid_and_loaded(level, x0 - 1, y0, z0 - 1, x1 + 1, y1, z1 + 1) {
            return false;
        }

        let entity_y = entity.entity().get_bounding_box().min_y;
        let eye_block_x = entity.entity().block_position().get_x();
        let eye_y = entity.entity().get_eye_y();
        let eye_block_z = entity.entity().block_position().get_z();
        let mut last_fluid_type: Option<Fluids> = None;
        let mut tracker: Option<usize> = None;
        let mut current: Option<usize> = None;

        for x in x0..=x1 {
            for y in y0..=y1 {
                for z in z0..=z1 {
                    let mutable_pos = BlockPos::new(x, y, z);
                    let fluid_state = level.get_fluid_state(&mutable_pos);
                    if !fluid_state.is_empty() {
                        let fluid_bottom = f64::from(mutable_pos.get_y());
                        let fluid_top =
                            fluid_bottom + f64::from(fluid_state.get_height(level, &mutable_pos));
                        if !(fluid_top < r#box.min_y) {
                            let fluid_type = fluid_state.get_type();
                            if last_fluid_type != Some(fluid_type) {
                                last_fluid_type = Some(fluid_type);
                                tracker = Some(self.get_or_create_tracker_for(fluid_type));
                                if !ignore_current {
                                    current = self.get_current_accumulator_for(fluid_type);
                                }
                            }
                            let tracker = &mut self.fluid_trackers
                                [tracker.expect("a fluid's tracker is found first")];

                            if x == eye_block_x && z == eye_block_z && eye_y >= fluid_bottom {
                                let fluid_top_for_camera = fluid_bottom
                                    + f64::from(
                                        fluid_state.get_height_for_camera(level, &mutable_pos),
                                    );
                                if eye_y <= fluid_top_for_camera {
                                    tracker.eyes_inside = true;
                                }
                            }

                            tracker.height = Math::max_f64(fluid_top - entity_y, tracker.height);
                            if let Some(current) = current {
                                let tracker_height = tracker.height;
                                let current = &mut self.current_accumulators[current].1;
                                let mut flow = fluid_state.get_flow(level, &mutable_pos);
                                current.height = Math::max_f64(tracker_height, current.height);
                                if current.height < 0.4 {
                                    flow = flow.scale(current.height);
                                }

                                current.accumulate(&flow);
                            }
                        }
                    }
                }
            }
        }

        last_fluid_type.is_some()
    }

    fn get_or_create_tracker_for(&mut self, fluid_type: Fluids) -> usize {
        if let Some(index) = self
            .fluid_trackers
            .iter()
            .position(|tracker| tracker.fluid_type == fluid_type)
        {
            return index;
        }

        self.fluid_trackers.push(Tracker::new(fluid_type));
        self.fluid_trackers.len() - 1
    }

    fn get_current_accumulator_for(&self, fluid: Fluids) -> Option<usize> {
        self.current_accumulators
            .iter()
            .position(|(tag, _)| fluid.is_tag(*tag))
    }

    /// `applyCurrentTo`: pushes the entity with the tag's flows, scaled.
    pub fn apply_current_to(
        &self,
        fluid: FluidTagKey,
        entity: &mut (impl EntityClass + ?Sized),
        scale: f64,
    ) {
        if let Some((_, current)) = self
            .current_accumulators
            .iter()
            .find(|(tag, _)| *tag == fluid)
        {
            current.apply_to(entity, scale);
        }
    }

    /// `getFluidHeight(TagKey)`: how far above its feet the tag's deepest fluid goes.
    pub fn get_fluid_height(&self, fluid: FluidTagKey) -> f64 {
        let mut height = 0.0;

        for tracker in &self.fluid_trackers {
            if tracker.height > height && tracker.fluid_type.is_tag(fluid) {
                height = tracker.height;
            }
        }

        height
    }

    pub fn is_in_fluid(&self, fluid: FluidTagKey) -> bool {
        self.get_fluid_height(fluid) > 0.0
    }

    pub fn is_eye_in_fluid(&self, fluid: FluidTagKey) -> bool {
        self.fluid_trackers
            .iter()
            .any(|tracker| tracker.eyes_inside && tracker.fluid_type.is_tag(fluid))
    }
}

/// `hasFluidAndLoaded`: whether every chunk from `x0`, `z0` to `x1`, `z1` is loaded and one of
/// their sections from `y0` to `y1` holds a fluid.
fn has_fluid_and_loaded(
    level: &dyn Level,
    x0: i32,
    y0: i32,
    z0: i32,
    x1: i32,
    y1: i32,
    z1: i32,
) -> bool {
    let section_x0 = SectionPos::block_to_section_coord(x0);
    let section_y0 = SectionPos::block_to_section_coord(y0);
    let section_z0 = SectionPos::block_to_section_coord(z0);
    let section_x1 = SectionPos::block_to_section_coord(x1);
    let section_y1 = SectionPos::block_to_section_coord(y1);
    let section_z1 = SectionPos::block_to_section_coord(z1);
    let mut has_fluid = false;

    for chunk_z in section_z0..=section_z1 {
        for chunk_x in section_x0..=section_x1 {
            match level.chunk_has_fluid(chunk_x, chunk_z, section_y0, section_y1) {
                None => return false,
                Some(fluid) => has_fluid |= fluid,
            }
        }
    }

    has_fluid
}

/// `CurrentAccumulator`: the flows of a tag's fluids the entity is in.
#[derive(Clone, Debug)]
struct CurrentAccumulator {
    height: f64,
    accumulated_current: Vec3,
    current_count: i32,
}

impl CurrentAccumulator {
    fn new() -> CurrentAccumulator {
        CurrentAccumulator {
            height: 0.0,
            accumulated_current: Vec3::ZERO,
            current_count: 0,
        }
    }

    fn reset(&mut self) {
        self.height = 0.0;
        self.accumulated_current = Vec3::ZERO;
        self.current_count = 0;
    }

    fn accumulate(&mut self, flow: &Vec3) {
        self.accumulated_current = self.accumulated_current.add(flow);
        self.current_count += 1;
    }

    /// `applyTo`: a player is pushed by the flows' mean, other entities by their direction, scaled;
    /// an entity nearly at rest at least by 0.0045.
    fn apply_to(&self, entity: &mut (impl EntityClass + ?Sized), scale: f64) {
        if self.current_count != 0
            && !(self.accumulated_current.length_sqr() < f64::from(1.0E-5_f32))
        {
            let mut impulse = if !entity.is_player() {
                self.accumulated_current.normalize()
            } else {
                self.accumulated_current
                    .scale(1.0 / f64::from(self.current_count))
            };

            let old_movement = entity.entity().get_delta_movement();
            impulse = impulse.scale(scale);
            if old_movement.x.abs() < 0.003
                && old_movement.z.abs() < 0.003
                && impulse.length() < 0.0045000000000000005
            {
                impulse = impulse.normalize().scale(0.0045000000000000005);
            }

            entity.entity_mut().add_delta_movement(&impulse);
        }
    }
}

/// `Tracker`: how high a fluid type goes above the entity's feet, and whether its eyes are in it.
#[derive(Clone, Debug)]
struct Tracker {
    fluid_type: Fluids,
    height: f64,
    eyes_inside: bool,
}

impl Tracker {
    fn new(fluid_type: Fluids) -> Tracker {
        Tracker {
            fluid_type,
            height: 0.0,
            eyes_inside: false,
        }
    }

    /// `reset()`: true (dropped) when it found nothing last time, else cleared for this one.
    fn reset(&mut self) -> bool {
        if self.height == 0.0 && !self.eyes_inside {
            return true;
        }

        self.height = 0.0;
        self.eyes_inside = false;
        false
    }
}
