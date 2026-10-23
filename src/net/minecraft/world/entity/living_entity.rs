//! net/minecraft/world/entity/LivingEntity.java
//!
//! What a living entity adds to an entity's movement: its attributes, `tick` and `aiStep` (the
//! small movement cut-off, the input, jumping, travelling), `travel` and `travelInAir` with the
//! block's friction, `jumpFromGround`, sprinting's speed modifier, and its overrides of `Entity`'s
//! gravity, air drag, step height, speed factor and size. `LivingEntityClass` holds the methods
//! its subclasses override (PORTING.md, "Entities").
//!
//! Not ported yet: health beyond its value, damage and death, effects (`hasEffect` is never true:
//! no jump boost, slow falling or levitation), equipment and using items, riding (`travelRidden`),
//! swimming and travelling in fluids (`travelInFluid`, which needs fluid states, panics), gliding
//! (`travelFallFlying`, which panics), pushing other entities, the body's and head's rotation and
//! the walk animation, which only rendering reads, and `applyEffectsFromBlocks` (see `Entity`).

// Java's negated comparisons are kept as they are: they differ from the plain ones for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use crate::java::lang::math::Math;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::ai::attributes::attribute::Attribute;
use crate::net::minecraft::world::entity::ai::attributes::attribute_map::AttributeMap;
use crate::net::minecraft::world::entity::ai::attributes::attribute_modifier::{
    AttributeModifier, Operation,
};
use crate::net::minecraft::world::entity::ai::attributes::attributes::Attributes;
use crate::net::minecraft::world::entity::entity::{self, Entity, EntityClass};
use crate::net::minecraft::world::entity::entity_dimensions::EntityDimensions;
use crate::net::minecraft::world::entity::mover_type::MoverType;
use crate::net::minecraft::world::entity::pose::Pose;
use crate::net::minecraft::world::level::block::trap_door_block;
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// What a living entity has besides an entity's.
#[derive(Clone, Debug)]
pub struct LivingEntity {
    entity: Entity,
    pub attributes: AttributeMap,
    speed: f32,
    pub jumping: bool,
    pub xxa: f32,
    pub yya: f32,
    pub zza: f32,
    no_jump_delay: i32,
    health: f32,
    dead: bool,
    pub discard_friction: bool,
}

impl std::ops::Deref for LivingEntity {
    type Target = Entity;

    fn deref(&self) -> &Entity {
        &self.entity
    }
}

impl std::ops::DerefMut for LivingEntity {
    fn deref_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

/// `SPRINTING_MODIFIER_ID`.
fn sprinting_modifier_id() -> Identifier {
    Identifier::with_default_namespace("sprinting")
}

/// `SPEED_MODIFIER_SPRINTING`: 30% more speed, of the total.
fn speed_modifier_sprinting() -> AttributeModifier {
    AttributeModifier::new(
        sprinting_modifier_id(),
        f64::from(0.3_f32),
        Operation::ADD_MULTIPLIED_TOTAL,
    )
}

impl LivingEntity {
    /// `SLEEPING_DIMENSIONS`.
    pub fn sleeping_dimensions() -> EntityDimensions {
        EntityDimensions::fixed_size(0.2, 0.2).with_eye_height(0.2)
    }

    /// A living entity of a type with these dimensions and attributes, at full health.
    pub fn new(dimensions: EntityDimensions, attributes: AttributeMap) -> LivingEntity {
        let health = attributes.get_value(Attributes::MAX_HEALTH) as f32;
        LivingEntity {
            entity: Entity::new(dimensions),
            attributes,
            speed: 0.0,
            jumping: false,
            xxa: 0.0,
            yya: 0.0,
            zza: 0.0,
            no_jump_delay: 0,
            health,
            dead: false,
            discard_friction: false,
        }
    }

    pub fn get_attribute_value(&self, attribute: &'static Attribute) -> f64 {
        self.attributes.get_value(attribute)
    }

    pub fn get_health(&self) -> f32 {
        self.health
    }

    pub fn set_health(&mut self, health: f32) {
        let max_health = self.get_max_health();
        self.health = Mth::clamp_f32(health, 0.0, max_health);
    }

    pub fn get_max_health(&self) -> f32 {
        self.get_attribute_value(Attributes::MAX_HEALTH) as f32
    }

    pub fn is_dead_or_dying(&self) -> bool {
        self.health <= 0.0 || self.dead
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn get_scale(&self) -> f32 {
        self.get_attribute_value(Attributes::SCALE) as f32
    }

    pub fn no_jump_delay(&self) -> i32 {
        self.no_jump_delay
    }
}

/// `computeModifiedFriction`: the friction an attribute's modifier makes, between 0 and 1.
pub fn compute_modified_friction(friction: f32, modifier: f32) -> f32 {
    Mth::clamp_f32(1.0 - (1.0 - friction) * modifier, 0.0, 1.0)
}

/// Java's `LivingEntity` methods that subclasses override or that call overridden ones. The
/// defaults are `LivingEntity`'s.
pub trait LivingEntityClass: EntityClass {
    fn living_entity(&self) -> &LivingEntity;

    fn living_entity_mut(&mut self) -> &mut LivingEntity;

    fn get_speed(&self) -> f32 {
        self.living_entity().speed
    }

    /// No player rides it, so it's the default 0.02.
    fn get_flying_speed(&self) -> f32 {
        0.02
    }

    fn is_affected_by_fluids(&self) -> bool {
        true
    }

    fn is_immobile(&self) -> bool {
        self.living_entity().is_dead_or_dying()
    }

    fn is_spectator(&self) -> bool {
        false
    }

    fn apply_input(&mut self) {
        let living = self.living_entity_mut();
        living.xxa *= 0.98;
        living.zza *= 0.98;
    }

    fn on_climbable(&mut self, level: &dyn Level) -> bool
    where
        Self: Sized,
    {
        on_climbable(self, level)
    }

    fn travel(&mut self, level: &dyn Level, input: &Vec3)
    where
        Self: Sized,
    {
        travel(self, level, input);
    }

    fn ai_step(&mut self, level: &dyn Level)
    where
        Self: Sized,
    {
        ai_step(self, level);
    }
}

/// `LivingEntity.getDimensions(Pose)`: the pose's size, scaled.
pub fn get_dimensions(
    this: &(impl LivingEntityClass + ?Sized),
    pose: Pose,
    default_dimensions: EntityDimensions,
) -> EntityDimensions {
    if pose == Pose::SLEEPING {
        LivingEntity::sleeping_dimensions()
    } else {
        default_dimensions.scale(this.living_entity().get_scale())
    }
}

/// `LivingEntity.setSprinting`: the flag, and the speed's sprinting modifier on or off.
pub fn set_sprinting(this: &mut (impl LivingEntityClass + ?Sized), is_sprinting: bool) {
    this.entity_mut()
        .set_shared_flag(Entity::FLAG_SPRINTING, is_sprinting);
    let speed = this
        .living_entity_mut()
        .attributes
        .get_instance_mut(Attributes::MOVEMENT_SPEED)
        .expect("a living entity has a movement speed");
    speed.remove_modifier(&sprinting_modifier_id());
    if is_sprinting {
        speed.add_transient_modifier(speed_modifier_sprinting());
    }
}

pub fn get_default_gravity(this: &(impl LivingEntityClass + ?Sized)) -> f64 {
    this.living_entity()
        .get_attribute_value(Attributes::GRAVITY)
}

/// `LivingEntity.getEffectiveGravity`: slow falling (an effect, not ported) would cap it.
pub fn get_effective_gravity(this: &(impl LivingEntityClass + ?Sized)) -> f64 {
    Entity::get_gravity(this)
}

/// `LivingEntity.getAirDrag`.
pub fn get_air_drag(this: &(impl LivingEntityClass + ?Sized)) -> f32 {
    compute_modified_friction(
        0.98,
        this.living_entity()
            .get_attribute_value(Attributes::AIR_DRAG_MODIFIER) as f32,
    )
}

pub fn get_entity_bounciness(this: &(impl LivingEntityClass + ?Sized)) -> f64 {
    this.living_entity()
        .get_attribute_value(Attributes::BOUNCINESS)
}

/// `LivingEntity.maxUpStep`: no player rides it, so its own step height.
pub fn max_up_step(this: &(impl LivingEntityClass + ?Sized)) -> f32 {
    this.living_entity()
        .get_attribute_value(Attributes::STEP_HEIGHT) as f32
}

/// `LivingEntity.getBlockSpeedFactor`: the block's, less as movement efficiency grows.
pub fn get_block_speed_factor(
    this: &mut (impl LivingEntityClass + Sized),
    level: &dyn Level,
) -> f32 {
    let efficiency = this
        .living_entity()
        .get_attribute_value(Attributes::MOVEMENT_EFFICIENCY) as f32;
    Mth::lerp_f32(efficiency, entity::get_block_speed_factor(this, level), 1.0)
}

/// `LivingEntity.checkFallDamage`: the fluids again unless in water, then `Entity`'s. The
/// server's fall particles aren't the client's.
pub fn check_fall_damage(
    this: &mut (impl LivingEntityClass + Sized),
    level: &dyn Level,
    ya: f64,
    on_ground: bool,
    on_state: crate::net::minecraft::world::level::block::state::block_state::BlockState,
    pos: crate::net::minecraft::core::block_pos::BlockPos,
) {
    if !this.entity().is_in_water() {
        this.update_fluid_interaction(level);
    }
    entity::check_fall_damage(this, level, ya, on_ground, on_state, pos);
}

/// `LivingEntity.onClimbable`: in a climbable block. A trapdoor used as a ladder isn't ported.
pub fn on_climbable(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level) -> bool {
    if this.is_spectator() {
        return false;
    }
    let state = this.entity_mut().get_in_block_state(level);
    if this.entity().is_fall_flying() && state.is_tag(BlockTags::CAN_GLIDE_THROUGH) {
        false
    } else if state.is_tag(BlockTags::CLIMBABLE) {
        true
    } else {
        assert!(
            !state.get_block().is_instance_of(&trap_door_block::CLASS),
            "a trapdoor as a ladder isn't ported"
        );
        false
    }
}

/// `LivingEntity.baseTick()`'s part that reaches movement: `Entity`'s.
pub fn base_tick(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level) {
    entity::base_tick(this, level);
}

/// `LivingEntity.tick()`: the base tick, then `aiStep`, then the attributes that changed.
pub fn tick(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level) {
    base_tick(this, level);
    this.ai_step(level);

    let entity = this.entity_mut();
    while entity.get_y_rot() - entity.y_rot_o < -180.0 {
        entity.y_rot_o -= 360.0;
    }
    while entity.get_y_rot() - entity.y_rot_o >= 180.0 {
        entity.y_rot_o += 360.0;
    }
    while entity.get_x_rot() - entity.x_rot_o < -180.0 {
        entity.x_rot_o -= 360.0;
    }
    while entity.get_x_rot() - entity.x_rot_o >= 180.0 {
        entity.x_rot_o += 360.0;
    }

    refresh_dirty_attributes(this);
}

/// `refreshDirtyAttributes`: a new scale resizes the entity, and a lower maximum health caps its
/// health.
fn refresh_dirty_attributes(this: &mut (impl LivingEntityClass + Sized)) {
    for attribute in this
        .living_entity_mut()
        .attributes
        .take_attributes_to_update()
    {
        if Attribute::same(attribute, Attributes::MAX_HEALTH) {
            let current_max_health = this.living_entity().get_max_health();
            if this.living_entity().get_health() > current_max_health {
                this.living_entity_mut().set_health(current_max_health);
            }
        } else if Attribute::same(attribute, Attributes::SCALE) {
            entity::refresh_dimensions(this);
        }
    }
}

/// `LivingEntity.aiStep()`: the movement too small to keep is dropped, the input applied, a jump
/// made, and the entity travels.
pub fn ai_step(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level) {
    if this.living_entity().no_jump_delay > 0 {
        this.living_entity_mut().no_jump_delay -= 1;
    }

    let movement = this.entity().get_delta_movement();
    let mut dx = movement.x;
    let mut dy = movement.y;
    let mut dz = movement.z;
    // `is(EntityTypes.PLAYER)`: only players are ported.
    if movement.horizontal_distance_sqr() < 9.0E-6 {
        dx = 0.0;
        dz = 0.0;
    }

    if movement.y.abs() < 0.003 {
        dy = 0.0;
    }

    this.entity_mut().set_delta_movement_xyz(dx, dy, dz);
    this.apply_input();
    if this.is_immobile() {
        let living = this.living_entity_mut();
        living.jumping = false;
        living.xxa = 0.0;
        living.zza = 0.0;
    }

    if this.living_entity().jumping && this.is_affected_by_fluids() {
        let fluid_height = this.entity().get_fluid_height();
        let in_water_and_has_fluid_height = this.entity().is_in_water() && fluid_height > 0.0;
        let fluid_jump_threshold = this.entity().get_fluid_jump_threshold();
        if !in_water_and_has_fluid_height
            || this.entity().on_ground() && !(fluid_height > fluid_jump_threshold)
        {
            assert!(!this.entity().is_in_lava(), "jumping in lava isn't ported");
            if (this.entity().on_ground()
                || in_water_and_has_fluid_height && fluid_height <= fluid_jump_threshold)
                && this.living_entity().no_jump_delay == 0
            {
                jump_from_ground(this, level);
                this.living_entity_mut().no_jump_delay = 10;
            }
        } else {
            panic!("jumping in water isn't ported");
        }
    } else {
        this.living_entity_mut().no_jump_delay = 0;
    }

    assert!(!this.entity().is_fall_flying(), "gliding isn't ported");
    let living = this.living_entity();
    let input = Vec3::new(
        f64::from(living.xxa),
        f64::from(living.yya),
        f64::from(living.zza),
    );
    // canSimulateMovement and isEffectiveAi: the local player's always are.
    this.travel(level, &input);
}

/// `LivingEntity.getJumpPower(float)`: the jump strength, times the block's jump factor.
pub fn get_jump_power(
    this: &(impl LivingEntityClass + ?Sized),
    level: &dyn Level,
    multiplier: f32,
) -> f32 {
    this.living_entity()
        .get_attribute_value(Attributes::JUMP_STRENGTH) as f32
        * multiplier
        * this.entity().get_block_jump_factor(level)
        + 0.0
}

/// `jumpFromGround()`: up by the jump power at least, and forward a little when sprinting.
pub fn jump_from_ground(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level) {
    let jump_power = get_jump_power(this, level, 1.0);
    if !(jump_power <= 1.0E-5) {
        let movement = this.entity().get_delta_movement();
        this.entity_mut().set_delta_movement_xyz(
            movement.x,
            Math::max_f64(f64::from(jump_power), movement.y),
            movement.z,
        );
        if this.is_sprinting() {
            let angle = this.entity().get_y_rot() * (std::f64::consts::PI / 180.0) as f32;
            this.entity_mut().add_delta_movement(&Vec3::new(
                f64::from(-Mth::sin(f64::from(angle))) * 0.2,
                0.0,
                f64::from(Mth::cos(f64::from(angle))) * 0.2,
            ));
        }
    }
}

/// `LivingEntity.travel(Vec3)`: in the air, fluids and gliding not being ported.
pub fn travel(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level, input: &Vec3) {
    assert!(
        !(this.entity().is_in_liquid() && this.is_affected_by_fluids()),
        "travelling in fluids isn't ported"
    );
    assert!(!this.entity().is_fall_flying(), "gliding isn't ported");
    travel_in_air(this, level, input);
}

/// Moves with the block's friction on the ground (none in the air), then gravity, then the air's
/// and the block's drag.
fn travel_in_air(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level, input: &Vec3) {
    let pos_below = this
        .entity()
        .get_block_pos_below_that_affects_my_movement(level);
    let block_friction = if this.entity().on_ground() {
        compute_modified_friction(
            level.get_block_state(&pos_below).get_block().get_friction(),
            this.living_entity()
                .get_attribute_value(Attributes::FRICTION_MODIFIER) as f32,
        )
    } else {
        1.0
    };
    let movement =
        handle_relative_friction_and_calculate_movement(this, level, input, block_friction);
    let mut movement_y = movement.y;
    if !level.is_client_side() || level.has_chunk_at(&pos_below) {
        movement_y -= this.get_effective_gravity();
    } else if this.entity().get_y() > f64::from(level.get_min_y()) {
        movement_y = -0.1;
    } else {
        movement_y = 0.0;
    }

    if this.living_entity().discard_friction {
        this.entity_mut()
            .set_delta_movement_xyz(movement.x, movement_y, movement.z);
    } else {
        let entity_air_drag_modifier =
            this.living_entity()
                .get_attribute_value(Attributes::AIR_DRAG_MODIFIER) as f32;
        let air_drag = compute_modified_friction(0.91, entity_air_drag_modifier);
        let friction = block_friction * air_drag;
        let vertical_friction = compute_modified_friction(0.98, entity_air_drag_modifier);
        this.entity_mut().set_delta_movement_xyz(
            movement.x * f64::from(friction),
            movement_y * f64::from(vertical_friction),
            movement.z * f64::from(friction),
        );
    }
}

/// `getFrictionInfluencedSpeed`: on the ground the speed, more on slippery blocks; in the air
/// the flying speed.
pub fn get_friction_influenced_speed(
    this: &(impl LivingEntityClass + ?Sized),
    block_friction: f32,
) -> f32 {
    if this.entity().on_ground() {
        if f64::from(block_friction) > 0.6 {
            this.get_speed() * (0.21600002 / (block_friction * block_friction * block_friction))
        } else {
            this.get_speed()
        }
    } else {
        this.get_flying_speed()
    }
}

fn handle_on_climbable(
    this: &mut (impl LivingEntityClass + Sized),
    level: &dyn Level,
    delta: Vec3,
) -> Vec3 {
    if this.on_climbable(level) {
        panic!("climbing isn't ported");
    }
    delta
}

fn handle_relative_friction_and_calculate_movement(
    this: &mut (impl LivingEntityClass + Sized),
    level: &dyn Level,
    input: &Vec3,
    friction: f32,
) -> Vec3 {
    let speed = get_friction_influenced_speed(this, friction);
    entity::move_relative(this, speed, input);
    let delta = this.entity().get_delta_movement();
    let delta = handle_on_climbable(this, level, delta);
    this.entity_mut().set_delta_movement(delta);
    let delta = this.entity().get_delta_movement();
    this.r#move(level, MoverType::SELF, delta);
    let movement = this.entity().get_delta_movement();
    if (this.entity().horizontal_collision || this.living_entity().jumping)
        && (this.on_climbable(level)
            || this.entity().was_in_powder_snow && panic!("powder snow isn't ported"))
    {
        return Vec3::new(movement.x, 0.2, movement.z);
    }
    movement
}

#[cfg(test)]
mod tests {
    use super::compute_modified_friction;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.LivingEntity");
        golden.check("computeModifiedFriction(FF)F", compute_modified_friction);
        golden.finish();
    }
}
