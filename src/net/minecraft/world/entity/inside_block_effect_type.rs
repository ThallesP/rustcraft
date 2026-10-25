//! net/minecraft/world/entity/InsideBlockEffectType.java
//!
//! The effects the blocks an entity is inside ask for (`InsideBlockEffectApplier.apply`), in the
//! order they're applied, each its effect on the entity: freezing in powder snow (`FREEZE`), the
//! freeze ending in lava (`CLEAR_FREEZE`), and the entity's fire (`FIRE_IGNITE`, `LAVA_IGNITE`,
//! `EXTINGUISH`). An entity's fire isn't ported (`remainingFireTicks`, which only its sounds read
//! on the client, and which the client's `baseTick` clears every tick), so those three change
//! nothing here.

use crate::net::minecraft::world::entity::entity::EntityClass;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InsideBlockEffectType {
    FREEZE,
    CLEAR_FREEZE,
    FIRE_IGNITE,
    LAVA_IGNITE,
    EXTINGUISH,
}

impl InsideBlockEffectType {
    pub const VALUES: [InsideBlockEffectType; 5] = [
        InsideBlockEffectType::FREEZE,
        InsideBlockEffectType::CLEAR_FREEZE,
        InsideBlockEffectType::FIRE_IGNITE,
        InsideBlockEffectType::LAVA_IGNITE,
        InsideBlockEffectType::EXTINGUISH,
    ];

    /// `effect()`: what it does to the entity.
    pub fn effect(self) -> fn(&mut dyn EntityClass) {
        match self {
            InsideBlockEffectType::FREEZE => |entity| {
                entity.entity_mut().set_is_in_powder_snow(true);
                if entity.can_freeze() {
                    let ticks_frozen = entity
                        .entity()
                        .get_ticks_required_to_freeze()
                        .min(entity.entity().get_ticks_frozen() + 1);
                    entity.entity_mut().set_ticks_frozen(ticks_frozen);
                }
            },
            InsideBlockEffectType::CLEAR_FREEZE => |entity| entity.entity_mut().clear_freeze(),
            // `BaseFireBlock::fireIgnite`, `Entity::lavaIgnite` and `Entity::clearFire`: the
            // entity's fire, which isn't ported.
            InsideBlockEffectType::FIRE_IGNITE
            | InsideBlockEffectType::LAVA_IGNITE
            | InsideBlockEffectType::EXTINGUISH => |_| {},
        }
    }
}
