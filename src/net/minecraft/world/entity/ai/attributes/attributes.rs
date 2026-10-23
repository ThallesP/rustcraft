//! net/minecraft/world/entity/ai/attributes/Attributes.java
//!
//! Only the attributes an entity's movement reads (and the player's health and armor, which the
//! HUD shows): their defaults and ranges as `Attributes` registers them. The others come with what reads them.
//!
//! Java's fields are the attributes' holders, which `BuiltInRegistries.ATTRIBUTE` numbers in the
//! order they register: the id the server sends an attribute by (`Attribute.STREAM_CODEC`). So
//! every field's holder is here (`holder`), and `by_holder` finds a ported attribute from its
//! holder. `bootstrap` only makes Java's static initializer run, so it isn't ported.

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::entity::ai::attributes::attribute::Attribute;

/// Each field and the name it registers its attribute under, in the order Java registers them.
pub(crate) const REGISTERED: [(&str, &str); 40] = [
    ("AIR_DRAG_MODIFIER", "air_drag_modifier"),
    ("ARMOR", "armor"),
    ("ARMOR_TOUGHNESS", "armor_toughness"),
    ("ATTACK_DAMAGE", "attack_damage"),
    ("ATTACK_KNOCKBACK", "attack_knockback"),
    ("ATTACK_SPEED", "attack_speed"),
    ("BELOW_NAME_DISTANCE", "below_name_distance"),
    ("BLOCK_BREAK_SPEED", "block_break_speed"),
    ("BLOCK_INTERACTION_RANGE", "block_interaction_range"),
    ("BOUNCINESS", "bounciness"),
    ("BURNING_TIME", "burning_time"),
    ("CAMERA_DISTANCE", "camera_distance"),
    (
        "EXPLOSION_KNOCKBACK_RESISTANCE",
        "explosion_knockback_resistance",
    ),
    ("ENTITY_INTERACTION_RANGE", "entity_interaction_range"),
    ("FALL_DAMAGE_MULTIPLIER", "fall_damage_multiplier"),
    ("FLYING_SPEED", "flying_speed"),
    ("FOLLOW_RANGE", "follow_range"),
    ("FRICTION_MODIFIER", "friction_modifier"),
    ("GRAVITY", "gravity"),
    ("JUMP_STRENGTH", "jump_strength"),
    ("KNOCKBACK_RESISTANCE", "knockback_resistance"),
    ("LUCK", "luck"),
    ("MAX_ABSORPTION", "max_absorption"),
    ("MAX_HEALTH", "max_health"),
    ("MINING_EFFICIENCY", "mining_efficiency"),
    ("MOVEMENT_EFFICIENCY", "movement_efficiency"),
    ("MOVEMENT_SPEED", "movement_speed"),
    ("NAME_TAG_DISTANCE", "name_tag_distance"),
    ("OXYGEN_BONUS", "oxygen_bonus"),
    ("SAFE_FALL_DISTANCE", "safe_fall_distance"),
    ("SCALE", "scale"),
    ("SNEAKING_SPEED", "sneaking_speed"),
    ("SPAWN_REINFORCEMENTS_CHANCE", "spawn_reinforcements"),
    ("STEP_HEIGHT", "step_height"),
    ("SUBMERGED_MINING_SPEED", "submerged_mining_speed"),
    ("SWEEPING_DAMAGE_RATIO", "sweeping_damage_ratio"),
    ("TEMPT_RANGE", "tempt_range"),
    ("WATER_MOVEMENT_EFFICIENCY", "water_movement_efficiency"),
    ("WAYPOINT_TRANSMIT_RANGE", "waypoint_transmit_range"),
    ("WAYPOINT_RECEIVE_RANGE", "waypoint_receive_range"),
];

/// The entities' attributes, which the server sends values of (`update_attributes`).
pub struct Attributes;

impl Attributes {
    pub const DEFAULT_ATTACK_SPEED: f64 = 4.0;
    pub const AIR_DRAG_MODIFIER: &'static Attribute = &AIR_DRAG_MODIFIER;
    pub const ARMOR: &'static Attribute = &ARMOR;
    pub const BOUNCINESS: &'static Attribute = &BOUNCINESS;
    pub const FRICTION_MODIFIER: &'static Attribute = &FRICTION_MODIFIER;
    pub const GRAVITY: &'static Attribute = &GRAVITY;
    pub const JUMP_STRENGTH: &'static Attribute = &JUMP_STRENGTH;
    pub const MAX_HEALTH: &'static Attribute = &MAX_HEALTH;
    pub const MOVEMENT_EFFICIENCY: &'static Attribute = &MOVEMENT_EFFICIENCY;
    pub const MOVEMENT_SPEED: &'static Attribute = &MOVEMENT_SPEED;
    pub const SCALE: &'static Attribute = &SCALE;
    pub const SNEAKING_SPEED: &'static Attribute = &SNEAKING_SPEED;
    pub const STEP_HEIGHT: &'static Attribute = &STEP_HEIGHT;
    pub const WATER_MOVEMENT_EFFICIENCY: &'static Attribute = &WATER_MOVEMENT_EFFICIENCY;

    /// `register(name, attribute)`'s holder: the attribute's key in the registry.
    pub fn holder(name: &str) -> Holder {
        Holder::Reference(ResourceKey::create(
            Registries::ATTRIBUTE,
            Identifier::with_default_namespace(name),
        ))
    }

    /// The ported attribute a holder of the registry is, or none for one that isn't ported (or
    /// isn't an attribute's).
    pub fn by_holder(holder: &Holder) -> Option<&'static Attribute> {
        let key = holder.unwrap_key()?;
        if key.registry() != Registries::ATTRIBUTE.identifier()
            || key.identifier().get_namespace() != Identifier::DEFAULT_NAMESPACE
        {
            return None;
        }
        let attribute = match key.identifier().get_path() {
            "air_drag_modifier" => Attributes::AIR_DRAG_MODIFIER,
            "armor" => Attributes::ARMOR,
            "bounciness" => Attributes::BOUNCINESS,
            "friction_modifier" => Attributes::FRICTION_MODIFIER,
            "gravity" => Attributes::GRAVITY,
            "jump_strength" => Attributes::JUMP_STRENGTH,
            "max_health" => Attributes::MAX_HEALTH,
            "movement_efficiency" => Attributes::MOVEMENT_EFFICIENCY,
            "movement_speed" => Attributes::MOVEMENT_SPEED,
            "scale" => Attributes::SCALE,
            "sneaking_speed" => Attributes::SNEAKING_SPEED,
            "step_height" => Attributes::STEP_HEIGHT,
            "water_movement_efficiency" => Attributes::WATER_MOVEMENT_EFFICIENCY,
            _ => return None,
        };
        Some(attribute)
    }
}

static AIR_DRAG_MODIFIER: Attribute =
    Attribute::ranged("attribute.name.air_drag_modifier", 1.0, 0.0, 2048.0, true);
static ARMOR: Attribute = Attribute::ranged("attribute.name.armor", 0.0, 0.0, 30.0, true);
static BOUNCINESS: Attribute = Attribute::ranged("attribute.name.bounciness", 0.0, 0.0, 1.0, true);
static FRICTION_MODIFIER: Attribute =
    Attribute::ranged("attribute.name.friction_modifier", 1.0, 0.0, 2048.0, true);
static GRAVITY: Attribute = Attribute::ranged("attribute.name.gravity", 0.08, -1.0, 1.0, true);
// 0.42F, a float widened to the constructor's double.
static JUMP_STRENGTH: Attribute = Attribute::ranged(
    "attribute.name.jump_strength",
    0.41999998688697815,
    0.0,
    32.0,
    true,
);
static MAX_HEALTH: Attribute =
    Attribute::ranged("attribute.name.max_health", 20.0, 1.0, 1024.0, true);
static MOVEMENT_EFFICIENCY: Attribute =
    Attribute::ranged("attribute.name.movement_efficiency", 0.0, 0.0, 1.0, true);
static MOVEMENT_SPEED: Attribute =
    Attribute::ranged("attribute.name.movement_speed", 0.7, 0.0, 1024.0, true);
static SCALE: Attribute = Attribute::ranged("attribute.name.scale", 1.0, 0.0625, 16.0, true);
static SNEAKING_SPEED: Attribute =
    Attribute::ranged("attribute.name.sneaking_speed", 0.3, 0.0, 1.0, true);
static STEP_HEIGHT: Attribute =
    Attribute::ranged("attribute.name.step_height", 0.6, 0.0, 10.0, true);
static WATER_MOVEMENT_EFFICIENCY: Attribute = Attribute::ranged(
    "attribute.name.water_movement_efficiency",
    0.0,
    0.0,
    1.0,
    true,
);

#[cfg(test)]
mod tests {
    use super::{Attributes, REGISTERED};
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.ai.attributes.Attributes");
        golden.field("DEFAULT_ATTACK_SPEED", Attributes::DEFAULT_ATTACK_SPEED);
        for (field, name) in REGISTERED {
            golden.field(field, Attributes::holder(name));
        }
        golden.pending(
            "bootstrap(Lnet/minecraft/core/Registry;)Lnet/minecraft/core/Holder;",
            "only runs Java's static initializer",
        );
        golden.finish();
    }

    /// Each ported attribute is the one its holder names: its description id is its name's.
    #[test]
    fn finds_the_ported_attributes_by_holder() {
        let mut ported = 0;
        for (_, name) in REGISTERED {
            if let Some(attribute) = Attributes::by_holder(&Attributes::holder(name)) {
                assert_eq!(
                    attribute.get_description_id(),
                    format!("attribute.name.{name}")
                );
                ported += 1;
            }
        }
        assert_eq!(ported, 13);
    }
}
