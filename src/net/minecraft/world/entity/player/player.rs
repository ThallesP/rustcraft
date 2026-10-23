//! net/minecraft/world/entity/player/Player.java
//!
//! What a player adds to a living entity's movement: its abilities (flying), `tick`'s pose and
//! the world's edge, `aiStep`'s speed, `travel` (swimming toward where it looks), the flying
//! speed, sneaking back from an edge (`maybeBackOffFromEdge`), whether it's under water and
//! swimming, and its attributes (`createAttributes`). `PlayerClass` holds the methods its
//! subclasses override.
//!
//! Not ported yet: the inventory and containers, food beyond its level, experience, sleeping,
//! attacking, item cooldowns, the shoulder entities, picking items up, and fall damage. A
//! player's game mode is a parameter of what reads it (`isSpectator`).

// Java's negated comparisons are kept as they are: they differ from the plain ones for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use crate::java::lang::math::Math;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::tags::fluid_tags::FluidTags;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::ai::attributes::attribute::Attribute;
use crate::net::minecraft::world::entity::ai::attributes::attribute_map::AttributeMap;
use crate::net::minecraft::world::entity::ai::attributes::attributes::Attributes;
use crate::net::minecraft::world::entity::avatar::Avatar;
use crate::net::minecraft::world::entity::entity;
use crate::net::minecraft::world::entity::entity_types::EntityTypes;
use crate::net::minecraft::world::entity::humanoid_arm::HumanoidArm;
use crate::net::minecraft::world::entity::living_entity::{self, LivingEntity, LivingEntityClass};
use crate::net::minecraft::world::entity::mover_type::MoverType;
use crate::net::minecraft::world::entity::player::abilities::Abilities;
use crate::net::minecraft::world::entity::player::player_model_part::PlayerModelPart;
use crate::net::minecraft::world::entity::pose::Pose;
use crate::net::minecraft::world::level::level::{self, Level};
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// What a player has besides a living entity's.
#[derive(Clone, Debug)]
pub struct Player {
    living_entity: LivingEntity,
    abilities: Abilities,
    pub jump_trigger_time: i32,
    pub was_underwater: bool,
    /// `FoodData`'s food level.
    pub food_level: i32,
    /// `FoodData`'s saturation level.
    pub saturation_level: f32,
    pub experience_level: i32,
    pub total_experience: i32,
    pub experience_progress: f32,
    /// The inventory's selected slot (`getInventory().getSelectedSlot()`): the inventory isn't
    /// ported yet.
    pub selected_slot: i32,
    /// `Avatar.DATA_PLAYER_MAIN_HAND`, as the server last set it.
    pub main_arm: HumanoidArm,
    /// `Avatar.DATA_PLAYER_MODE_CUSTOMISATION`: the skin's parts shown, as the server last set
    /// them (the masks of the `PlayerModelPart`s the client sent).
    pub model_customisation: i8,
}

impl std::ops::Deref for Player {
    type Target = LivingEntity;

    fn deref(&self) -> &LivingEntity {
        &self.living_entity
    }
}

impl std::ops::DerefMut for Player {
    fn deref_mut(&mut self) -> &mut LivingEntity {
        &mut self.living_entity
    }
}

impl Player {
    /// `createAttributes()`: a living entity's attributes that movement reads, and its armor,
    /// with the player's speed of 0.1F.
    pub fn create_attributes() -> Vec<(&'static Attribute, f64)> {
        [
            Attributes::MAX_HEALTH,
            Attributes::MOVEMENT_SPEED,
            Attributes::ARMOR,
            Attributes::STEP_HEIGHT,
            Attributes::SCALE,
            Attributes::GRAVITY,
            Attributes::JUMP_STRENGTH,
            Attributes::WATER_MOVEMENT_EFFICIENCY,
            Attributes::MOVEMENT_EFFICIENCY,
            Attributes::BOUNCINESS,
            Attributes::AIR_DRAG_MODIFIER,
            Attributes::FRICTION_MODIFIER,
            Attributes::SNEAKING_SPEED,
            Attributes::ATTACK_SPEED,
            Attributes::BLOCK_INTERACTION_RANGE,
            Attributes::ENTITY_INTERACTION_RANGE,
        ]
        .into_iter()
        .map(|attribute| {
            let base_value = if Attribute::same(attribute, Attributes::MOVEMENT_SPEED) {
                f64::from(0.1_f32)
            } else {
                attribute.get_default_value()
            };
            (attribute, base_value)
        })
        .collect()
    }

    pub fn new() -> Player {
        Player {
            living_entity: LivingEntity::new(
                EntityTypes::PLAYER,
                Avatar::standing_dimensions(),
                AttributeMap::new(&Player::create_attributes()),
            ),
            abilities: Abilities::default(),
            jump_trigger_time: 0,
            was_underwater: false,
            food_level: 20,
            saturation_level: 5.0,
            experience_level: 0,
            total_experience: 0,
            experience_progress: 0.0,
            selected_slot: 0,
            main_arm: HumanoidArm::RIGHT,
            model_customisation: 0,
        }
    }

    pub fn get_xp_needed_for_next_level(&self) -> i32 {
        if self.experience_level >= 30 {
            112 + (self.experience_level - 30) * 9
        } else if self.experience_level >= 15 {
            37 + (self.experience_level - 15) * 5
        } else {
            7 + self.experience_level * 2
        }
    }

    pub fn get_abilities(&self) -> &Abilities {
        &self.abilities
    }

    pub fn get_abilities_mut(&mut self) -> &mut Abilities {
        &mut self.abilities
    }

    /// `hasEnoughFoodToDoExhaustiveManoeuvres()`: more than 6 food, or able to fly.
    pub fn has_enough_food_to_do_exhaustive_manoeuvres(&self) -> bool {
        self.food_level > 6 || self.abilities.mayfly
    }

    /// `Avatar.getMainArm()`.
    pub fn get_main_arm(&self) -> HumanoidArm {
        self.main_arm
    }

    /// `Avatar.isModelPartShown`.
    pub fn is_model_part_shown(&self, part: PlayerModelPart) -> bool {
        (i32::from(self.model_customisation) & part.get_mask()) == part.get_mask()
    }

    /// `blockInteractionRange()`.
    pub fn block_interaction_range(&self) -> f64 {
        self.get_attribute_value(Attributes::BLOCK_INTERACTION_RANGE)
    }

    /// `entityInteractionRange()`.
    pub fn entity_interaction_range(&self) -> f64 {
        self.get_attribute_value(Attributes::ENTITY_INTERACTION_RANGE)
    }

    /// `getCurrentItemAttackStrengthDelay()`: the ticks an attack takes to charge.
    pub fn get_current_item_attack_strength_delay(&self) -> f32 {
        (1.0 / self.get_attribute_value(Attributes::ATTACK_SPEED) * 20.0) as f32
    }

    pub fn get_attack_strength_scale(&self, a: f32) -> f32 {
        Mth::clamp_f32(
            (self.attack_strength_ticker as f32 + a)
                / self.get_current_item_attack_strength_delay(),
            0.0,
            1.0,
        )
    }

    /// How far the held item is back up after a swap, 0 to 1.
    pub fn get_item_swap_scale(&self, a: f32) -> f32 {
        Mth::clamp_f32(
            (self.item_swap_ticker as f32 + a) / self.get_current_item_attack_strength_delay(),
            0.0,
            1.0,
        )
    }

    pub fn reset_attack_strength_ticker(&mut self) {
        self.attack_strength_ticker = 0;
        self.item_swap_ticker = 0;
    }

    /// `isMobilityRestricted()`: blind, which is an effect, not ported.
    pub fn is_mobility_restricted(&self) -> bool {
        false
    }
}

impl Default for Player {
    fn default() -> Player {
        Player::new()
    }
}

/// Java's `Player` methods that subclasses override or that call overridden ones.
pub trait PlayerClass: LivingEntityClass {
    fn player(&self) -> &Player;

    fn player_mut(&mut self) -> &mut Player;

    /// `isStayingOnGroundSurface()`: sneaking.
    fn is_staying_on_ground_surface(&self) -> bool {
        self.is_shift_key_down()
    }

    fn update_is_underwater(&mut self) -> bool {
        let was_underwater = self.entity().is_eye_in_fluid(FluidTags::WATER);
        self.player_mut().was_underwater = was_underwater;
        was_underwater
    }
}

/// `Player.getDefaultDimensions` through `LivingEntity.getDimensions`.
pub fn get_dimensions(
    this: &(impl PlayerClass + ?Sized),
    pose: Pose,
) -> crate::net::minecraft::world::entity::entity_dimensions::EntityDimensions {
    living_entity::get_dimensions(this, pose, Avatar::get_default_dimensions(pose))
}

/// `Player.isSwimming()`: never while flying.
pub fn is_swimming(this: &(impl PlayerClass + ?Sized)) -> bool {
    !this.player().abilities.flying
        && !this.is_spectator()
        && this
            .entity()
            .get_shared_flag(crate::net::minecraft::world::entity::entity::Entity::FLAG_SWIMMING)
}

/// `Player.getSpeed()`: the movement speed attribute.
pub fn get_speed(this: &(impl PlayerClass + ?Sized)) -> f32 {
    this.living_entity()
        .get_attribute_value(Attributes::MOVEMENT_SPEED) as f32
}

/// `Player.getFlyingSpeed()`: the abilities' while flying, else 0.02, more when sprinting.
pub fn get_flying_speed(this: &(impl PlayerClass + ?Sized)) -> f32 {
    if this.player().abilities.flying && !this.entity().is_passenger() {
        if this.is_sprinting() {
            this.player().abilities.get_flying_speed() * 2.0
        } else {
            this.player().abilities.get_flying_speed()
        }
    } else if this.is_sprinting() {
        0.025999999
    } else {
        0.02
    }
}

pub fn is_affected_by_fluids(this: &(impl PlayerClass + ?Sized)) -> bool {
    !this.player().abilities.flying
}

pub fn on_climbable(this: &mut (impl PlayerClass + Sized), level: &dyn Level) -> bool {
    if this.player().abilities.flying {
        false
    } else {
        living_entity::on_climbable(this, level)
    }
}

/// `Player.getBlockSpeedFactor()`: none while flying or gliding.
pub fn get_block_speed_factor(this: &mut (impl PlayerClass + Sized), level: &dyn Level) -> f32 {
    if !this.player().abilities.flying && !this.entity().is_fall_flying() {
        living_entity::get_block_speed_factor(this, level)
    } else {
        1.0
    }
}

/// `Player.tick()`: the living entity's tick, kept inside the world's edge, then the pose.
pub fn tick(this: &mut (impl PlayerClass + Sized), level: &dyn Level) {
    this.entity_mut().no_physics = this.is_spectator();
    if this.is_spectator() || this.entity().is_passenger() {
        entity::set_on_ground(this, level, false);
    }

    this.update_is_underwater();
    living_entity::tick(this, level);
    let nx = Mth::clamp_f64(this.entity().get_x(), -2.9999999E7, 2.9999999E7);
    let nz = Mth::clamp_f64(this.entity().get_z(), -2.9999999E7, 2.9999999E7);
    if nx != this.entity().get_x() || nz != this.entity().get_z() {
        let y = this.entity().get_y();
        this.entity_mut().set_pos(nx, y, nz);
    }

    // The held item never changes (the inventory isn't ported), so the tickers only count.
    let player = this.player_mut();
    player.attack_strength_ticker += 1;
    player.item_swap_ticker += 1;
    update_player_pose(this, level);
}

/// The pose the player wants, or crouching or swimming when that's what fits.
fn update_player_pose(this: &mut (impl PlayerClass + Sized), level: &dyn Level) {
    if can_player_fit_within_blocks_and_entities_when(this, level, Pose::SWIMMING) {
        let desired_pose = get_desired_pose(this);
        let actual_pose = if this.is_spectator()
            || this.entity().is_passenger()
            || can_player_fit_within_blocks_and_entities_when(this, level, desired_pose)
        {
            desired_pose
        } else if can_player_fit_within_blocks_and_entities_when(this, level, Pose::CROUCHING) {
            Pose::CROUCHING
        } else {
            Pose::SWIMMING
        };
        entity::set_pose(this, actual_pose);
    }
}

fn get_desired_pose(this: &(impl PlayerClass + Sized)) -> Pose {
    if this.is_swimming() {
        Pose::SWIMMING
    } else if this.entity().is_fall_flying() {
        Pose::FALL_FLYING
    } else if this.is_shift_key_down() && !this.player().abilities.flying {
        Pose::CROUCHING
    } else {
        Pose::STANDING
    }
}

/// Whether the pose's box, a hair smaller, is free where the player is.
pub fn can_player_fit_within_blocks_and_entities_when(
    this: &(impl PlayerClass + Sized),
    level: &dyn Level,
    new_pose: Pose,
) -> bool {
    let r#box = this
        .get_dimensions(new_pose)
        .make_bounding_box(&this.entity().position())
        .deflate(1.0E-7);
    level::collisions(level).no_collision(Some(this), &r#box)
}

/// `Player.aiStep()`: the living entity's, then the speed from the attribute.
pub fn ai_step(this: &mut (impl PlayerClass + Sized), level: &dyn Level) {
    if this.player().jump_trigger_time > 0 {
        this.player_mut().jump_trigger_time -= 1;
    }

    if this.player().abilities.flying && !this.entity().is_passenger() {
        this.entity_mut().reset_fall_distance();
    }

    living_entity::ai_step(this, level);
    let speed = this
        .living_entity()
        .get_attribute_value(Attributes::MOVEMENT_SPEED) as f32;
    this.living_entity_mut().set_speed(speed);
}

/// `Player.updateSwimming()`: never while flying.
pub fn update_swimming(this: &mut (impl PlayerClass + Sized), level: &dyn Level) {
    if this.player().abilities.flying {
        this.entity_mut().set_swimming(false);
    } else {
        entity::update_swimming(this, level);
    }
}

/// `Player.travel(Vec3)`: a swimmer rises or sinks toward where it looks (unless it looks up out
/// of the water without jumping), and flying keeps 60% of the vertical movement.
pub fn travel(this: &mut (impl PlayerClass + Sized), level: &dyn Level, input: &Vec3) {
    if this.entity().is_passenger() {
        living_entity::travel(this, level, input);
        return;
    }

    if this.is_swimming() {
        let look_angle_y =
            entity::calculate_view_vector(this.entity().get_x_rot(), this.entity().get_y_rot()).y;
        let multiplier = if look_angle_y < -0.2 { 0.085 } else { 0.06 };
        if look_angle_y <= 0.0
            || this.living_entity().jumping
            || !level
                .get_fluid_state(&BlockPos::containing_xyz(
                    this.entity().get_x(),
                    this.entity().get_y() + 1.0 - 0.1,
                    this.entity().get_z(),
                ))
                .is_empty()
        {
            let movement = this.entity().get_delta_movement();
            this.entity_mut().set_delta_movement(movement.add_xyz(
                0.0,
                (look_angle_y - movement.y) * multiplier,
                0.0,
            ));
        }
    }

    if this.player().abilities.flying {
        let original_movement_y = this.entity().get_delta_movement().y;
        living_entity::travel(this, level, input);
        let delta = this.entity().get_delta_movement();
        this.entity_mut().set_delta_movement(delta.with(
            crate::net::minecraft::core::direction::Axis::Y,
            original_movement_y * 0.6,
        ));
    } else {
        living_entity::travel(this, level, input);
    }
}

/// `Player.maybeBackOffFromEdge`: a player sneaking on the ground stops its movement short of
/// where it could fall farther than its step height, 0.05 at a time on each axis.
pub fn maybe_back_off_from_edge(
    this: &mut (impl PlayerClass + Sized),
    level: &dyn Level,
    delta: Vec3,
    mover_type: MoverType,
) -> Vec3 {
    let max_down_step = this.max_up_step();
    if !this.player().abilities.flying
        && !(delta.y > 0.0)
        && (mover_type == MoverType::SELF || mover_type == MoverType::PLAYER)
        && this.is_staying_on_ground_surface()
        && is_above_ground(this, level, max_down_step)
    {
        let mut delta_x = delta.x;
        let mut delta_z = delta.z;
        let step_x = Math::signum_f64(delta_x) * 0.05;
        let step_z = Math::signum_f64(delta_z) * 0.05;

        while delta_x != 0.0
            && can_fall_at_least(this, level, delta_x, 0.0, f64::from(max_down_step))
        {
            if delta_x.abs() <= 0.05 {
                delta_x = 0.0;
                break;
            }

            delta_x -= step_x;
        }

        while delta_z != 0.0
            && can_fall_at_least(this, level, 0.0, delta_z, f64::from(max_down_step))
        {
            if delta_z.abs() <= 0.05 {
                delta_z = 0.0;
                break;
            }

            delta_z -= step_z;
        }

        while delta_x != 0.0
            && delta_z != 0.0
            && can_fall_at_least(this, level, delta_x, delta_z, f64::from(max_down_step))
        {
            if delta_x.abs() <= 0.05 {
                delta_x = 0.0;
            } else {
                delta_x -= step_x;
            }

            if delta_z.abs() <= 0.05 {
                delta_z = 0.0;
            } else {
                delta_z -= step_z;
            }
        }

        Vec3::new(delta_x, delta.y, delta_z)
    } else {
        delta
    }
}

fn is_above_ground(
    this: &(impl PlayerClass + Sized),
    level: &dyn Level,
    max_down_step: f32,
) -> bool {
    this.entity().on_ground()
        || this.entity().fall_distance < f64::from(max_down_step)
            && !can_fall_at_least(
                this,
                level,
                0.0,
                0.0,
                f64::from(max_down_step) - this.entity().fall_distance,
            )
}

fn can_fall_at_least(
    this: &(impl PlayerClass + Sized),
    level: &dyn Level,
    delta_x: f64,
    delta_z: f64,
    min_height: f64,
) -> bool {
    let bounding_box = this.entity().get_bounding_box();
    level::collisions(level).no_collision(
        Some(this),
        &AABB::new(
            bounding_box.min_x + 1.0E-7 + delta_x,
            bounding_box.min_y - min_height - 1.0E-7,
            bounding_box.min_z + 1.0E-7 + delta_z,
            bounding_box.max_x - 1.0E-7 + delta_x,
            bounding_box.min_y,
            bounding_box.max_z - 1.0E-7 + delta_z,
        ),
    )
}
