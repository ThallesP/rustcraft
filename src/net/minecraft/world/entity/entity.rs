//! net/minecraft/world/entity/Entity.java
//!
//! The entity hierarchy (PORTING.md, "Entities"): a class's fields are a struct that holds its
//! parent's (`LivingEntity` holds an `Entity`), and Java's methods that subclasses override, or
//! that call overridden ones, are methods of a trait per class (`EntityClass` here) whose defaults
//! are the class's own, written as functions of this module that an override can call like
//! `super`. The level an entity lives in is a parameter (`&dyn Level`), as are the entities'
//! back-references.
//!
//! What's here is an entity's movement: `move` with `collide` (step-up included),
//! `collideBoundingBox` and `collideWithShapes`, the collision flags, the supporting block and
//! `getOnPos`, the fall distance, `restituteMovementAfterCollisions` (bouncing), the blocks' speed
//! and jump factors, `moveRelative`, its position, box, pose and size, the shared flags, the fluids
//! it's in (`updateFluidInteraction` through `EntityFluidInteraction`, with their currents pushing
//! it), swimming (`updateSwimming`), `baseTick` and `commonTick`.
//!
//! The blocks it moves through: each tick's moves (`movementThisTick`, which `move` adds to), and
//! `applyEffectsFromBlocks` over them, the block it stands on (`stepOn`), then `checkInsideBlocks`
//! along each move, axis by axis in its collision's order, for the blocks and fluids it's inside
//! (`entityInside`), whose effects an `InsideBlockEffectApplier.StepBasedCollector` applies in
//! their order once every move is checked: getting stuck (`makeStuckInBlock`), bubble columns
//! (`onAboveBubbleColumn`, `onInsideBubbleColumn`), powder snow (`isInPowderSnow`) and freezing
//! (`DATA_TICKS_FROZEN`'s value, a plain field like the air).
//!
//! Not ported yet, with what they need:
//! - water's splash (`doWaterSplashEffect`), which only plays a sound and makes particles, from
//!   the entity's random source; lava's speed in a dimension with fast lava
//!   (`EnvironmentAttributes.FAST_LAVA`, the nether's), which panics;
//! - the movement's sounds and game events (`applyMovementEmissionAndPlaySound`, a bounce's
//!   `GameEvent.BOUNCE`), which change nothing an entity's movement reads; the blocks' `fallOn`
//!   and `bounceOn`, which change nothing the client has (`Block`);
//! - vehicles and passengers, portals, fire (what the blocks' fire effects and rain would change,
//!   and the extinguishing sound), `lastKnownSpeed`, and entity data (only the shared flags, pose,
//!   no-gravity flag, air and ticks frozen are here, as fields);
//! - the server's side of the blocks' effects: `debugBlockIntersection`, and the fire ticks
//!   `applyEffectsFromBlocks` resets there.
//!
//! The entities the server moves are interpolated (`moveOrInterpolateTo`), each with what its
//! class's `createInterpolationHandler` makes (`create_interpolation_handler`, by its type until
//! the classes come), from the moves' deltas off its position codec (`getPositionCodec`).

// Java's negated comparisons are kept as they are: they differ from the plain ones for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use std::collections::{HashSet, VecDeque};

use crate::java::lang::float::Float;
use crate::java::lang::math::Math;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::network::protocol::game::clientbound_add_entity_packet::ClientboundAddEntityPacket;
use crate::net::minecraft::network::protocol::game::vec_delta_codec::VecDeltaCodec;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::tags::entity_type_tags::EntityTypeTags;
use crate::net::minecraft::tags::fluid_tags::{FluidTagKey, FluidTags};
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::abstract_interpolation_handler::Interpolated;
use crate::net::minecraft::world::entity::animal::pig::pig::Pig;
use crate::net::minecraft::world::entity::decoration::armor_stand::ArmorStand;
use crate::net::minecraft::world::entity::entity_dimensions::EntityDimensions;
use crate::net::minecraft::world::entity::entity_fluid_interaction::EntityFluidInteraction;
use crate::net::minecraft::world::entity::entity_type::EntityType;
use crate::net::minecraft::world::entity::entity_types::EntityTypes;
use crate::net::minecraft::world::entity::inside_block_effect_applier::StepBasedCollector;
use crate::net::minecraft::world::entity::interpolation_handler::InterpolationHandler;
use crate::net::minecraft::world::entity::item::falling_block_entity::FallingBlockEntity;
use crate::net::minecraft::world::entity::item::item_entity::ItemEntity;
use crate::net::minecraft::world::entity::linear_interpolation_handler::LinearInterpolationHandler;
use crate::net::minecraft::world::entity::living_entity::{self, LivingEntityClass};
use crate::net::minecraft::world::entity::move_simulation_type::MoveSimulationType;
use crate::net::minecraft::world::entity::mover_type::MoverType;
use crate::net::minecraft::world::entity::pose::Pose;
use crate::net::minecraft::world::entity::position_path::PositionPath;
use crate::net::minecraft::world::entity::stepped_interpolation_handler::SteppedInterpolationHandler;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::fence_gate_block;
use crate::net::minecraft::world::level::block::powder_snow_block::PowderSnowBlock;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter;
use crate::net::minecraft::world::level::clip_context::{self, ClipContext};
use crate::net::minecraft::world::level::collision_getter::CollisionGetter;
use crate::net::minecraft::world::level::entity::entity_lookup::EntityAccess;
use crate::net::minecraft::world::level::level::{self, Level};
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::hit_result::Type;
use crate::net::minecraft::world::phys::shapes::entity_collision_context::{
    CollidingEntity, EntityCollisionContext, LivingCollision,
};
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// What every entity has.
#[derive(Clone, Debug)]
pub struct Entity {
    r#type: EntityType,
    id: i32,
    uuid: UUID,
    position: Vec3,
    block_position: BlockPos,
    delta_movement: Vec3,
    y_rot: f32,
    x_rot: f32,
    pub y_rot_o: f32,
    pub x_rot_o: f32,
    old_position: Vec3,
    bb: AABB,
    on_ground: bool,
    pub horizontal_collision: bool,
    pub vertical_collision: bool,
    pub vertical_collision_below: bool,
    pub minor_horizontal_collision: bool,
    pub fall_distance: f64,
    dimensions: EntityDimensions,
    eye_height: f32,
    pub no_physics: bool,
    stuck_speed_multiplier: Vec3,
    /// `pistonDeltas` and `pistonDeltasGameTime`: how far pistons have moved it along each axis
    /// this game tick.
    piston_deltas: [f64; 3],
    piston_deltas_game_time: i64,
    main_supporting_block_pos: Option<BlockPos>,
    on_ground_no_blocks: bool,
    fluid_interaction: EntityFluidInteraction,
    was_touching_water: bool,
    was_eye_in_water: bool,
    pub was_in_powder_snow: bool,
    pub is_in_powder_snow: bool,
    pub first_tick: bool,
    pub tick_count: i32,
    shared_flags: i8,
    pose: Pose,
    no_gravity: bool,
    in_block_state: Option<BlockState>,
    /// `DATA_AIR_SUPPLY_ID`'s value, synced data that's a plain field until entity data is
    /// ported, like the shared flags.
    air_supply: i32,
    /// `DATA_TICKS_FROZEN`'s value, the same.
    ticks_frozen: i32,
    /// `movementThisTick`: the moves made since the blocks' effects were last applied, at most
    /// 100.
    movement_this_tick: VecDeque<Movement>,
    /// `finalMovementsThisTick`: the moves the blocks' effects were last applied over.
    final_movements_this_tick: Vec<Movement>,
    /// `packetPositionCodec`: where the server's moves have it, which their deltas are from.
    packet_position_codec: VecDeltaCodec,
    interpolation_handler: InterpolationHandler,
}

impl Entity {
    /// `applyPistonMovementRestriction`: the part of `amount` that keeps this tick's pushes
    /// along the axis within 0.51 either way.
    fn apply_piston_movement_restriction(&mut self, axis: Axis, amount: f64) -> f64 {
        let ordinal = axis as usize;
        let min = Mth::clamp_f64(amount + self.piston_deltas[ordinal], -0.51, 0.51);
        let amount = min - self.piston_deltas[ordinal];
        self.piston_deltas[ordinal] = min;
        amount
    }

    pub const FLAG_ONFIRE: i32 = 0;
    pub const FLAG_SHIFT_KEY_DOWN: i32 = 1;
    pub const FLAG_SPRINTING: i32 = 3;
    pub const FLAG_SWIMMING: i32 = 4;
    pub const FLAG_INVISIBLE: i32 = 5;
    pub const FLAG_GLOWING: i32 = 6;
    pub const FLAG_FALL_FLYING: i32 = 7;

    /// `FLUIDS_WITH_CURRENT`.
    const FLUIDS_WITH_CURRENT: [FluidTagKey; 2] = [FluidTags::WATER, FluidTags::LAVA];

    /// An entity of the type, of these dimensions (the type's, or a living entity's for its
    /// pose), at the level's origin, like Java's constructor before anything moves it.
    pub fn new(r#type: EntityType, dimensions: EntityDimensions) -> Entity {
        Entity {
            r#type,
            id: 0,
            uuid: UUID::new(0, 0),
            position: Vec3::ZERO,
            block_position: BlockPos::ZERO,
            delta_movement: Vec3::ZERO,
            y_rot: 0.0,
            x_rot: 0.0,
            y_rot_o: 0.0,
            x_rot_o: 0.0,
            old_position: Vec3::ZERO,
            bb: dimensions.make_bounding_box(&Vec3::ZERO),
            on_ground: false,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            minor_horizontal_collision: false,
            fall_distance: 0.0,
            eye_height: dimensions.eye_height(),
            dimensions,
            no_physics: false,
            stuck_speed_multiplier: Vec3::ZERO,
            piston_deltas: [0.0; 3],
            piston_deltas_game_time: 0,
            main_supporting_block_pos: None,
            on_ground_no_blocks: false,
            fluid_interaction: EntityFluidInteraction::new(&Entity::FLUIDS_WITH_CURRENT),
            was_touching_water: false,
            was_eye_in_water: false,
            was_in_powder_snow: false,
            is_in_powder_snow: false,
            first_tick: true,
            tick_count: 0,
            shared_flags: 0,
            pose: Pose::STANDING,
            no_gravity: false,
            in_block_state: None,
            air_supply: Entity::TOTAL_AIR_SUPPLY,
            ticks_frozen: 0,
            movement_this_tick: VecDeque::with_capacity(100),
            final_movements_this_tick: Vec::new(),
            packet_position_codec: VecDeltaCodec::new(),
            interpolation_handler: InterpolationHandler::NoOp,
        }
    }

    /// What the class of an entity of the type makes in `createInterpolationHandler`, on the client:
    /// a living entity's moves are stepped over its type's update interval (`LivingEntity`), but a
    /// shulker's, which stays put; boats', minecarts', experience orbs' and fishing bobbers' are
    /// linear over 3 ticks, displays' at once; any other's isn't interpolated (`Entity`'s
    /// `NO_OP`). The living types are the 93 whose classes extend `LivingEntity`, the shulker
    /// left out.
    pub fn create_interpolation_handler(r#type: EntityType) -> InterpolationHandler {
        const LIVING: [&str; 92] = [
            "allay",
            "armadillo",
            "armor_stand",
            "axolotl",
            "bat",
            "bee",
            "blaze",
            "bogged",
            "breeze",
            "camel",
            "camel_husk",
            "cat",
            "cave_spider",
            "chicken",
            "cod",
            "copper_golem",
            "cow",
            "creaking",
            "creeper",
            "dolphin",
            "donkey",
            "drowned",
            "elder_guardian",
            "enderman",
            "endermite",
            "ender_dragon",
            "evoker",
            "fox",
            "frog",
            "ghast",
            "happy_ghast",
            "giant",
            "glow_squid",
            "goat",
            "guardian",
            "hoglin",
            "horse",
            "husk",
            "illusioner",
            "iron_golem",
            "llama",
            "magma_cube",
            "mannequin",
            "mooshroom",
            "mule",
            "nautilus",
            "ocelot",
            "panda",
            "parched",
            "parrot",
            "phantom",
            "pig",
            "piglin",
            "piglin_brute",
            "pillager",
            "polar_bear",
            "pufferfish",
            "rabbit",
            "ravager",
            "salmon",
            "sheep",
            "silverfish",
            "skeleton",
            "skeleton_horse",
            "slime",
            "sniffer",
            "snow_golem",
            "spider",
            "squid",
            "stray",
            "strider",
            "sulfur_cube",
            "tadpole",
            "trader_llama",
            "tropical_fish",
            "turtle",
            "vex",
            "villager",
            "vindicator",
            "wandering_trader",
            "warden",
            "witch",
            "wither",
            "wither_skeleton",
            "wolf",
            "zoglin",
            "zombie",
            "zombie_horse",
            "zombie_nautilus",
            "zombie_villager",
            "zombified_piglin",
            "player",
        ];
        let name = r#type.to_short_string();
        if LIVING.contains(&name) {
            InterpolationHandler::Stepped(SteppedInterpolationHandler::create(
                r#type.update_interval(),
            ))
        } else if name.ends_with("_boat")
            || name.ends_with("_raft")
            || name.ends_with("minecart")
            || name == "experience_orb"
            || name == "fishing_bobber"
        {
            InterpolationHandler::Linear(LinearInterpolationHandler::create(
                LinearInterpolationHandler::DEFAULT_INTERPOLATION_STEPS,
            ))
        } else if name.ends_with("_display") {
            InterpolationHandler::Linear(LinearInterpolationHandler::create(0))
        } else {
            InterpolationHandler::NoOp
        }
    }

    pub fn set_interpolation_handler(&mut self, interpolation_handler: InterpolationHandler) {
        self.interpolation_handler = interpolation_handler;
    }

    pub fn get_interpolation(&self) -> &InterpolationHandler {
        &self.interpolation_handler
    }

    pub fn is_interpolating(&self) -> bool {
        self.interpolation_handler.has_active_interpolation()
    }

    /// `syncPacketPositionCodec(double, double, double)`.
    pub fn sync_packet_position_codec(&mut self, x: f64, y: f64, z: f64) {
        self.packet_position_codec.set_base(Vec3::new(x, y, z));
    }

    /// `getPositionCodec()`.
    pub fn get_position_codec(&mut self) -> &mut VecDeltaCodec {
        &mut self.packet_position_codec
    }

    /// `moveOrInterpolateTo(PositionPath, float, float, boolean)`: its handler takes it there, or
    /// it's put there at once.
    pub fn move_or_interpolate_to(
        &mut self,
        level: &dyn Level,
        position: Option<&PositionPath>,
        y_rot: f32,
        x_rot: f32,
        has_rotation: bool,
    ) {
        let mut handler = std::mem::take(&mut self.interpolation_handler);
        let interpolated = handler.interpolate_to(
            &mut InLevel {
                entity: self,
                level,
            },
            position,
            y_rot,
            x_rot,
            has_rotation,
        );
        self.interpolation_handler = handler;
        if !interpolated {
            if let Some(position) = position {
                self.set_pos_vec3(&position.end_position());
            }

            if has_rotation {
                self.set_rot(y_rot, x_rot);
            }
        }
    }

    /// `push(double, double, double)`: a finite impulse added to its movement. The server's
    /// `needsSync`, which it sets too, isn't the client's.
    pub fn push_xyz(&mut self, xa: f64, ya: f64, za: f64) {
        if xa.is_finite() && ya.is_finite() && za.is_finite() {
            self.set_delta_movement(self.get_delta_movement().add_xyz(xa, ya, za));
        }
    }

    /// `setRot(float, float)`: each rotation `% 360`.
    pub fn set_rot(&mut self, y_rot: f32, x_rot: f32) {
        self.set_y_rot(y_rot % 360.0);
        self.set_x_rot(x_rot % 360.0);
    }

    /// `commonTick()` on the client, for an entity in the level: where it was, its interpolation's
    /// step, and the tick counted. The player's own (`common_tick`) has none to step.
    pub fn common_tick_in(&mut self, level: &dyn Level) {
        self.set_old_pos_and_rot();
        let mut handler = std::mem::take(&mut self.interpolation_handler);
        handler.interpolate(&mut InLevel {
            entity: self,
            level,
        });
        self.interpolation_handler = handler;
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    /// `TOTAL_AIR_SUPPLY`, what `getMaxAirSupply()` answers.
    pub const TOTAL_AIR_SUPPLY: i32 = 300;

    pub fn get_max_air_supply(&self) -> i32 {
        Entity::TOTAL_AIR_SUPPLY
    }

    pub fn get_air_supply(&self) -> i32 {
        self.air_supply
    }

    pub fn set_air_supply(&mut self, supply: i32) {
        self.air_supply = supply;
    }

    pub fn get_ticks_frozen(&self) -> i32 {
        self.ticks_frozen
    }

    pub fn set_ticks_frozen(&mut self, ticks: i32) {
        self.ticks_frozen = ticks;
    }

    /// `clearFreeze()`.
    pub fn clear_freeze(&mut self) {
        self.set_ticks_frozen(0);
    }

    /// `getTicksRequiredToFreeze()`.
    pub fn get_ticks_required_to_freeze(&self) -> i32 {
        140
    }

    /// `setIsInPowderSnow(boolean)`.
    pub fn set_is_in_powder_snow(&mut self, is_in_powder_snow: bool) {
        self.is_in_powder_snow = is_in_powder_snow;
    }

    /// `Entity.makeStuckInBlock(BlockState, Vec3)`: its fall over, and its next move slowed by the
    /// multiplier (`move`).
    pub fn make_stuck_in_block(&mut self, speed_multiplier: Vec3) {
        self.reset_fall_distance();
        self.stuck_speed_multiplier = speed_multiplier;
    }

    /// `addMovementThisTick(Movement)`: past 100, the first two become one.
    fn add_movement_this_tick(&mut self, movement: Movement) {
        if self.movement_this_tick.len() >= 100 {
            let first = self
                .movement_this_tick
                .pop_front()
                .expect("there are 100 moves");
            let second = self
                .movement_this_tick
                .pop_front()
                .expect("there are 100 moves");
            let combined = Movement::new(first.from, second.to);
            self.movement_this_tick.push_front(combined);
        }

        self.movement_this_tick.push_back(movement);
    }

    /// `collidedWithFluid(FluidState, BlockPos, Vec3, Vec3)`: whether its box going from `from`
    /// to `to` touches the fluid's.
    pub fn collided_with_fluid(
        &self,
        level: &dyn Level,
        fluid_state: FluidState,
        block_pos: &BlockPos,
        from: &Vec3,
        to: &Vec3,
    ) -> bool {
        fluid_state
            .get_aabb(level, block_pos)
            .is_some_and(|fluid_aabb| self.collided_with_shape_moving_from(from, to, &[fluid_aabb]))
    }

    /// `collidedWithShapeMovingFrom(Vec3, Vec3, List)`: whether its box going from `from` to `to`
    /// touches one of the boxes.
    pub fn collided_with_shape_moving_from(&self, from: &Vec3, to: &Vec3, aabbs: &[AABB]) -> bool {
        let bounding_box_at_from = self.make_bounding_box_at(from);
        let travel_vector = to.subtract(from);
        bounding_box_at_from.collided_along_vector(&travel_vector, aabbs)
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    /// `getType()`.
    pub fn get_type(&self) -> EntityType {
        self.r#type
    }

    pub fn get_uuid(&self) -> UUID {
        self.uuid
    }

    pub fn set_uuid(&mut self, uuid: UUID) {
        self.uuid = uuid;
    }

    /// `snapTo(double, double, double, float, float)`: there, looking that way, as if it had been
    /// there the tick before too.
    pub fn snap_to(&mut self, x: f64, y: f64, z: f64, y_rot: f32, x_rot: f32) {
        self.set_pos_raw(x, y, z);
        self.set_y_rot(y_rot);
        self.set_x_rot(x_rot);
        self.set_old_pos_and_rot();
        self.reapply_position();
    }

    /// `recreateFromPacket(ClientboundAddEntityPacket)`: where, which way and how fast the server
    /// says it is, with its id and UUID. `LivingEntity`'s override, which snaps with `absSnapTo`
    /// and sets the head's and body's rotation, comes with the living classes: the position is the
    /// same either way.
    pub fn recreate_from_packet(&mut self, packet: &ClientboundAddEntityPacket) {
        let x = packet.get_x();
        let y = packet.get_y();
        let z = packet.get_z();
        self.sync_packet_position_codec(x, y, z);
        self.snap_to(x, y, z, packet.get_y_rot(), packet.get_x_rot());
        self.set_id(packet.get_id());
        self.set_uuid(packet.get_uuid());
        self.set_delta_movement(packet.get_movement());
    }

    /// `absSnapTo(double, double, double, float, float)`: `snapTo` within the world's thirty
    /// million blocks, the rotations wrapped and clamped first.
    pub fn abs_snap_to(&mut self, x: f64, y: f64, z: f64, y_rot: f32, x_rot: f32) {
        let cx = Mth::clamp_f64(x, -3.0E7, 3.0E7);
        let cz = Mth::clamp_f64(z, -3.0E7, 3.0E7);
        self.old_position = Vec3::new(cx, y, cz);
        self.set_pos(cx, y, cz);
        self.set_y_rot(y_rot % 360.0);
        self.set_x_rot(Mth::clamp_f32(x_rot, -90.0, 90.0) % 360.0);
        self.y_rot_o = self.get_y_rot();
        self.x_rot_o = self.get_x_rot();
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn get_x(&self) -> f64 {
        self.position.x
    }

    pub fn get_y(&self) -> f64 {
        self.position.y
    }

    pub fn get_z(&self) -> f64 {
        self.position.z
    }

    pub fn block_position(&self) -> BlockPos {
        self.block_position
    }

    pub fn old_position(&self) -> Vec3 {
        self.old_position
    }

    pub fn get_delta_movement(&self) -> Vec3 {
        self.delta_movement
    }

    pub fn set_delta_movement(&mut self, delta_movement: Vec3) {
        self.delta_movement = delta_movement;
    }

    pub fn set_delta_movement_xyz(&mut self, xd: f64, yd: f64, zd: f64) {
        self.set_delta_movement(Vec3::new(xd, yd, zd));
    }

    /// `lerpMotion(Vec3)`: the movement the server says it has.
    pub fn lerp_motion(&mut self, movement: Vec3) {
        self.set_delta_movement(movement);
    }

    pub fn add_delta_movement(&mut self, momentum: &Vec3) {
        self.set_delta_movement(self.get_delta_movement().add(momentum));
    }

    pub fn get_y_rot(&self) -> f32 {
        self.y_rot
    }

    /// Java's `setYRot` refuses an infinite or NaN rotation, with a warning.
    pub fn set_y_rot(&mut self, y_rot: f32) {
        if y_rot.is_finite() {
            self.y_rot = y_rot;
        } else {
            eprintln!("Invalid entity rotation: {y_rot}, discarding.");
        }
    }

    pub fn get_x_rot(&self) -> f32 {
        self.x_rot
    }

    /// `turn`: the mouse's turn, in degrees times 0.15, the old rotation turned with it so the
    /// frame between ticks doesn't swing back. A vehicle isn't ported.
    /// `getXRot(partialTicks)`: the pitch between ticks, the current one at a partial tick of 1.
    pub fn get_x_rot_partial(&self, partial_ticks: f32) -> f32 {
        if partial_ticks == 1.0 {
            self.get_x_rot()
        } else {
            Mth::lerp_f32(partial_ticks, self.x_rot_o, self.get_x_rot())
        }
    }

    /// `getYRot(partialTicks)`: the yaw between ticks, the current one at a partial tick of 1.
    pub fn get_y_rot_partial(&self, partial_ticks: f32) -> f32 {
        if partial_ticks == 1.0 {
            self.get_y_rot()
        } else {
            Mth::rot_lerp_f32(partial_ticks, self.y_rot_o, self.get_y_rot())
        }
    }

    /// `getEyePosition(partialTicks)`.
    pub fn get_eye_position_partial(&self, partial_ticks: f32) -> Vec3 {
        let old = self.old_position();
        Vec3::new(
            Mth::lerp_f64(f64::from(partial_ticks), old.x, self.get_x()),
            Mth::lerp_f64(f64::from(partial_ticks), old.y, self.get_y())
                + f64::from(self.get_eye_height()),
            Mth::lerp_f64(f64::from(partial_ticks), old.z, self.get_z()),
        )
    }

    pub fn turn(&mut self, xo: f64, yo: f64) {
        let x_delta = yo as f32 * 0.15;
        let y_delta = xo as f32 * 0.15;
        self.set_x_rot(self.get_x_rot() + x_delta);
        self.set_y_rot(self.get_y_rot() + y_delta);
        self.set_x_rot(Mth::clamp_f32(self.get_x_rot(), -90.0, 90.0));
        self.x_rot_o += x_delta;
        self.y_rot_o += y_delta;
        self.x_rot_o = Mth::clamp_f32(self.x_rot_o, -90.0, 90.0);
    }

    /// Java's `setXRot` refuses an infinite or NaN rotation, with a warning, and keeps the rest
    /// within straight down and straight up (`Math.clamp(xRot % 360, -90, 90)`).
    pub fn set_x_rot(&mut self, x_rot: f32) {
        if x_rot.is_finite() {
            self.x_rot = (x_rot % 360.0).clamp(-90.0, 90.0);
        } else {
            eprintln!("Invalid entity rotation: {x_rot}, discarding.");
        }
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn get_bounding_box(&self) -> AABB {
        self.bb
    }

    pub fn set_bounding_box(&mut self, bb: AABB) {
        self.bb = bb;
    }

    pub fn get_dimensions_field(&self) -> EntityDimensions {
        self.dimensions
    }

    pub fn get_bb_width(&self) -> f32 {
        self.dimensions.width()
    }

    pub fn get_bb_height(&self) -> f32 {
        self.dimensions.height()
    }

    pub fn get_eye_height(&self) -> f32 {
        self.eye_height
    }

    pub fn get_eye_y(&self) -> f64 {
        self.position.y + f64::from(self.eye_height)
    }

    pub fn main_supporting_block_pos(&self) -> Option<BlockPos> {
        self.main_supporting_block_pos
    }

    /// Moves it without its box (`setPosRaw`): the block it's in changes with it.
    pub fn set_pos_raw(&mut self, x: f64, y: f64, z: f64) {
        if self.position.x != x || self.position.y != y || self.position.z != z {
            self.position = Vec3::new(x, y, z);
            let fx = Mth::floor_f64(x);
            let fy = Mth::floor_f64(y);
            let fz = Mth::floor_f64(z);
            if fx != self.block_position.get_x()
                || fy != self.block_position.get_y()
                || fz != self.block_position.get_z()
            {
                self.block_position = BlockPos::new(fx, fy, fz);
                self.in_block_state = None;
            }
        }
    }

    /// `setPos(double, double, double)`: moves it and its box.
    pub fn set_pos(&mut self, x: f64, y: f64, z: f64) {
        self.set_pos_raw(x, y, z);
        self.bb = self.make_bounding_box();
    }

    pub fn set_pos_vec3(&mut self, pos: &Vec3) {
        self.set_pos(pos.x, pos.y, pos.z);
    }

    pub fn make_bounding_box(&self) -> AABB {
        self.make_bounding_box_at(&self.position)
    }

    pub fn make_bounding_box_at(&self, position: &Vec3) -> AABB {
        self.dimensions.make_bounding_box(position)
    }

    fn reapply_position(&mut self) {
        let position = self.position;
        self.set_pos(position.x, position.y, position.z);
    }

    /// `setOldPosAndRot()`: where it was and looked before this tick.
    pub fn set_old_pos_and_rot(&mut self) {
        self.old_position = self.position;
        self.y_rot_o = self.y_rot;
        self.x_rot_o = self.x_rot;
    }

    /// `xo`, `yo` and `zo` set, the rotations left.
    pub fn set_old_position(&mut self, position: Vec3) {
        self.old_position = position;
    }

    /// `setOldPosAndRot(Vec3, float, float)`.
    pub fn set_old_pos_and_rot_to(&mut self, position: Vec3, y_rot: f32, x_rot: f32) {
        self.old_position = position;
        self.y_rot_o = y_rot;
        self.x_rot_o = x_rot;
    }

    /// `commonTick()`, which the level runs before `tick`: the last tick's position and
    /// rotation, and the tick count.
    pub fn common_tick(&mut self) {
        self.set_old_pos_and_rot();
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    pub fn get_shared_flag(&self, flag: i32) -> bool {
        (self.shared_flags & (1 << flag)) != 0
    }

    pub fn set_shared_flag(&mut self, flag: i32, value: bool) {
        if value {
            self.shared_flags |= 1 << flag;
        } else {
            self.shared_flags &= !(1 << flag);
        }
    }

    pub fn get_pose(&self) -> Pose {
        self.pose
    }

    pub fn has_pose(&self, pose: Pose) -> bool {
        self.pose == pose
    }

    pub fn is_no_gravity(&self) -> bool {
        self.no_gravity
    }

    pub fn set_no_gravity(&mut self, no_gravity: bool) {
        self.no_gravity = no_gravity;
    }

    pub fn is_passenger(&self) -> bool {
        false
    }

    pub fn is_in_water(&self) -> bool {
        self.was_touching_water
    }

    pub fn is_in_lava(&self) -> bool {
        !self.first_tick && self.fluid_interaction.is_in_fluid(FluidTags::LAVA)
    }

    pub fn is_in_liquid(&self) -> bool {
        self.is_in_water() || self.is_in_lava()
    }

    /// `isEyeInFluid(TagKey)`: as of the last `updateFluidInteraction`.
    pub fn is_eye_in_fluid(&self, fluid: FluidTagKey) -> bool {
        self.fluid_interaction.is_eye_in_fluid(fluid)
    }

    /// `getFluidHeight(TagKey)`: how far above its feet the tag's fluid goes.
    pub fn get_fluid_height(&self, fluid: FluidTagKey) -> f64 {
        self.fluid_interaction.get_fluid_height(fluid)
    }

    pub fn set_swimming(&mut self, swimming: bool) {
        self.set_shared_flag(Entity::FLAG_SWIMMING, swimming);
    }

    pub fn get_fluid_jump_threshold(&self) -> f64 {
        if self.get_eye_height() < 0.4 {
            0.0
        } else {
            0.4
        }
    }

    pub fn is_fall_flying(&self) -> bool {
        self.get_shared_flag(Entity::FLAG_FALL_FLYING)
    }

    pub fn reset_fall_distance(&mut self) {
        self.fall_distance = 0.0;
    }

    /// The block it's in, read once a tick.
    pub fn get_in_block_state(&mut self, level: &dyn Level) -> BlockState {
        match self.in_block_state {
            Some(state) => state,
            None => {
                let state = level.get_block_state(&self.block_position);
                self.in_block_state = Some(state);
                state
            }
        }
    }

    /// `getOnPos(float)`: the block under it that decides how it moves, from the block it's
    /// supported by, at `offset` below its feet unless that's a fence, wall or gate it stands on.
    pub fn get_on_pos_offset(&self, level: &dyn Level, offset: f32) -> BlockPos {
        match self.main_supporting_block_pos {
            Some(get_on_pos) => {
                if !(offset > 1.0E-5) {
                    return get_on_pos;
                }

                let below_state = level.get_block_state(&get_on_pos);
                if (!(f64::from(offset) <= 0.5) || !below_state.is_tag(BlockTags::FENCES))
                    && !below_state.is_tag(BlockTags::WALLS)
                    && !below_state
                        .get_block()
                        .is_instance_of(&fence_gate_block::CLASS)
                {
                    get_on_pos.at_y(Mth::floor_f64(self.position.y - f64::from(offset)))
                } else {
                    get_on_pos
                }
            }
            None => {
                let x_truncated = Mth::floor_f64(self.position.x);
                let y_truncated_below = Mth::floor_f64(self.position.y - f64::from(offset));
                let z_truncated = Mth::floor_f64(self.position.z);
                BlockPos::new(x_truncated, y_truncated_below, z_truncated)
            }
        }
    }

    /// `getOnPosLegacy()`.
    pub fn get_on_pos_legacy(&self, level: &dyn Level) -> BlockPos {
        self.get_on_pos_offset(level, 0.2)
    }

    pub fn get_block_pos_below_that_affects_my_movement(&self, level: &dyn Level) -> BlockPos {
        self.get_on_pos_offset(level, 0.500001)
    }

    pub fn get_on_pos(&self, level: &dyn Level) -> BlockPos {
        self.get_on_pos_offset(level, 1.0E-5)
    }

    /// The block's jump factor here, or below when here's is 1.
    pub fn get_block_jump_factor(&self, level: &dyn Level) -> f32 {
        let jump_factor_here = level
            .get_block_state(&self.block_position())
            .get_block()
            .get_jump_factor();
        let jump_factor_below = level
            .get_block_state(&self.get_block_pos_below_that_affects_my_movement(level))
            .get_block()
            .get_jump_factor();
        if f64::from(jump_factor_here) == 1.0 {
            jump_factor_below
        } else {
            jump_factor_here
        }
    }

    pub fn get_gravity(this: &(impl EntityClass + ?Sized)) -> f64 {
        if this.entity().is_no_gravity() {
            0.0
        } else {
            this.get_default_gravity()
        }
    }
}

/// `Entity.Movement`: a move from one place to another, with the movement a collision made it of,
/// whose order of axes it's checked in (`checkInsideBlocks`).
#[derive(Clone, Copy, Debug)]
struct Movement {
    from: Vec3,
    to: Vec3,
    axis_dependent_original_movement: Option<Vec3>,
}

impl Movement {
    fn new(from: Vec3, to: Vec3) -> Movement {
        Movement {
            from,
            to,
            axis_dependent_original_movement: None,
        }
    }

    fn with_original_movement(
        from: Vec3,
        to: Vec3,
        axis_dependent_original_movement: Vec3,
    ) -> Movement {
        Movement {
            from,
            to,
            axis_dependent_original_movement: Some(axis_dependent_original_movement),
        }
    }
}

impl EntityAccess for Entity {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_uuid(&self) -> UUID {
        self.uuid
    }
}

/// Java's `Entity` methods that subclasses override or that call overridden ones. The defaults
/// are `Entity`'s.
/// An entity in its level, as its interpolation handler moves it.
struct InLevel<'a> {
    entity: &'a mut Entity,
    level: &'a dyn Level,
}

impl Interpolated for InLevel<'_> {
    fn position(&self) -> Vec3 {
        self.entity.position()
    }

    fn get_y_rot(&self) -> f32 {
        self.entity.get_y_rot()
    }

    fn get_x_rot(&self) -> f32 {
        self.entity.get_x_rot()
    }

    fn set_pos(&mut self, position: &Vec3) {
        self.entity.set_pos_vec3(position);
    }

    fn set_rot(&mut self, y_rot: f32, x_rot: f32) {
        self.entity.set_rot(y_rot, x_rot);
    }

    fn snap_to(&mut self, position: &Vec3, y_rot: f32, x_rot: f32) {
        self.entity
            .snap_to(position.x, position.y, position.z, y_rot, x_rot);
    }

    fn no_collision_at(&self, position: &Vec3) -> bool {
        let r#box = self.entity.make_bounding_box_at(position);
        level::collisions(self.level).no_collision(Some(&*self.entity), &r#box)
    }

    /// `ClientLevel.getRelativeTickSpeed()`: the tick rate over 20, at least 1. The client's tick
    /// rate manager isn't ported, so it's 20 (TESTING.md: a paced scenario's client keeps it).
    fn relative_tick_speed(&self) -> f32 {
        1.0
    }
}

/// A plain entity, of a class not ported yet: what its type says.
impl EntityClass for Entity {
    fn entity(&self) -> &Entity {
        self
    }

    fn entity_mut(&mut self) -> &mut Entity {
        self
    }

    fn get_dimensions(&self, _pose: Pose) -> EntityDimensions {
        self.r#type.get_dimensions()
    }
}

/// A Java `Entity` reference, as the level holds one: an entity of a class the port has, its
/// class's struct, or a plain `Entity` of its type for the other classes.
#[derive(Clone, Debug)]
pub enum AnyEntity {
    Entity(Entity),
    ItemEntity(ItemEntity),
    FallingBlockEntity(FallingBlockEntity),
    ArmorStand(ArmorStand),
    Pig(Pig),
}

impl AnyEntity {
    /// `EntityType.create(Level, EntitySpawnReason)`: an entity of the type's class, with the
    /// interpolation handler its class makes.
    pub fn create(r#type: EntityType) -> AnyEntity {
        let mut entity = if r#type == EntityTypes::ITEM {
            AnyEntity::ItemEntity(ItemEntity::new())
        } else if r#type == EntityTypes::FALLING_BLOCK {
            AnyEntity::FallingBlockEntity(FallingBlockEntity::new())
        } else if r#type == EntityTypes::ARMOR_STAND {
            AnyEntity::ArmorStand(ArmorStand::new())
        } else if r#type == EntityTypes::PIG {
            AnyEntity::Pig(Pig::new())
        } else {
            AnyEntity::Entity(Entity::new(r#type, r#type.get_dimensions()))
        };
        entity
            .entity_mut()
            .set_interpolation_handler(Entity::create_interpolation_handler(r#type));
        entity
    }

    pub fn entity(&self) -> &Entity {
        match self {
            AnyEntity::Entity(entity) => entity,
            AnyEntity::ItemEntity(item) => item.entity(),
            AnyEntity::FallingBlockEntity(falling_block) => falling_block.entity(),
            AnyEntity::ArmorStand(armor_stand) => armor_stand.entity(),
            AnyEntity::Pig(pig) => pig.entity(),
        }
    }

    pub fn entity_mut(&mut self) -> &mut Entity {
        match self {
            AnyEntity::Entity(entity) => entity,
            AnyEntity::ItemEntity(item) => item.entity_mut(),
            AnyEntity::FallingBlockEntity(falling_block) => falling_block.entity_mut(),
            AnyEntity::ArmorStand(armor_stand) => armor_stand.entity_mut(),
            AnyEntity::Pig(pig) => pig.entity_mut(),
        }
    }

    /// `recreateFromPacket(ClientboundAddEntityPacket)`, by its class.
    pub fn recreate_from_packet(&mut self, packet: &ClientboundAddEntityPacket) {
        match self {
            AnyEntity::FallingBlockEntity(falling_block) => {
                falling_block.recreate_from_packet(packet);
            }
            _ => self.entity_mut().recreate_from_packet(packet),
        }
    }

    /// `isRemoved()`: only a falling block of air removes itself on the client.
    pub fn is_removed(&self) -> bool {
        match self {
            AnyEntity::FallingBlockEntity(falling_block) => falling_block.is_removed(),
            _ => false,
        }
    }

    /// `ClientLevel.tickNonPassenger`'s `tick()` after its `commonTick()` (`common_tick_in`), by
    /// its class. A class not ported yet has no tick: what `Entity.baseTick` works out for it
    /// (its fluids, swimming) moves nothing. A living entity's `pushEntities` pushes what the
    /// level finds pushable by it, which on the client is the local player alone
    /// (`EntitySelector.pushableBy`): the game holds it, so it's handed in.
    pub fn tick(
        &mut self,
        level: &dyn Level,
        local_player: Option<&mut (impl LivingEntityClass + Sized)>,
    ) {
        match self {
            AnyEntity::Entity(_) => {}
            AnyEntity::ItemEntity(item) => item.tick(level),
            AnyEntity::FallingBlockEntity(falling_block) => falling_block.tick(level),
            AnyEntity::ArmorStand(armor_stand) => {
                living_entity::tick_in_level(armor_stand, level, local_player);
            }
            AnyEntity::Pig(pig) => living_entity::tick_in_level(pig, level, local_player),
        }
    }
}

impl EntityAccess for AnyEntity {
    fn get_id(&self) -> i32 {
        self.entity().get_id()
    }

    fn get_uuid(&self) -> UUID {
        self.entity().get_uuid()
    }
}

pub trait EntityClass {
    fn entity(&self) -> &Entity;

    fn entity_mut(&mut self) -> &mut Entity;

    /// The size of a pose: the type's, scaled for a living entity.
    fn get_dimensions(&self, pose: Pose) -> EntityDimensions;

    fn is_shift_key_down(&self) -> bool {
        self.entity().get_shared_flag(Entity::FLAG_SHIFT_KEY_DOWN)
    }

    fn is_descending(&self) -> bool {
        self.is_shift_key_down()
    }

    fn is_suppressing_bounce(&self) -> bool {
        self.is_shift_key_down()
    }

    fn is_crouching(&self) -> bool {
        self.entity().has_pose(Pose::CROUCHING)
    }

    fn is_sprinting(&self) -> bool {
        self.entity().get_shared_flag(Entity::FLAG_SPRINTING)
    }

    fn set_sprinting(&mut self, sprinting: bool) {
        self.entity_mut()
            .set_shared_flag(Entity::FLAG_SPRINTING, sprinting);
    }

    fn is_swimming(&self) -> bool {
        self.entity().get_shared_flag(Entity::FLAG_SWIMMING)
    }

    fn is_visually_swimming(&self) -> bool {
        self.entity().has_pose(Pose::SWIMMING)
    }

    fn is_visually_crawling(&self) -> bool {
        self.is_visually_swimming() && !self.entity().is_in_water()
    }

    fn is_pushed_by_fluid(&self) -> bool {
        true
    }

    /// `instanceof Player`.
    fn is_player(&self) -> bool {
        false
    }

    /// `instanceof LivingEntity`.
    fn is_living_entity(&self) -> bool {
        false
    }

    /// `isAlive()`: not removed, which nothing the client ticks is while it ticks.
    fn is_alive(&self) -> bool {
        true
    }

    /// `isSteppingCarefully()`: sneaking.
    fn is_stepping_carefully(&self) -> bool {
        self.is_shift_key_down()
    }

    /// `canFreeze()`.
    fn can_freeze(&self) -> bool {
        can_freeze(self)
    }

    /// `makeStuckInBlock(BlockState, Vec3)`.
    fn make_stuck_in_block(&mut self, _block_state: BlockState, speed_multiplier: Vec3) {
        self.entity_mut().make_stuck_in_block(speed_multiplier);
    }

    /// `onAboveBubbleColumn(boolean, BlockPos)`.
    fn on_above_bubble_column(&mut self, drag_down: bool, pos: &BlockPos) {
        handle_on_above_bubble_column(self, drag_down, pos);
    }

    /// `onInsideBubbleColumn(boolean)`.
    fn on_inside_bubble_column(&mut self, drag_down: bool) {
        handle_on_inside_bubble_column(self, drag_down);
    }

    /// `isUnderWater()`: its eyes were in water at the start of the tick, and it's in water.
    fn is_under_water(&self) -> bool {
        self.entity().was_eye_in_water && self.entity().is_in_water()
    }

    fn is_in_shallow_water(&self) -> bool {
        self.entity().is_in_water() && !self.is_under_water()
    }

    /// `getFluidInteractionBox()`: its box, a hair smaller. A vehicle would change its passengers',
    /// which riding would bring.
    fn get_fluid_interaction_box(&self) -> Option<AABB> {
        Some(self.entity().get_bounding_box().deflate(0.001))
    }

    fn max_up_step(&self) -> f32 {
        0.0
    }

    fn get_default_gravity(&self) -> f64 {
        0.0
    }

    fn get_effective_gravity(&self) -> f64 {
        Entity::get_gravity(self)
    }

    fn get_air_drag(&self) -> f32 {
        0.98
    }

    fn get_entity_bounciness(&self) -> f64 {
        0.0
    }

    fn is_horizontal_collision_minor(&self, _movement: &Vec3) -> bool {
        false
    }

    /// Whether this side decides where the entity goes (`isLocalInstanceAuthoritative`), and so
    /// may move it (`canSimulateMovement`).
    fn is_local_instance_authoritative(&self) -> bool {
        false
    }

    /// `getMoveSimulationType()`: which sides move it.
    fn get_move_simulation_type(&self) -> MoveSimulationType {
        MoveSimulationType::AUTHORITATIVE_SIDE
    }

    /// `canSimulateMovement()` on the client: whether its own tick may move it.
    fn can_simulate_movement(&self) -> bool {
        match self.get_move_simulation_type() {
            MoveSimulationType::SERVER_AND_CLIENT => true,
            // `isLocalInstanceAuthoritative()`, and on the client `isLocalClientAuthoritative()`,
            // which is the same there.
            MoveSimulationType::AUTHORITATIVE_SIDE
            | MoveSimulationType::AUTHORITATIVE_SIDE_AND_SERVER => {
                self.is_local_instance_authoritative()
            }
        }
    }

    /// `isPushable()`: whether other entities push it.
    fn is_pushable(&mut self, _level: &dyn Level) -> bool
    where
        Self: Sized,
    {
        false
    }

    /// `push(Entity)`: this and the other entity pushed apart.
    fn push(&mut self, level: &dyn Level, entity: &mut (impl EntityClass + Sized))
    where
        Self: Sized,
    {
        push(self, level, entity);
    }

    /// `CollisionContext.of(this)`: with the entity, as blocks' shapes ask about it (a living
    /// one's liquid shape and `canStandOnFluid` are `LivingEntity`'s, which no class ported
    /// overrides). Its hand is empty: the inventory isn't ported.
    fn collision_context(&self) -> EntityCollisionContext {
        EntityCollisionContext::new(self.is_descending(), false, self.entity().get_y(), false)
            .with_entity(CollidingEntity {
                fall_distance: self.entity().fall_distance,
                is_falling_block: self.entity().get_type() == EntityTypes::FALLING_BLOCK,
                can_walk_on_powder_snow: PowderSnowBlock::can_entity_walk_on_powder_snow(self),
                living: self.is_living_entity().then(|| LivingCollision {
                    liquid_collision_shape: Shapes::empty(),
                    can_stand_on_fluid: |_| false,
                }),
            })
    }

    fn maybe_back_off_from_edge(
        &mut self,
        _level: &dyn Level,
        delta: Vec3,
        _mover_type: MoverType,
    ) -> Vec3 {
        delta
    }

    fn check_fall_damage(
        &mut self,
        level: &dyn Level,
        ya: f64,
        on_ground: bool,
        on_state: BlockState,
        pos: BlockPos,
    ) where
        Self: Sized,
    {
        check_fall_damage(self, level, ya, on_ground, on_state, pos);
    }

    fn get_block_speed_factor(&mut self, level: &dyn Level) -> f32
    where
        Self: Sized,
    {
        get_block_speed_factor(self, level)
    }

    fn r#move(&mut self, level: &dyn Level, mover_type: MoverType, delta: Vec3)
    where
        Self: Sized,
    {
        r#move(self, level, mover_type, delta);
    }

    fn update_fluid_interaction(&mut self, level: &dyn Level) -> bool
    where
        Self: Sized,
    {
        update_fluid_interaction(self, level)
    }

    fn update_swimming(&mut self, level: &dyn Level)
    where
        Self: Sized,
    {
        update_swimming(self, level);
    }
}

/// `setPose`: a new pose resizes the entity (`onSyncedDataUpdated` of `DATA_POSE`).
pub fn set_pose(this: &mut (impl EntityClass + ?Sized), pose: Pose) {
    if this.entity().pose != pose {
        this.entity_mut().pose = pose;
        refresh_dimensions(this);
    }
}

/// The pose's size, and the box made again around the position. Java fudges the position of an
/// entity that grows on the server, not here.
pub fn refresh_dimensions(this: &mut (impl EntityClass + ?Sized)) {
    let pose = this.entity().get_pose();
    let new_dim = this.get_dimensions(pose);
    let entity = this.entity_mut();
    entity.dimensions = new_dim;
    entity.eye_height = new_dim.eye_height();
    entity.reapply_position();
}

/// `calculateViewVector(float, float)`: the unit vector it looks along, in floats.
pub fn calculate_view_vector(x_rot: f32, y_rot: f32) -> Vec3 {
    let real_x_rot = x_rot * (std::f64::consts::PI / 180.0) as f32;
    let real_y_rot = -y_rot * (std::f64::consts::PI / 180.0) as f32;
    let y_cos = Mth::cos(f64::from(real_y_rot));
    let y_sin = Mth::sin(f64::from(real_y_rot));
    let x_cos = Mth::cos(f64::from(real_x_rot));
    let x_sin = Mth::sin(f64::from(real_x_rot));
    Vec3::new(
        f64::from(y_sin * x_cos),
        f64::from(-x_sin),
        f64::from(y_cos * x_cos),
    )
}

/// `isFree(double, double, double)`: whether its box moved that far has no collisions and no
/// fluid.
pub fn is_free(
    this: &(impl EntityClass + Sized),
    level: &dyn Level,
    xa: f64,
    ya: f64,
    za: f64,
) -> bool {
    let r#box = this.entity().get_bounding_box().move_xyz(xa, ya, za);
    level::collisions(level).no_collision(Some(this), &r#box)
        && !level::contains_any_liquid(level, &r#box)
}

/// `setOnGround(boolean)`.
pub fn set_on_ground(this: &mut (impl EntityClass + Sized), level: &dyn Level, on_ground: bool) {
    this.entity_mut().on_ground = on_ground;
    check_supporting_block(this, level, on_ground, None);
}

/// `setOnGroundWithMovement(boolean, boolean, Vec3)`.
pub fn set_on_ground_with_movement(
    this: &mut (impl EntityClass + Sized),
    level: &dyn Level,
    on_ground: bool,
    horizontal_collision: bool,
    movement: &Vec3,
) {
    this.entity_mut().on_ground = on_ground;
    this.entity_mut().horizontal_collision = horizontal_collision;
    check_supporting_block(this, level, on_ground, Some(movement));
}

/// The block an entity on the ground stands on: under its box, or, when nothing is, under where
/// its box was before this movement.
fn check_supporting_block(
    this: &mut (impl EntityClass + Sized),
    level: &dyn Level,
    on_ground: bool,
    movement: Option<&Vec3>,
) {
    if on_ground {
        let bounding_box = this.entity().get_bounding_box();
        let test_area = AABB::new(
            bounding_box.min_x,
            bounding_box.min_y - 1.0E-6,
            bounding_box.min_z,
            bounding_box.max_x,
            bounding_box.min_y,
            bounding_box.max_z,
        );
        let mut supporting_block = level::collisions(level).find_supporting_block(this, &test_area);
        if supporting_block.is_some() || this.entity().on_ground_no_blocks {
            this.entity_mut().main_supporting_block_pos = supporting_block;
        } else if let Some(movement) = movement {
            let on_ground_collision_test_area = test_area.move_xyz(-movement.x, 0.0, -movement.z);
            supporting_block = level::collisions(level)
                .find_supporting_block(this, &on_ground_collision_test_area);
            this.entity_mut().main_supporting_block_pos = supporting_block;
        }

        this.entity_mut().on_ground_no_blocks = supporting_block.is_none();
    } else {
        let entity = this.entity_mut();
        entity.on_ground_no_blocks = false;
        entity.main_supporting_block_pos = None;
    }
}

/// `move(MoverType, Vec3)`: moves the entity as far as its collisions let it, sets the collision
/// flags and whether it's on the ground, and slows it by the block's speed factor.
pub fn r#move(
    this: &mut (impl EntityClass + Sized),
    level: &dyn Level,
    mover_type: MoverType,
    delta: Vec3,
) {
    let mut delta = delta;
    if this.entity().no_physics {
        let entity = this.entity_mut();
        entity.set_pos(
            entity.get_x() + delta.x,
            entity.get_y() + delta.y,
            entity.get_z() + delta.z,
        );
        entity.horizontal_collision = false;
        entity.vertical_collision = false;
        entity.vertical_collision_below = false;
        entity.minor_horizontal_collision = false;
        return;
    }

    if mover_type == MoverType::PISTON {
        delta = limit_piston_movement(this.entity_mut(), level, &delta);
        if delta == Vec3::ZERO {
            return;
        }
    }

    if this.entity().stuck_speed_multiplier.length_sqr() > 1.0E-7 {
        if mover_type != MoverType::PISTON {
            delta = delta.multiply(&this.entity().stuck_speed_multiplier);
        }
        let entity = this.entity_mut();
        entity.stuck_speed_multiplier = Vec3::ZERO;
        entity.set_delta_movement(Vec3::ZERO);
    }

    delta = this.maybe_back_off_from_edge(level, delta, mover_type);
    let movement = collide(this, level, &delta);
    let movement_length = movement.length_sqr();
    if movement_length > 1.0E-7 || delta.length_sqr() - movement_length < 1.0E-7 {
        if this.entity().fall_distance != 0.0 && movement_length >= 1.0 {
            // What a fast fall passes through that ends it: water, ladders, cobwebs.
            let check_distance = Math::min_f64(movement.length(), 8.0);
            let check_to = this
                .entity()
                .position()
                .add(&movement.normalize().scale(check_distance));
            let hit_result = block_getter::clip(
                level::collisions(level),
                &ClipContext::new(
                    this.entity().position(),
                    check_to,
                    clip_context::Block::FALLDAMAGE_RESETTING,
                    clip_context::Fluid::WATER,
                    this.collision_context(),
                    this.is_player(),
                ),
            );
            if hit_result.get_type() != Type::MISS {
                this.entity_mut().reset_fall_distance();
            }
        }

        let pos = this.entity().position();
        let new_position = pos.add(&movement);
        this.entity_mut()
            .add_movement_this_tick(Movement::with_original_movement(pos, new_position, delta));
        this.entity_mut().set_pos_vec3(&new_position);
    }

    let x_collision = !Mth::equal_f64(delta.x, movement.x);
    let z_collision = !Mth::equal_f64(delta.z, movement.z);
    this.entity_mut().horizontal_collision = x_collision || z_collision;
    let moved_vertically = delta.y.abs() > 0.0;
    if moved_vertically || this.is_local_instance_authoritative() {
        let vertical_collision = delta.y != movement.y;
        this.entity_mut().vertical_collision = vertical_collision;
        this.entity_mut().vertical_collision_below = vertical_collision && delta.y < 0.0;
        let horizontal_collision = this.entity().horizontal_collision;
        let vertical_collision_below = this.entity().vertical_collision_below;
        set_on_ground_with_movement(
            this,
            level,
            vertical_collision_below,
            horizontal_collision,
            &movement,
        );
    }

    if this.entity().horizontal_collision {
        let minor = this.is_horizontal_collision_minor(&movement);
        this.entity_mut().minor_horizontal_collision = minor;
    } else {
        this.entity_mut().minor_horizontal_collision = false;
    }

    let effect_pos = this.entity().get_on_pos_legacy(level);
    let effect_state = level.get_block_state(&effect_pos);
    if this.is_local_instance_authoritative() {
        let on_ground = this.entity().on_ground();
        this.check_fall_damage(level, movement.y, on_ground, effect_state, effect_pos);
    }

    // `isRemoved()`: nothing the client moves removes itself in its move.
    if this.can_simulate_movement()
        && (moved_vertically && this.entity().vertical_collision
            || this.entity().horizontal_collision)
    {
        restitute_movement_after_collisions(
            this,
            effect_state,
            x_collision,
            z_collision,
            &movement,
        );
    }

    let block_speed_factor = this.get_block_speed_factor(level);
    let delta_movement = this.entity().get_delta_movement().multiply_xyz(
        f64::from(block_speed_factor),
        1.0,
        f64::from(block_speed_factor),
    );
    this.entity_mut().set_delta_movement(delta_movement);
}

/// `limitPistonMovement`: what pistons push it by along one axis, held to 0.51 blocks either way
/// a game tick, nothing when that leaves less than 1.0E-5.
fn limit_piston_movement(entity: &mut Entity, level: &dyn Level, vec: &Vec3) -> Vec3 {
    if vec.length_sqr() <= 1.0E-7 {
        return *vec;
    }

    let current_game_time = level.get_game_time();
    if current_game_time != entity.piston_deltas_game_time {
        entity.piston_deltas = [0.0; 3];
        entity.piston_deltas_game_time = current_game_time;
    }

    if vec.x != 0.0 {
        let xa = entity.apply_piston_movement_restriction(Axis::X, vec.x);
        if xa.abs() <= f64::from(1.0E-5_f32) {
            Vec3::ZERO
        } else {
            Vec3::new(xa, 0.0, 0.0)
        }
    } else if vec.y != 0.0 {
        let ya = entity.apply_piston_movement_restriction(Axis::Y, vec.y);
        if ya.abs() <= f64::from(1.0E-5_f32) {
            Vec3::ZERO
        } else {
            Vec3::new(0.0, ya, 0.0)
        }
    } else if vec.z != 0.0 {
        let za = entity.apply_piston_movement_restriction(Axis::Z, vec.z);
        if za.abs() <= f64::from(1.0E-5_f32) {
            Vec3::ZERO
        } else {
            Vec3::new(0.0, 0.0, za)
        }
    } else {
        Vec3::ZERO
    }
}

/// Stops the movement along each axis it collided on, bouncing by the restitution, the entity's
/// own or, landing, the block's. A bounce calls the block's `bounceOn`, which changes nothing the
/// client has (the shelf mushroom's plays a sound, which isn't ported), with a game event and
/// `syncPosition`, the server's.
fn restitute_movement_after_collisions(
    this: &mut (impl EntityClass + Sized),
    effect_state: BlockState,
    x_collision: bool,
    z_collision: bool,
    movement: &Vec3,
) {
    let mut restitution = if this.is_suppressing_bounce() {
        0.0
    } else {
        this.get_entity_bounciness()
    };
    let current_movement = this.entity().get_delta_movement();
    let mut movement_after_bounce = current_movement;
    if x_collision {
        movement_after_bounce =
            movement_after_bounce.with(Axis::X, -current_movement.x * restitution);
    }

    if z_collision {
        movement_after_bounce =
            movement_after_bounce.with(Axis::Z, -current_movement.z * restitution);
    }

    if this.entity().vertical_collision {
        if this.entity().vertical_collision_below {
            restitution = if !(-current_movement.y <= this.get_effective_gravity())
                && !this.is_suppressing_bounce()
                && !effect_state.is_tag(BlockTags::SUPPRESSES_BOUNCE)
            {
                Math::max_f64(
                    restitution,
                    get_block_bounciness(this, effect_state.get_block()),
                )
            } else {
                0.0
            };
        }

        let (gravity_compensation, effective_drag) = if restitution > 0.0 {
            let portion_with_movement = movement.y / current_movement.y;
            (
                portion_with_movement * this.get_effective_gravity(),
                Mth::lerp_f64(portion_with_movement, 1.0, f64::from(this.get_air_drag())),
            )
        } else {
            (0.0, 1.0)
        };

        movement_after_bounce = movement_after_bounce.with(
            Axis::Y,
            (gravity_compensation - current_movement.y) * effective_drag * restitution,
        );
    }

    this.entity_mut().set_delta_movement(movement_after_bounce);
}

/// `getBlockBounciness(Block)`: the block's restitution, less for what isn't living.
fn get_block_bounciness(this: &(impl EntityClass + Sized), on_block: Block) -> f64 {
    let mut block_bounciness = on_block.get_bounce_restitution();
    if !this.is_living_entity() {
        block_bounciness *= 0.8;
    }

    f64::from(block_bounciness)
}

/// `Entity.getBlockSpeedFactor()`: the block's here, or below when here's is 1, except in water
/// and bubble columns.
pub fn get_block_speed_factor(this: &mut (impl EntityClass + Sized), level: &dyn Level) -> f32 {
    let state = level.get_block_state(&this.entity().block_position());
    let speed_factor_here = state.get_block().get_speed_factor();
    if !state.is(Blocks::WATER) && !state.is(Blocks::BUBBLE_COLUMN) {
        if f64::from(speed_factor_here) == 1.0 {
            level
                .get_block_state(
                    &this
                        .entity()
                        .get_block_pos_below_that_affects_my_movement(level),
                )
                .get_block()
                .get_speed_factor()
        } else {
            speed_factor_here
        }
    } else {
        speed_factor_here
    }
}

/// `Entity.checkFallDamage`: the fall distance grows while falling, and goes back to 0 on the
/// ground. Landing calls the block's `fallOn`, which changes nothing the client has (`Block`), and
/// makes a game event, the server's.
pub fn check_fall_damage(
    this: &mut (impl EntityClass + ?Sized),
    _level: &dyn Level,
    ya: f64,
    on_ground: bool,
    _on_state: BlockState,
    _pos: BlockPos,
) {
    if !this.entity().is_in_water() && ya < 0.0 {
        this.entity_mut().fall_distance -= f64::from(ya as f32);
    }

    if on_ground {
        this.entity_mut().reset_fall_distance();
    }
}

/// `collide(Vec3)`: how far the entity can move, stepping up onto what it walks into when it's
/// on the ground (or lands this move).
fn collide(this: &mut (impl EntityClass + Sized), level: &dyn Level, movement: &Vec3) -> Vec3 {
    let aabb = this.entity().get_bounding_box();
    let max_up_step = this.max_up_step();
    let entity_colliders = level::collisions(level).get_entity_collisions(
        Some(this),
        &aabb
            .expand_towards(movement)
            .expand_towards_xyz(0.0, f64::from(max_up_step), 0.0),
    );
    let movement_step = if movement.length_sqr() == 0.0 {
        *movement
    } else {
        collide_bounding_box(
            Some(this),
            movement,
            &aabb,
            level::collisions(level),
            &entity_colliders,
        )
    };
    let x_collision = movement.x != movement_step.x;
    let y_collision = movement.y != movement_step.y;
    let z_collision = movement.z != movement_step.z;
    let on_ground_after_collision = y_collision && movement.y < 0.0;
    if max_up_step > 0.0
        && (on_ground_after_collision || this.entity().on_ground())
        && (x_collision || z_collision)
    {
        let grounded_aabb = if on_ground_after_collision {
            aabb.move_xyz(0.0, movement_step.y, 0.0)
        } else {
            aabb
        };
        let mut step_up_aabb =
            grounded_aabb.expand_towards_xyz(movement.x, f64::from(max_up_step), movement.z);
        if !on_ground_after_collision {
            step_up_aabb = step_up_aabb.expand_towards_xyz(0.0, f64::from(-1.0E-5_f32), 0.0);
        }

        let colliders = collect_colliders_ignoring_world_border(
            Some(this),
            level::collisions(level),
            &entity_colliders,
            &step_up_aabb,
        );
        let step_height_to_skip = movement_step.y as f32;
        let candidate_step_up_heights = collect_candidate_step_up_heights(
            &grounded_aabb,
            &colliders,
            max_up_step,
            step_height_to_skip,
        );

        for candidate_step_up_height in candidate_step_up_heights {
            let step_from_ground = collide_with_shapes(
                &Vec3::new(movement.x, f64::from(candidate_step_up_height), movement.z),
                &grounded_aabb,
                &colliders,
            );
            if step_from_ground.horizontal_distance_sqr() > movement_step.horizontal_distance_sqr()
            {
                let distance_to_ground = aabb.min_y - grounded_aabb.min_y;
                return step_from_ground.subtract_xyz(0.0, distance_to_ground, 0.0);
            }
        }
    }

    movement_step
}

/// The heights a box could step up to: the shapes' grid lines from its bottom up to the step
/// height, as floats, without the height it's at already, each once (by bits) and sorted.
pub fn collect_candidate_step_up_heights(
    bounding_box: &AABB,
    colliders: &[VoxelShape],
    max_step_height: f32,
    step_height_to_skip: f32,
) -> Vec<f32> {
    // A FloatArraySet: no two the same by Float.floatToIntBits.
    let mut candidates: Vec<f32> = Vec::with_capacity(4);
    for collider in colliders {
        let coords = collider.get_coords(Axis::Y);
        for index in 0..coords.size() {
            let coord = coords.get_double(index);
            let relative_coord = (coord - bounding_box.min_y) as f32;
            if !(relative_coord < 0.0) && relative_coord != step_height_to_skip {
                if relative_coord > max_step_height {
                    break;
                }

                let bits = Float::float_to_int_bits(relative_coord);
                if !candidates
                    .iter()
                    .any(|&candidate| Float::float_to_int_bits(candidate) == bits)
                {
                    candidates.push(relative_coord);
                }
            }
        }
    }

    // FloatArrays.unstableSort, by Float.compare.
    candidates.sort_by(|a, b| a.total_cmp(b));
    candidates
}

/// `collideBoundingBox(Entity, Vec3, AABB, Level, List)`: the movement the box can make through
/// the level's shapes and the entities' along it.
pub fn collide_bounding_box(
    source: Option<&dyn EntityClass>,
    movement: &Vec3,
    bounding_box: &AABB,
    level: &dyn CollisionGetter,
    entity_colliders: &[VoxelShape],
) -> Vec3 {
    let colliders = collect_colliders_ignoring_world_border(
        source,
        level,
        entity_colliders,
        &bounding_box.expand_towards(movement),
    );
    collide_with_shapes(movement, bounding_box, &colliders)
}

/// The entities' shapes, the border's when the entity is near it, and the blocks'.
fn collect_colliders_ignoring_world_border(
    source: Option<&dyn EntityClass>,
    level: &dyn CollisionGetter,
    entity_colliders: &[VoxelShape],
    bounding_box: &AABB,
) -> Vec<VoxelShape> {
    let mut colliders = Vec::with_capacity(entity_colliders.len() + 1);
    colliders.extend_from_slice(entity_colliders);
    let world_border = level.get_world_border();
    let is_entity_inside_close_to_border = source.is_some_and(|source| {
        world_border.is_inside_close_to_border(
            source.entity().get_x(),
            source.entity().get_z(),
            bounding_box,
        )
    });
    if is_entity_inside_close_to_border {
        colliders.push(world_border.get_collision_shape());
    }

    colliders.extend(level.get_block_collisions(source, bounding_box));
    colliders
}

/// Moves the box through the shapes one axis at a time, the largest movement's axis last but y
/// first (`Direction.axisStepOrder`).
pub fn collide_with_shapes(movement: &Vec3, bounding_box: &AABB, shapes: &[VoxelShape]) -> Vec3 {
    if shapes.is_empty() {
        return *movement;
    }

    let mut resolved_movement = Vec3::ZERO;
    for axis in Direction::axis_step_order(movement) {
        let axis_movement = movement.get(axis);
        if axis_movement != 0.0 {
            let collision = Shapes::collide(
                axis,
                &bounding_box.r#move(&resolved_movement),
                shapes,
                axis_movement,
            );
            resolved_movement = resolved_movement.with(axis, collision);
        }
    }
    resolved_movement
}

/// `applyGravity()`.
pub fn apply_gravity(this: &mut (impl EntityClass + ?Sized)) {
    let gravity = Entity::get_gravity(this);
    if gravity != 0.0 {
        let delta = this.entity().get_delta_movement();
        this.entity_mut()
            .set_delta_movement(delta.add_xyz(0.0, -gravity, 0.0));
    }
}

/// `Entity.push(Entity)`: each pushed away from the other by a twentieth, less when they're over
/// a block apart, when it's pushable; nothing rides either (riding isn't ported).
pub fn push(
    this: &mut (impl EntityClass + Sized),
    level: &dyn Level,
    entity: &mut (impl EntityClass + Sized),
) {
    if !entity.entity().no_physics && !this.entity().no_physics {
        let mut xa = entity.entity().get_x() - this.entity().get_x();
        let mut za = entity.entity().get_z() - this.entity().get_z();
        let mut dd = Mth::abs_max_f64(xa, za);
        if dd >= f64::from(0.01_f32) {
            dd = dd.sqrt();
            xa /= dd;
            za /= dd;
            let mut pow = 1.0 / dd;
            if pow > 1.0 {
                pow = 1.0;
            }

            xa *= pow;
            za *= pow;
            xa *= f64::from(0.05_f32);
            za *= f64::from(0.05_f32);
            if this.is_pushable(level) {
                this.entity_mut().push_xyz(-xa, 0.0, -za);
            }

            if entity.is_pushable(level) {
                entity.entity_mut().push_xyz(xa, 0.0, za);
            }
        }
    }
}

/// `moveRelative(float, Vec3)`: the input, turned by where the entity looks, added to its
/// movement.
pub fn move_relative(this: &mut (impl EntityClass + ?Sized), speed: f32, input: &Vec3) {
    let delta = get_input_vector(input, speed, this.entity().get_y_rot());
    this.entity_mut().add_delta_movement(&delta);
}

/// The input scaled to `speed` (normalized when it's longer than 1) and turned by `y_rot`; none
/// when it's under 1.0E-7 long squared.
pub fn get_input_vector(input: &Vec3, speed: f32, y_rot: f32) -> Vec3 {
    let length = input.length_sqr();
    if length < 1.0E-7 {
        return Vec3::ZERO;
    }

    let movement = if length > 1.0 {
        input.normalize()
    } else {
        *input
    }
    .scale(f64::from(speed));
    let angle = y_rot * (std::f64::consts::PI / 180.0) as f32;
    let sin = Mth::sin(f64::from(angle));
    let cos = Mth::cos(f64::from(angle));
    Vec3::new(
        movement.x * f64::from(cos) - movement.z * f64::from(sin),
        movement.y,
        movement.z * f64::from(cos) + movement.x * f64::from(sin),
    )
}

/// `applyEffectsFromBlocks()`: the effects of the blocks it went through this tick, over its
/// moves (from where it was to where it is when it made none, and on to where it is when
/// something moved it after its last), which it keeps (`finalMovementsThisTick`).
pub fn apply_effects_from_blocks(this: &mut (impl EntityClass + Sized), level: &dyn Level) {
    let entity = this.entity_mut();
    entity.final_movements_this_tick.clear();
    let movements: Vec<Movement> = entity.movement_this_tick.drain(..).collect();
    entity.final_movements_this_tick.extend(movements);
    let position = entity.position();
    match entity.final_movements_this_tick.last() {
        None => {
            let old_position = entity.old_position();
            entity
                .final_movements_this_tick
                .push(Movement::new(old_position, position));
        }
        Some(last) if last.to.distance_to_sqr(&position) > f64::from(9.9999994E-11_f32) => {
            let last_to = last.to;
            entity
                .final_movements_this_tick
                .push(Movement::new(last_to, position));
        }
        Some(_) => {}
    }

    let movements = this.entity().final_movements_this_tick.clone();
    apply_effects_from_blocks_over(this, level, &movements);
}

/// `applyEffectsFromBlocksForLastMovements()`: the same over the moves they were last applied
/// over.
pub fn apply_effects_from_blocks_for_last_movements(
    this: &mut (impl EntityClass + Sized),
    level: &dyn Level,
) {
    let movements = this.entity().final_movements_this_tick.clone();
    apply_effects_from_blocks_over(this, level, &movements);
}

/// `applyEffectsFromBlocks(List)`: the block it stands on (`stepOn`), then the ones it's inside.
/// Java's collector is a field the entity reuses; each application empties it.
fn apply_effects_from_blocks_over(
    this: &mut (impl EntityClass + Sized),
    level: &dyn Level,
    movements: &[Movement],
) {
    if is_affected_by_blocks(this) {
        if this.entity().on_ground() {
            let effect_pos = this.entity().get_on_pos_legacy(level);
            let effect_state = level.get_block_state(&effect_pos);
            effect_state
                .get_block()
                .step_on(level, &effect_pos, effect_state, this);
        }

        let mut inside_effect_collector = StepBasedCollector::new();
        check_inside_blocks(this, level, movements, &mut inside_effect_collector);
        inside_effect_collector.apply_and_clear(this, level);
    }
}

/// `isAffectedByBlocks()`: not removed (see `is_alive`), and with physics.
fn is_affected_by_blocks(this: &(impl EntityClass + Sized)) -> bool {
    !this.entity().no_physics
}

/// `checkInsideBlocks(List, StepBasedCollector)`: along each move, axis by axis in the order its
/// collision took them, at most 16 steps each, the blocks its box goes through; past that, the
/// ones where the move ends. Each block once (`visitedBlocks`, a `LongOpenHashSet` it's only asked
/// what it holds).
fn check_inside_blocks(
    this: &mut (impl EntityClass + Sized),
    level: &dyn Level,
    movements: &[Movement],
    effect_collector: &mut StepBasedCollector,
) {
    if is_affected_by_blocks(this) {
        let mut visited_blocks = HashSet::new();

        for movement in movements {
            let mut pos = movement.from;
            let delta = movement.to.subtract(&movement.from);
            let mut max_movement_iterations = 16;
            match movement.axis_dependent_original_movement {
                Some(original_movement) if delta.length_sqr() > 0.0 => {
                    for axis in Direction::axis_step_order(&original_movement) {
                        let axis_move = delta.get(axis);
                        if axis_move != 0.0 {
                            let to = pos.relative(axis.get_positive(), axis_move);
                            max_movement_iterations -= check_inside_blocks_between(
                                this,
                                level,
                                &pos,
                                &to,
                                effect_collector,
                                &mut visited_blocks,
                                max_movement_iterations,
                            );
                            pos = to;
                        }
                    }
                }
                _ => {
                    max_movement_iterations -= check_inside_blocks_between(
                        this,
                        level,
                        &movement.from,
                        &movement.to,
                        effect_collector,
                        &mut visited_blocks,
                        16,
                    );
                }
            }

            if max_movement_iterations <= 0 {
                check_inside_blocks_between(
                    this,
                    level,
                    &movement.to,
                    &movement.to,
                    effect_collector,
                    &mut visited_blocks,
                    1,
                );
            }
        }
    }
}

/// `checkInsideBlocks(Vec3, Vec3, StepBasedCollector, LongSet, int)`: the blocks its box, a hair
/// smaller, goes through from `from` to `to`, up to the step limit, each block it's inside (its
/// inside shape the whole block or touched on the way) or whose fluid it touches, the first time:
/// the block's `entityInside` (precise when the box ends in the block or moved about a block or
/// more) and the fluid's, each in its step. How many steps it went. The server's debug view of
/// the blocks met isn't the client's, and `onInsideBlock` is only ender pearls'.
fn check_inside_blocks_between(
    this: &mut (impl EntityClass + Sized),
    level: &dyn Level,
    from: &Vec3,
    to: &Vec3,
    effect_collector: &mut StepBasedCollector,
    visited_blocks: &mut HashSet<i64>,
    max_movement_iterations: i32,
) -> i32 {
    let deflated_bounding_box_at_target = this
        .entity()
        .make_bounding_box_at(to)
        .deflate(f64::from(1.0E-5_f32));
    let moved_far = from.distance_to_sqr(to) > Mth::square_f64(0.9999900000002526);
    let mut iterations = 0;
    block_getter::for_each_block_intersected_between(
        from,
        to,
        &deflated_bounding_box_at_target,
        &mut |block_intersection, iteration| {
            if !this.is_alive() {
                return false;
            }

            if iteration >= max_movement_iterations {
                return false;
            }

            iterations = iteration;
            let state = level.get_block_state(block_intersection);
            if state.is_air() {
                return true;
            }

            let intersect_shape =
                state.get_entity_inside_collision_shape(level, block_intersection, &*this);
            let inside_block = intersect_shape == Shapes::block()
                || this.entity().collided_with_shape_moving_from(
                    from,
                    to,
                    &intersect_shape
                        .r#move(&Vec3::from_vec3i(block_intersection))
                        .to_aabbs(),
                );
            let inside_fluid = this.entity().collided_with_fluid(
                level,
                state.get_fluid_state(),
                block_intersection,
                from,
                to,
            );
            if (inside_block || inside_fluid) && visited_blocks.insert(block_intersection.as_long())
            {
                if inside_block {
                    let is_precise = moved_far
                        || deflated_bounding_box_at_target.intersects_block_pos(block_intersection);
                    effect_collector.advance_step(iteration);
                    state.entity_inside(
                        level,
                        block_intersection,
                        &mut *this,
                        &mut *effect_collector,
                        is_precise,
                    );
                }

                if inside_fluid {
                    effect_collector.advance_step(iteration);
                    state.get_fluid_state().entity_inside(
                        level,
                        block_intersection,
                        &mut *this,
                        &mut *effect_collector,
                    );
                }
            }

            true
        },
    );
    iterations + 1
}

/// `Entity.canFreeze()`: unless its type is immune.
pub fn can_freeze(this: &(impl EntityClass + ?Sized)) -> bool {
    !this
        .entity()
        .get_type()
        .is_tag(EntityTypeTags::FREEZE_IMMUNE_ENTITY_TYPES)
}

/// `handleOnAboveBubbleColumn(Entity, boolean, BlockPos)`: pulled down or pushed up, harder than
/// inside one, where it moves itself. The bubbles it makes are the server's.
pub fn handle_on_above_bubble_column(
    entity: &mut (impl EntityClass + ?Sized),
    drag_down: bool,
    _pos: &BlockPos,
) {
    if entity.can_simulate_movement() {
        let movement = entity.entity().get_delta_movement();
        let yd = if drag_down {
            Math::max_f64(-0.9, movement.y - 0.03)
        } else {
            Math::min_f64(1.8, movement.y + 0.1)
        };

        entity
            .entity_mut()
            .set_delta_movement_xyz(movement.x, yd, movement.z);
    }
}

/// `handleOnInsideBubbleColumn(Entity, boolean)`: pulled down or pushed up where it moves itself,
/// and its fall over.
pub fn handle_on_inside_bubble_column(entity: &mut (impl EntityClass + ?Sized), drag_down: bool) {
    if entity.can_simulate_movement() {
        let movement = entity.entity().get_delta_movement();
        let yd = if drag_down {
            Math::max_f64(-0.3, movement.y - 0.03)
        } else {
            Math::min_f64(0.7, movement.y + 0.06)
        };

        entity
            .entity_mut()
            .set_delta_movement_xyz(movement.x, yd, movement.z);
    }

    entity.entity_mut().reset_fall_distance();
}

/// `Entity.baseTick()`'s part that reaches an entity's movement: the block it's in is read again,
/// the powder snow and the fluids it's in, whether it swims, and the first tick ending.
pub fn base_tick(this: &mut (impl EntityClass + Sized), level: &dyn Level) {
    let entity = this.entity_mut();
    entity.in_block_state = None;
    entity.was_in_powder_snow = entity.is_in_powder_snow;
    entity.is_in_powder_snow = false;
    entity.was_eye_in_water = entity.is_eye_in_fluid(FluidTags::WATER);
    this.update_fluid_interaction(level);
    this.update_swimming(level);
    if this.entity().is_in_lava() {
        this.entity_mut().fall_distance *= 0.5;
    }
    this.entity_mut().first_tick = false;
}

/// `Entity.updateFluidInteraction()`: the fluids it's in, which stop its fall in water, and their
/// currents pushing it when it's pushed by fluids. Whether it's in any.
pub fn update_fluid_interaction(this: &mut (impl EntityClass + Sized), level: &dyn Level) -> bool {
    let ignore_current = !this.is_pushed_by_fluid();
    let mut fluid_interaction = std::mem::replace(
        &mut this.entity_mut().fluid_interaction,
        EntityFluidInteraction::new(&[]),
    );
    let in_any_fluid = fluid_interaction.update(this, level, ignore_current);
    let in_water = fluid_interaction.is_in_fluid(FluidTags::WATER);
    let in_lava = fluid_interaction.is_in_fluid(FluidTags::LAVA);
    if in_water {
        this.entity_mut().reset_fall_distance();
        // A splash when it wasn't in water and it's not the first tick: a sound and particles.
    }

    this.entity_mut().was_touching_water = in_water;
    if this.is_pushed_by_fluid() {
        if in_water {
            fluid_interaction.apply_current_to(FluidTags::WATER, this, 0.014);
        }

        assert!(
            !in_lava || !level.has_fast_lava(),
            "lava's flow in a dimension with fast lava isn't ported"
        );
        if in_lava {
            fluid_interaction.apply_current_to(FluidTags::LAVA, this, 0.0023333333333333335);
        }
    }

    this.entity_mut().fluid_interaction = fluid_interaction;
    in_any_fluid
}

/// `Entity.updateSwimming()`: a swimmer keeps swimming while it sprints in water; anyone else
/// starts when it sprints under water, standing in water.
pub fn update_swimming(this: &mut (impl EntityClass + Sized), level: &dyn Level) {
    if this.is_swimming() {
        let swimming =
            this.is_sprinting() && this.entity().is_in_water() && !this.entity().is_passenger();
        this.entity_mut().set_swimming(swimming);
    } else {
        let swimming = this.is_sprinting()
            && this.is_under_water()
            && !this.entity().is_passenger()
            && level
                .get_fluid_state(&this.entity().block_position())
                .is_tag(FluidTags::WATER);
        this.entity_mut().set_swimming(swimming);
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_candidate_step_up_heights, collide_with_shapes, get_input_vector};
    use crate::golden::Golden;
    use crate::net::minecraft::world::phys::aabb::AABB;
    use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    /// The pure parts of an entity's movement (`ShapeSuites.EntitySuite`).
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.Entity");
        golden.check(
            "collideWithShapes(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/AABB;Ljava/util/List;)Lnet/minecraft/world/phys/Vec3;",
            |movement: Vec3, bounding_box: AABB, shapes: Vec<VoxelShape>| {
                collide_with_shapes(&movement, &bounding_box, &shapes)
            },
        );
        golden.check(
            "collectCandidateStepUpHeights(Lnet/minecraft/world/phys/AABB;Ljava/util/List;FF)[F",
            |bounding_box: AABB,
             colliders: Vec<VoxelShape>,
             max_step_height,
             step_height_to_skip| {
                collect_candidate_step_up_heights(
                    &bounding_box,
                    &colliders,
                    max_step_height,
                    step_height_to_skip,
                )
            },
        );
        golden.check(
            "getInputVector(Lnet/minecraft/world/phys/Vec3;FF)Lnet/minecraft/world/phys/Vec3;",
            |input: Vec3, speed, y_rot| get_input_vector(&input, speed, y_rot),
        );
        golden.finish();
    }
}
