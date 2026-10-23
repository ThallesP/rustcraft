//! net/minecraft/world/entity/ai/attributes/Attributes.java
//!
//! Only the attributes an entity's movement reads (and the player's health): their defaults and
//! ranges as `Attributes` registers them. The others come with what reads them.

use crate::net::minecraft::world::entity::ai::attributes::attribute::Attribute;

pub struct Attributes;

impl Attributes {
    pub const AIR_DRAG_MODIFIER: &'static Attribute = &AIR_DRAG_MODIFIER;
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
}

static AIR_DRAG_MODIFIER: Attribute =
    Attribute::ranged("attribute.name.air_drag_modifier", 1.0, 0.0, 2048.0, true);
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
