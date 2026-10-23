//! net/minecraft/world/entity/LivingEntity.java
//!
//! What a living entity adds to an entity's movement: its attributes, `tick` and `aiStep` (the
//! small movement cut-off, the input, jumping on the ground and in fluids, travelling), `travel`,
//! `travelInAir` with the block's friction, `travelInFluid` (water and lava, getting out of them at
//! an edge), climbing (`onClimbable`, `handleOnClimbable`), `jumpFromGround`, sprinting's speed
//! modifier, and its overrides of `Entity`'s gravity, air drag, step height, speed factor and size.
//! `LivingEntityClass` holds the methods its subclasses override (PORTING.md, "Entities").
//!
//! Not ported yet: health beyond its value, damage and death, effects (`hasEffect` is never true:
//! no jump boost, slow falling, levitation or dolphin's grace), equipment and using items, riding
//! (`travelRidden`, `floatInLiquidWhileRidden`), gliding (`travelFallFlying`, which panics),
//! pushing other entities, the swim amount, the body's and head's rotation and the walk animation,
//! which only rendering reads, and `applyEffectsFromBlocks` (see `Entity`).

// Java's negated comparisons are kept as they are: they differ from the plain ones for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use crate::java::lang::math::Math;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::tags::fluid_tags::{FluidTagKey, FluidTags};
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::ai::attributes::attribute::Attribute;
use crate::net::minecraft::world::entity::ai::attributes::attribute_map::AttributeMap;
use crate::net::minecraft::world::entity::ai::attributes::attribute_modifier::{
    AttributeModifier, Operation,
};
use crate::net::minecraft::world::entity::ai::attributes::attributes::Attributes;
use crate::net::minecraft::world::entity::entity::{self, Entity, EntityClass};
use crate::net::minecraft::world::entity::entity_dimensions::EntityDimensions;
use crate::net::minecraft::world::entity::entity_type::EntityType;
use crate::net::minecraft::world::entity::mover_type::MoverType;
use crate::net::minecraft::world::entity::pose::Pose;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::ladder_block::LadderBlock;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::trap_door_block::{self, TrapDoorBlock};
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
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
    last_climbable_pos: Option<BlockPos>,
    pub hurt_time: i32,
    pub hurt_duration: i32,
    pub damage_cooldown_time: i32,
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

    /// A living entity of the type, with these dimensions and attributes, at full health.
    pub fn new(
        r#type: EntityType,
        dimensions: EntityDimensions,
        attributes: AttributeMap,
    ) -> LivingEntity {
        let health = attributes.get_value(Attributes::MAX_HEALTH) as f32;
        LivingEntity {
            entity: Entity::new(r#type, dimensions),
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
            last_climbable_pos: None,
            hurt_time: 0,
            hurt_duration: 0,
            damage_cooldown_time: 0,
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

    /// `handleDamageEvent(DamageSource)`, as the server tells of damage: the hurt animation's
    /// timers and the cooldown. Java also plays the hurt sound (from the entity's random source)
    /// and remembers the source, which nothing ported reads.
    pub fn handle_damage_event(&mut self) {
        self.damage_cooldown_time = 20;
        self.hurt_duration = 10;
        self.hurt_time = self.hurt_duration;
    }

    /// `getLastClimbablePos()`: where it last climbed, until it lands.
    pub fn get_last_climbable_pos(&self) -> Option<BlockPos> {
        self.last_climbable_pos
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

    /// `getWaterSlowDown()`: how much of its movement it keeps in water each tick.
    fn get_water_slow_down(&self) -> f32 {
        0.8
    }

    /// `canStandOnFluid(FluidState)`: striders can, on lava; no ported entity can.
    fn can_stand_on_fluid(&self, _fluid: FluidState) -> bool {
        false
    }

    /// `isSuppressingSlidingDownLadder()`: sneaking.
    fn is_suppressing_sliding_down_ladder(&self) -> bool {
        self.is_shift_key_down()
    }

    /// `goDownInWater()`.
    fn go_down_in_water(&mut self) {
        let delta = self.entity().get_delta_movement();
        self.entity_mut()
            .set_delta_movement(delta.add_xyz(0.0, f64::from(-0.04_f32), 0.0));
    }

    /// `jumpInLiquid(TagKey)`.
    fn jump_in_liquid(&mut self, _fluid: FluidTagKey) {
        let delta = self.entity().get_delta_movement();
        self.entity_mut()
            .set_delta_movement(delta.add_xyz(0.0, f64::from(0.04_f32), 0.0));
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
    on_state: BlockState,
    pos: BlockPos,
) {
    if !this.entity().is_in_water() {
        this.update_fluid_interaction(level);
    }
    entity::check_fall_damage(this, level, ya, on_ground, on_state, pos);
    if on_ground {
        this.living_entity_mut().last_climbable_pos = None;
    }
}

/// `LivingEntity.onClimbable`: in a climbable block, or an open trapdoor over a ladder facing the
/// same way; where it climbs is kept.
pub fn on_climbable(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level) -> bool {
    if this.is_spectator() {
        return false;
    }
    let ladder_check_pos = this.entity().block_position();
    let state = this.entity_mut().get_in_block_state(level);
    if this.entity().is_fall_flying() && state.is_tag(BlockTags::CAN_GLIDE_THROUGH) {
        false
    } else if state.is_tag(BlockTags::CLIMBABLE)
        || state.get_block().is_instance_of(&trap_door_block::CLASS)
            && trapdoor_usable_as_ladder(level, &ladder_check_pos, state)
    {
        this.living_entity_mut().last_climbable_pos = Some(ladder_check_pos);
        true
    } else {
        false
    }
}

/// `trapdoorUsableAsLadder`: an open trapdoor over a ladder that faces the same way.
fn trapdoor_usable_as_ladder(level: &dyn Level, pos: &BlockPos, state: BlockState) -> bool {
    if !state.get_value(TrapDoorBlock::OPEN) {
        return false;
    }

    let below_state = level.get_block_state(&pos.below());
    below_state.is(Blocks::LADDER)
        && below_state.get_value(LadderBlock::FACING)
            == state.get_value(HorizontalDirectionalBlock::FACING)
}

/// `LivingEntity.baseTick()` on the client: `Entity`'s, then the hurt animation and the damage
/// cooldown count down. The rest is the server's (drowning, the border's damage) or not ported
/// (dying, which no scenario reaches).
pub fn base_tick(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level) {
    entity::base_tick(this, level);
    let living = this.living_entity_mut();
    if living.hurt_time > 0 {
        living.hurt_time -= 1;
    }

    // `!(this instanceof ServerPlayer)`: on the client, always.
    if living.damage_cooldown_time > 0 {
        living.damage_cooldown_time -= 1;
    }
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
        let fluid_height = if this.entity().is_in_lava() {
            this.entity().get_fluid_height(FluidTags::LAVA)
        } else {
            this.entity().get_fluid_height(FluidTags::WATER)
        };

        let in_water_and_has_fluid_height = this.entity().is_in_water() && fluid_height > 0.0;
        let fluid_jump_threshold = this.entity().get_fluid_jump_threshold();
        if !in_water_and_has_fluid_height
            || this.entity().on_ground() && !(fluid_height > fluid_jump_threshold)
        {
            if !this.entity().is_in_lava()
                || this.entity().on_ground() && is_in_shallow_fluid(this, FluidTags::LAVA)
            {
                if (this.entity().on_ground()
                    || in_water_and_has_fluid_height && fluid_height <= fluid_jump_threshold)
                    && this.living_entity().no_jump_delay == 0
                {
                    jump_from_ground(this, level);
                    this.living_entity_mut().no_jump_delay = 10;
                }
            } else {
                this.jump_in_liquid(FluidTags::LAVA);
            }
        } else {
            this.jump_in_liquid(FluidTags::WATER);
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

/// `LivingEntity.travel(Vec3)`: in a fluid, or in the air. Gliding isn't ported.
pub fn travel(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level, input: &Vec3) {
    let fluid_state = level.get_fluid_state(&this.entity().block_position());
    if should_travel_in_fluid(this, fluid_state) {
        travel_in_fluid(this, level, input);
    } else {
        assert!(!this.entity().is_fall_flying(), "gliding isn't ported");
        travel_in_air(this, level, input);
    }
}

/// `shouldTravelInFluid`: in water or lava, moved by fluids, and not standing on this one.
fn should_travel_in_fluid(
    this: &(impl LivingEntityClass + Sized),
    fluid_state: FluidState,
) -> bool {
    this.entity().is_in_liquid()
        && this.is_affected_by_fluids()
        && !this.can_stand_on_fluid(fluid_state)
}

/// `travelInFluid`: in water if it's in water, else in lava. Riding isn't ported, so nothing
/// floats with a rider (`floatInLiquidWhileRidden`).
fn travel_in_fluid(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level, input: &Vec3) {
    let is_falling = this.entity().get_delta_movement().y <= 0.0;
    let old_y = this.entity().get_y();
    let base_gravity = this.get_effective_gravity();
    if this.entity().is_in_water() {
        travel_in_water(this, level, input, base_gravity, is_falling, old_y);
    } else {
        travel_in_lava(this, level, input, base_gravity, is_falling, old_y);
    }
}

/// `travelInWater`: slowed by the water, less while sprinting and as water movement efficiency
/// grows (half of it off the ground), sinking slowly, and climbing where it pushes into a ladder.
fn travel_in_water(
    this: &mut (impl LivingEntityClass + Sized),
    level: &dyn Level,
    input: &Vec3,
    base_gravity: f64,
    is_falling: bool,
    old_y: f64,
) {
    let mut slow_down = if this.is_sprinting() {
        0.9
    } else {
        this.get_water_slow_down()
    };
    let mut speed = 0.02_f32;
    let mut water_walker =
        this.living_entity()
            .get_attribute_value(Attributes::WATER_MOVEMENT_EFFICIENCY) as f32;
    if !this.entity().on_ground() {
        water_walker *= 0.5;
    }

    if water_walker > 0.0 {
        slow_down += (0.54600006 - slow_down) * water_walker;
        speed += (this.get_speed() - speed) * water_walker;
    }

    // Dolphin's grace (an effect, not ported) would make the slow-down 0.96.
    entity::move_relative(this, speed, input);
    let delta = this.entity().get_delta_movement();
    this.r#move(level, MoverType::SELF, delta);
    let mut movement = this.entity().get_delta_movement();
    if this.entity().horizontal_collision && this.on_climbable(level) {
        movement = Vec3::new(movement.x, 0.2, movement.z);
    }

    movement = movement.multiply_xyz(
        f64::from(slow_down),
        f64::from(0.8_f32),
        f64::from(slow_down),
    );
    let movement = get_fluid_falling_adjusted_movement(this, base_gravity, is_falling, &movement);
    this.entity_mut().set_delta_movement(movement);
    jump_out_of_fluid(this, level, old_y);
}

/// `isInShallowFluid`: the fluid no deeper than the height it can jump out of.
pub fn is_in_shallow_fluid(
    this: &(impl LivingEntityClass + ?Sized),
    fluid_tag: FluidTagKey,
) -> bool {
    this.entity().get_fluid_height(fluid_tag) <= this.entity().get_fluid_jump_threshold()
}

/// `travelInLava`: half its movement kept, with a quarter of gravity on top.
fn travel_in_lava(
    this: &mut (impl LivingEntityClass + Sized),
    level: &dyn Level,
    input: &Vec3,
    base_gravity: f64,
    is_falling: bool,
    old_y: f64,
) {
    entity::move_relative(this, 0.02, input);
    let delta = this.entity().get_delta_movement();
    this.r#move(level, MoverType::SELF, delta);
    if is_in_shallow_fluid(this, FluidTags::LAVA) {
        let delta = this
            .entity()
            .get_delta_movement()
            .multiply_xyz(0.5, f64::from(0.8_f32), 0.5);
        this.entity_mut().set_delta_movement(delta);
        let movement = get_fluid_falling_adjusted_movement(
            this,
            base_gravity,
            is_falling,
            &this.entity().get_delta_movement(),
        );
        this.entity_mut().set_delta_movement(movement);
    } else {
        let delta = this.entity().get_delta_movement().scale(0.5);
        this.entity_mut().set_delta_movement(delta);
    }

    if base_gravity != 0.0 {
        let delta = this
            .entity()
            .get_delta_movement()
            .add_xyz(0.0, -base_gravity / 4.0, 0.0);
        this.entity_mut().set_delta_movement(delta);
    }

    jump_out_of_fluid(this, level, old_y);
}

/// `jumpOutOfFluid`: pushing into an edge it would be clear over, it's lifted out.
fn jump_out_of_fluid(this: &mut (impl LivingEntityClass + Sized), level: &dyn Level, old_y: f64) {
    let movement = this.entity().get_delta_movement();
    if this.entity().horizontal_collision
        && entity::is_free(
            this,
            level,
            movement.x,
            movement.y + f64::from(0.6_f32) - this.entity().get_y() + old_y,
            movement.z,
        )
    {
        this.entity_mut()
            .set_delta_movement_xyz(movement.x, f64::from(0.3_f32), movement.z);
    }
}

/// `getFluidFallingAdjustedMovement`: unless it sprints, a sixteenth of gravity down, and a slow
/// sink where that would leave it nearly still.
pub fn get_fluid_falling_adjusted_movement(
    this: &(impl LivingEntityClass + ?Sized),
    base_gravity: f64,
    is_falling: bool,
    movement: &Vec3,
) -> Vec3 {
    if base_gravity != 0.0 && !this.is_sprinting() {
        let yd = if is_falling
            && (movement.y - 0.005).abs() >= 0.003
            && (movement.y - base_gravity / 16.0).abs() < 0.003
        {
            -0.003
        } else {
            movement.y - base_gravity / 16.0
        };

        Vec3::new(movement.x, yd, movement.z)
    } else {
        *movement
    }
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

/// `handleOnClimbable`: on a ladder, no fall, at most 0.15 a tick sideways and down, and a player
/// sneaking off scaffolding doesn't slide down.
fn handle_on_climbable(
    this: &mut (impl LivingEntityClass + Sized),
    level: &dyn Level,
    delta: Vec3,
) -> Vec3 {
    let mut delta = delta;
    if this.on_climbable(level) {
        this.entity_mut().reset_fall_distance();
        let xd = Mth::clamp_f64(delta.x, f64::from(-0.15_f32), f64::from(0.15_f32));
        let zd = Mth::clamp_f64(delta.z, f64::from(-0.15_f32), f64::from(0.15_f32));
        let mut yd = Math::max_f64(delta.y, f64::from(-0.15_f32));
        if yd < 0.0
            && !this
                .entity_mut()
                .get_in_block_state(level)
                .is(Blocks::SCAFFOLDING)
            && this.is_suppressing_sliding_down_ladder()
            && this.is_player()
        {
            yd = 0.0;
        }

        delta = Vec3::new(xd, yd, zd);
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
