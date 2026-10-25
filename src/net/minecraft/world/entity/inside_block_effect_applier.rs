//! net/minecraft/world/entity/InsideBlockEffectApplier.java, with its `StepBasedCollector`.
//!
//! What a block an entity is inside hands its effects to (`entityInside`): the effect types it
//! asks for, and effects of its own to run before or after one of them, Java's
//! `Consumer<Entity>`s, which get the level Java's lambdas close over when they run. The collector
//! an entity checks its blocks with (`Entity.checkInsideBlocks`) groups them by step, the place of
//! the block along the movement: each step's effects in `InsideBlockEffectType`'s order, each
//! type once, its own effects around it, all of them applied once the movements are checked.
//! `NOOP` isn't ported: nothing ported asks for it.

use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::entity::inside_block_effect_type::InsideBlockEffectType;
use crate::net::minecraft::world::level::level::Level;

/// A block's own effect on the entity inside it, Java's `Consumer<Entity>`.
pub type Effect = Box<dyn FnOnce(&mut dyn EntityClass, &dyn Level)>;

pub trait InsideBlockEffectApplier {
    fn apply(&mut self, r#type: InsideBlockEffectType);

    fn run_before(&mut self, r#type: InsideBlockEffectType, effect: Effect);

    fn run_after(&mut self, r#type: InsideBlockEffectType, effect: Effect);
}

/// `StepBasedCollector`: the effects of each step, in order.
pub struct StepBasedCollector {
    /// `effectsInStep`, an `EnumSet`: whether each type is asked for in this step.
    effects_in_step: [bool; 5],
    before_effects_in_step: [Vec<Effect>; 5],
    after_effects_in_step: [Vec<Effect>; 5],
    final_effects: Vec<Effect>,
    last_step: i32,
}

impl StepBasedCollector {
    /// `APPLY_ORDER`.
    const APPLY_ORDER: [InsideBlockEffectType; 5] = InsideBlockEffectType::VALUES;
    /// `NO_STEP`.
    const NO_STEP: i32 = -1;

    pub fn new() -> StepBasedCollector {
        StepBasedCollector {
            effects_in_step: [false; 5],
            before_effects_in_step: std::array::from_fn(|_| Vec::new()),
            after_effects_in_step: std::array::from_fn(|_| Vec::new()),
            final_effects: Vec::new(),
            last_step: StepBasedCollector::NO_STEP,
        }
    }

    /// `advanceStep(int)`: a step of its own ends the last one.
    pub fn advance_step(&mut self, step: i32) {
        if self.last_step != step {
            self.last_step = step;
            self.flush_step();
        }
    }

    /// `applyAndClear(Entity)`: every step's effects, in order, while the entity is alive.
    pub fn apply_and_clear(&mut self, entity: &mut dyn EntityClass, level: &dyn Level) {
        self.flush_step();

        for effect in self.final_effects.drain(..) {
            if !entity.is_alive() {
                break;
            }

            effect(entity, level);
        }

        self.final_effects.clear();
        self.last_step = StepBasedCollector::NO_STEP;
    }

    fn flush_step(&mut self) {
        for r#type in StepBasedCollector::APPLY_ORDER {
            let index = r#type as usize;
            self.final_effects
                .append(&mut self.before_effects_in_step[index]);
            if std::mem::replace(&mut self.effects_in_step[index], false) {
                let effect = r#type.effect();
                self.final_effects
                    .push(Box::new(move |entity, _level| effect(entity)));
            }

            self.final_effects
                .append(&mut self.after_effects_in_step[index]);
        }
    }
}

impl Default for StepBasedCollector {
    fn default() -> StepBasedCollector {
        StepBasedCollector::new()
    }
}

impl InsideBlockEffectApplier for StepBasedCollector {
    fn apply(&mut self, r#type: InsideBlockEffectType) {
        self.effects_in_step[r#type as usize] = true;
    }

    fn run_before(&mut self, r#type: InsideBlockEffectType, effect: Effect) {
        self.before_effects_in_step[r#type as usize].push(effect);
    }

    fn run_after(&mut self, r#type: InsideBlockEffectType, effect: Effect) {
        self.after_effects_in_step[r#type as usize].push(effect);
    }
}
