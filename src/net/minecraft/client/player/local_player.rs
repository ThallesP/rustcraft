//! net/minecraft/client/player/LocalPlayer.java, with what it inherits from
//! `AbstractClientPlayer`.
//!
//! The player the client plays as: the concrete class of the entity hierarchy the port has
//! (PORTING.md, "Entities"), which implements `EntityClass`, `LivingEntityClass` and
//! `PlayerClass` with Java's overrides along its chain. Its movement is `tick` on a level of
//! blocks: `aiStep`'s crouching, sprinting (and swimming's), `moveTowardsClosestSpace` and going
//! down in water, the keyboard's input (`applyInput`, `modifyInput`), whether it's under water
//! (`wasUnderwater`, as of the start of its tick), and `move`, through `Player`, `LivingEntity` and
//! `Entity`.
//!
//! `sendChanges` tells the server what it holds (`player_input`), whether it sprints
//! (`player_command`) and where it is (the four kinds of `move_player`), in Java's order. The
//! vehicle it rides isn't ported, so a passenger only sends its rotation.
//!
//! Not ported yet: auto-jump's look-ahead (`updateAutoJump`, which panics if auto-jump is on: it's
//! off by default), portals and the spinning effect, the tutorial, flying's toggle (`mayfly` is
//! false in survival), riding, gliding, using items, water vision (`waterVisionTime`, which only
//! rendering reads), sounds (going under water and coming out), the recipe book,
//! permissions, chat and screens.
//!
//! Java's player holds the game (`minecraft`), which is a parameter here, and its connection is
//! the client listener, shared with the game mode and the game. A test's player has none, and
//! keeps what it sends instead (`sent`).

// Java's negated comparisons are kept as they are: they differ from the plain ones for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::player::keyboard_input::KeyboardInput;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, AxisDirection, Direction};
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::game::serverbound_move_player_packet::{
    Pos, PosRot, Rot, StatusOnly,
};
use crate::net::minecraft::network::protocol::game::serverbound_player_command_packet::{
    self, ServerboundPlayerCommandPacket,
};
use crate::net::minecraft::network::protocol::game::serverbound_player_input_packet::ServerboundPlayerInputPacket;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::tags::fluid_tags::FluidTags;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::ai::attributes::attributes::Attributes;
use crate::net::minecraft::world::entity::entity::{self, Entity, EntityClass};
use crate::net::minecraft::world::entity::entity_dimensions::EntityDimensions;
use crate::net::minecraft::world::entity::living_entity::{self, LivingEntity, LivingEntityClass};
use crate::net::minecraft::world::entity::mover_type::MoverType;
use crate::net::minecraft::world::entity::player::input::Input;
use crate::net::minecraft::world::entity::player::player::{self, Player, PlayerClass};
use crate::net::minecraft::world::entity::pose::Pose;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::level::{self, Level};
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::vec2::Vec2;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// The player the client plays as.
pub struct LocalPlayer {
    player: Player,
    pub connection: Option<Rc<RefCell<ClientPacketListener>>>,
    /// What a player without a connection (a test's) has sent, in order, which Java's connection
    /// would have written to the server.
    pub sent: Vec<Box<dyn Packet<dyn ServerGamePacketListener>>>,
    x_last: f64,
    y_last: f64,
    z_last: f64,
    y_rot_last: f32,
    x_rot_last: f32,
    last_on_ground: bool,
    last_horizontal_collision: bool,
    crouching: bool,
    was_sprinting: bool,
    position_reminder: i32,
    pub input: KeyboardInput,
    last_sent_input: Input,
    sprint_trigger_time: i32,
    auto_jump_enabled: bool,
    auto_jump_time: i32,
    flash_on_set_health: bool,
    last_hurt: f32,
    /// How long the eyes have been in water, up to 600 ticks, which the fog and the lightmap
    /// read (`getWaterVision`).
    water_vision_time: i32,
    /// What the tick reads of the game's options (`minecraft.options`), set by `tick`.
    options: Options,
}

impl Deref for LocalPlayer {
    type Target = Player;

    fn deref(&self) -> &Player {
        &self.player
    }
}

impl DerefMut for LocalPlayer {
    fn deref_mut(&mut self) -> &mut Player {
        &mut self.player
    }
}

impl LocalPlayer {
    const MINOR_COLLISION_ANGLE_THRESHOLD_RADIAN: f32 = 0.13962634;

    /// Java's starts at the level's origin, like any entity, until the server says where it is.
    pub fn new(connection: Option<Rc<RefCell<ClientPacketListener>>>) -> LocalPlayer {
        let mut player = LocalPlayer {
            player: Player::new(),
            connection,
            sent: Vec::new(),
            x_last: 0.0,
            y_last: 0.0,
            z_last: 0.0,
            y_rot_last: 0.0,
            x_rot_last: 0.0,
            last_on_ground: false,
            last_horizontal_collision: false,
            crouching: false,
            was_sprinting: false,
            position_reminder: 0,
            input: KeyboardInput::default(),
            last_sent_input: Input::EMPTY,
            sprint_trigger_time: 0,
            auto_jump_enabled: true,
            auto_jump_time: 0,
            flash_on_set_health: false,
            last_hurt: 0.0,
            water_vision_time: 0,
            options: Options::new(),
        };
        entity::refresh_dimensions(&mut player);
        player
    }

    /// `setPos(Vec3)`.
    pub fn set_pos(&mut self, position: Vec3) {
        self.entity_mut().set_pos_vec3(&position);
    }

    /// `setOldPosAndRot(Vec3, float, float)`.
    pub fn set_old_pos_and_rot(&mut self, position: Vec3, y_rot: f32, x_rot: f32) {
        self.entity_mut()
            .set_old_pos_and_rot_to(position, y_rot, x_rot);
    }

    /// `getKnownMovement()`, which is the delta movement of a player that rides nothing.
    pub fn get_known_movement(&self) -> Vec3 {
        self.get_delta_movement()
    }

    pub fn set_y_rot(&mut self, y_rot: f32) {
        self.entity_mut().set_y_rot(y_rot);
    }

    pub fn set_x_rot(&mut self, x_rot: f32) {
        self.entity_mut().set_x_rot(x_rot);
    }

    fn has_client_loaded(&self) -> bool {
        self.connection
            .as_ref()
            .is_none_or(|connection| connection.borrow().has_client_loaded())
    }

    /// `resetPos()`: standing, with no movement and looking straight ahead. Java also moves it up
    /// out of any block it's in, at the lowest height the level's collisions let it be, which
    /// waits for the world track's blocks; the server's teleport puts it in place after anyway.
    pub fn reset_pos(&mut self) {
        entity::set_pose(self, Pose::STANDING);
        self.entity_mut().set_delta_movement(Vec3::ZERO);
        self.entity_mut().set_x_rot(0.0);
        let max_health = self.living_entity().get_max_health();
        self.living_entity_mut().set_health(max_health);
    }

    /// `hurtTo(float)`: the health the server says it has, with the hurt animation when it went
    /// down, but the first time.
    pub fn hurt_to(&mut self, new_health: f32) {
        if self.flash_on_set_health {
            let dmg = self.living_entity().get_health() - new_health;
            if dmg <= 0.0 {
                self.living_entity_mut().set_health(new_health);
                if dmg < 0.0 {
                    self.living_entity_mut().damage_cooldown_time = 10;
                }
            } else {
                self.last_hurt = dmg;
                let living = self.living_entity_mut();
                living.damage_cooldown_time = 20;
                living.set_health(new_health);
                living.hurt_duration = 10;
                living.hurt_time = living.hurt_duration;
            }
        } else {
            self.living_entity_mut().set_health(new_health);
            self.flash_on_set_health = true;
        }
    }

    /// `ClientLevel.tickNonPassenger`'s first part, `commonTick`: where it was and looked.
    pub fn common_tick(&mut self) {
        self.entity_mut().common_tick();
    }

    /// `tick()` on its level, once the level has loaded, with the game's options as they are
    /// this tick.
    pub fn tick(&mut self, level: &dyn Level, options: &Options) {
        if self.has_client_loaded() {
            self.options = options.clone();
            player::tick(self, level);
        }
    }

    /// `isMovingSlowly()`: crouching or crawling.
    pub fn is_moving_slowly(&self) -> bool {
        self.is_crouching() || self.is_visually_crawling()
    }

    /// The input turned into the movement's sideways and forward parts: slower when sneaking,
    /// and as fast along a diagonal as along an axis.
    fn modify_input(&self, input: Vec2) -> Vec2 {
        if input.length_squared() == 0.0 {
            return input;
        }

        let mut new_input = input.scale(0.98);
        if self.is_moving_slowly() {
            let sneaking_movement_factor =
                self.living_entity()
                    .get_attribute_value(Attributes::SNEAKING_SPEED) as f32;
            new_input = new_input.scale(sneaking_movement_factor);
        }

        LocalPlayer::modify_input_speed_for_square_movement(new_input)
    }

    fn modify_input_speed_for_square_movement(input: Vec2) -> Vec2 {
        let length = input.length();
        if length <= 0.0 {
            return input;
        }

        let direction = input.scale(1.0 / length);
        let distance_to_unit_square = LocalPlayer::distance_to_unit_square(direction);
        let modified_length =
            crate::java::lang::math::Math::min_f32(length * distance_to_unit_square, 1.0);
        direction.scale(modified_length)
    }

    fn distance_to_unit_square(direction: Vec2) -> f32 {
        let direction_x = direction.x.abs();
        let direction_y = direction.y.abs();
        let tan = if direction_y > direction_x {
            direction_x / direction_y
        } else {
            direction_y / direction_x
        };
        Mth::sqrt(1.0 + Mth::square_f32(tan))
    }

    /// Pushes the player out of a block that suffocates it at `x`, `z`, toward the nearest side
    /// that doesn't.
    fn move_towards_closest_space(&mut self, level: &dyn Level, x: f64, z: f64) {
        let pos = BlockPos::containing_xyz(x, self.get_y(), z);
        if self.suffocates_at(level, &pos) {
            let xd = x - f64::from(pos.get_x());
            let zd = z - f64::from(pos.get_z());
            let mut dir = None;
            let mut closest = f64::MAX;
            for direction in [
                Direction::WEST,
                Direction::EAST,
                Direction::NORTH,
                Direction::SOUTH,
            ] {
                let axis_distance = direction.get_axis().choose_f64(xd, 0.0, zd);
                let distance_to_edge = if direction.get_axis_direction() == AxisDirection::POSITIVE
                {
                    1.0 - axis_distance
                } else {
                    axis_distance
                };
                if distance_to_edge < closest
                    && !self.suffocates_at(level, &pos.relative(direction))
                {
                    closest = distance_to_edge;
                    dir = Some(direction);
                }
            }

            if let Some(dir) = dir {
                let old_movement = self.get_delta_movement();
                if dir.get_axis() == Axis::X {
                    self.entity_mut().set_delta_movement_xyz(
                        0.1 * f64::from(dir.get_step_x()),
                        old_movement.y,
                        old_movement.z,
                    );
                } else {
                    self.entity_mut().set_delta_movement_xyz(
                        old_movement.x,
                        old_movement.y,
                        0.1 * f64::from(dir.get_step_z()),
                    );
                }
            }
        }
    }

    fn suffocates_at(&self, level: &dyn Level, pos: &BlockPos) -> bool {
        let bounding_box = self.get_bounding_box();
        let test_area = AABB::new(
            f64::from(pos.get_x()),
            bounding_box.min_y,
            f64::from(pos.get_z()),
            f64::from(pos.get_x()) + 1.0,
            bounding_box.max_y,
            f64::from(pos.get_z()) + 1.0,
        )
        .deflate(1.0E-7);
        level::collisions(level).collides_with_suffocating_block(Some(self), &test_area)
    }

    fn is_sprinting_possible(&self, allowed_in_shallow_water: bool) -> bool {
        !self.is_mobility_restricted()
            && self.has_enough_food_to_do_exhaustive_manoeuvres()
            && (allowed_in_shallow_water || !self.is_in_shallow_water())
    }

    fn can_start_sprinting(&self) -> bool {
        !self.is_sprinting()
            && self.input.has_forward_impulse()
            && self.is_sprinting_possible(self.get_abilities().flying)
            && (!self.is_fall_flying() || self.is_under_water())
            && (!self.is_moving_slowly() || self.is_under_water())
    }

    fn should_stop_run_sprinting(&self) -> bool {
        !self.is_sprinting_possible(self.get_abilities().flying)
            || !self.input.has_forward_impulse()
            || self.horizontal_collision && !self.minor_horizontal_collision
    }

    fn should_stop_swim_sprinting(&self) -> bool {
        !self.is_sprinting_possible(true)
            || !self.is_in_water()
            || !self.input.has_forward_impulse()
                && !self.on_ground()
                && !self.input.key_presses.shift
    }

    /// `LocalPlayer.aiStep()`: whether it crouches, the input, being pushed out of blocks,
    /// sprinting, then `Player`'s.
    fn local_ai_step(&mut self, level: &dyn Level) {
        if self.sprint_trigger_time > 0 {
            self.sprint_trigger_time -= 1;
        }

        let was_jumping = self.input.key_presses.jump;
        let was_shift_key_down = self.input.key_presses.shift;
        let has_forward_impulse = self.input.has_forward_impulse();
        let flying = self.get_abilities().flying;
        self.crouching = !flying
            && !self.is_swimming()
            && !self.is_passenger()
            && player::can_player_fit_within_blocks_and_entities_when(self, level, Pose::CROUCHING)
            && (self.is_shift_key_down()
                || !player::can_player_fit_within_blocks_and_entities_when(
                    self,
                    level,
                    Pose::STANDING,
                ));
        let options = self.options.clone();
        self.input.tick(&options);
        if self.auto_jump_time > 0 {
            self.auto_jump_time -= 1;
            self.input.make_jump();
        }

        if !self.no_physics && !self.is_passenger() {
            let half = f64::from(self.get_bb_width()) * 0.35;
            let (x, z) = (self.get_x(), self.get_z());
            self.move_towards_closest_space(level, x - half, z + half);
            let (x, z) = (self.get_x(), self.get_z());
            self.move_towards_closest_space(level, x - half, z - half);
            let (x, z) = (self.get_x(), self.get_z());
            self.move_towards_closest_space(level, x + half, z - half);
            let (x, z) = (self.get_x(), self.get_z());
            self.move_towards_closest_space(level, x + half, z + half);
        }

        if was_shift_key_down || self.input.key_presses.backward {
            self.sprint_trigger_time = 0;
        }

        if self.can_start_sprinting() {
            if !has_forward_impulse {
                if self.sprint_trigger_time > 0 {
                    self.set_sprinting(true);
                } else {
                    self.sprint_trigger_time = self.options.sprint_window();
                }
            }

            if self.input.key_presses.sprint {
                self.set_sprinting(true);
            }
        }

        if self.is_sprinting() {
            if self.is_swimming() {
                if self.should_stop_swim_sprinting() {
                    self.set_sprinting(false);
                }
            } else if self.should_stop_run_sprinting() {
                self.set_sprinting(false);
            }
        }

        assert!(!self.get_abilities().mayfly, "toggling flight isn't ported");
        // A jump's first tick would start gliding, which needs an elytra: the player has no
        // equipment, so it never does.
        let _ = was_jumping;
        if self.is_in_water() && self.input.key_presses.shift && self.is_affected_by_fluids() {
            self.go_down_in_water();
        }

        if self.is_eye_in_fluid(FluidTags::WATER) {
            let speed = if self.is_spectator() { 10 } else { 1 };
            self.water_vision_time = Mth::clamp_i32(self.water_vision_time + speed, 0, 600);
        } else if self.water_vision_time > 0 {
            self.water_vision_time = Mth::clamp_i32(self.water_vision_time - 10, 0, 600);
        }

        // AbstractClientPlayer.aiStep's bob, which only rendering reads, then Player's.
        player::ai_step(self, level);
    }

    /// `getWaterVision`: how well the eyes see in water, from 0 to 1 over the first 600 ticks
    /// there, nothing out of it.
    pub fn get_water_vision(&self) -> f32 {
        if !self.is_eye_in_fluid(FluidTags::WATER) {
            return 0.0;
        }

        if self.water_vision_time as f32 >= 600.0 {
            return 1.0;
        }

        let a = Mth::clamp_f32(self.water_vision_time as f32 / 100.0, 0.0, 1.0);
        let b = if (self.water_vision_time as f32) < 100.0 {
            0.0
        } else {
            Mth::clamp_f32((self.water_vision_time as f32 - 100.0) / 500.0, 0.0, 1.0)
        };
        a * 0.6 + b * 0.39999998
    }

    /// `LocalPlayer.move`: `Entity`'s, then auto-jump.
    fn local_move(&mut self, level: &dyn Level, mover_type: MoverType, delta: Vec3) {
        entity::r#move(self, level, mover_type, delta);
        assert!(!self.can_auto_jump(), "auto-jump's look-ahead isn't ported");
    }

    fn can_auto_jump(&self) -> bool {
        self.auto_jump_enabled
            && self.auto_jump_time <= 0
            && self.on_ground()
            && !self.is_staying_on_ground_surface()
            && !self.is_passenger()
            && self.input.get_move_vector().length_squared() > 0.0
    }

    /// Tells the server what changed, once the level has loaded: the keys held, then whether it
    /// sprints and where it is.
    pub fn send_changes(&mut self, minecraft: &Minecraft) {
        if self.has_client_loaded() {
            if self.last_sent_input != self.input.key_presses {
                self.send(ServerboundPlayerInputPacket::new(self.input.key_presses));
                self.last_sent_input = self.input.key_presses;
            }

            // A vehicle the player moves (`isLocalInstanceAuthoritative`) would be sent too, with
            // the sprinting, once riding is ported.
            if self.is_passenger() {
                self.send(Rot::new(
                    self.get_y_rot(),
                    self.get_x_rot(),
                    self.on_ground(),
                    self.horizontal_collision,
                ));
            } else {
                self.send_position(minecraft);
            }
        }
    }

    fn send(&mut self, packet: impl Packet<dyn ServerGamePacketListener> + 'static) {
        match &self.connection {
            Some(connection) => connection.borrow().send(packet),
            None => self.sent.push(Box::new(packet)),
        }
    }

    fn send_position(&mut self, minecraft: &Minecraft) {
        self.send_is_sprinting_if_needed();
        if self.is_controlled_camera(minecraft) {
            let delta_x = self.get_x() - self.x_last;
            let delta_y = self.get_y() - self.y_last;
            let delta_z = self.get_z() - self.z_last;
            let delta_y_rot = f64::from(self.get_y_rot() - self.y_rot_last);
            let delta_x_rot = f64::from(self.get_x_rot() - self.x_rot_last);
            self.position_reminder += 1;
            let r#move = Mth::length_squared3_f64(delta_x, delta_y, delta_z)
                > Mth::square_f64(2.0E-4)
                || self.position_reminder >= 20;
            let rot = delta_y_rot != 0.0 || delta_x_rot != 0.0;
            if r#move && rot {
                self.send(PosRot::new(
                    self.position(),
                    self.get_y_rot(),
                    self.get_x_rot(),
                    self.on_ground(),
                    self.horizontal_collision,
                ));
            } else if r#move {
                self.send(Pos::new(
                    self.position(),
                    self.on_ground(),
                    self.horizontal_collision,
                ));
            } else if rot {
                self.send(Rot::new(
                    self.get_y_rot(),
                    self.get_x_rot(),
                    self.on_ground(),
                    self.horizontal_collision,
                ));
            } else if self.last_on_ground != self.on_ground()
                || self.last_horizontal_collision != self.horizontal_collision
            {
                self.send(StatusOnly::new(self.on_ground(), self.horizontal_collision));
            }

            if r#move {
                self.x_last = self.get_x();
                self.y_last = self.get_y();
                self.z_last = self.get_z();
                self.position_reminder = 0;
            }

            if rot {
                self.y_rot_last = self.get_y_rot();
                self.x_rot_last = self.get_x_rot();
            }

            self.last_on_ground = self.on_ground();
            self.last_horizontal_collision = self.horizontal_collision;
            self.auto_jump_enabled = minecraft.options.borrow().auto_jump();
        }
    }

    /// `player_command` with the sprint's start or end, when it changed.
    fn send_is_sprinting_if_needed(&mut self) {
        let is_sprinting = self.is_sprinting();
        if is_sprinting != self.was_sprinting {
            let action = if is_sprinting {
                serverbound_player_command_packet::Action::START_SPRINTING
            } else {
                serverbound_player_command_packet::Action::STOP_SPRINTING
            };
            self.send(ServerboundPlayerCommandPacket::new(self, action));
            self.was_sprinting = is_sprinting;
        }
    }

    fn is_controlled_camera(&self, minecraft: &Minecraft) -> bool {
        minecraft.get_camera_entity() == Some(self.get_id())
    }

    /// The input it last told the server about (`lastSentInput`).
    pub fn last_sent_input(&self) -> Input {
        self.last_sent_input
    }

    /// `autoJumpEnabled` as the options set it, which `sendPosition` does each tick.
    pub fn set_auto_jump_enabled(&mut self, auto_jump_enabled: bool) {
        self.auto_jump_enabled = auto_jump_enabled;
    }
}

impl EntityClass for LocalPlayer {
    fn entity(&self) -> &Entity {
        &self.player
    }

    fn entity_mut(&mut self) -> &mut Entity {
        &mut self.player
    }

    fn get_dimensions(&self, pose: Pose) -> EntityDimensions {
        player::get_dimensions(self, pose)
    }

    /// `LocalPlayer.isShiftKeyDown()`: the sneak key in this tick's input.
    fn is_shift_key_down(&self) -> bool {
        self.input.key_presses.shift
    }

    fn is_crouching(&self) -> bool {
        self.crouching
    }

    fn set_sprinting(&mut self, sprinting: bool) {
        living_entity::set_sprinting(self, sprinting);
    }

    fn is_swimming(&self) -> bool {
        player::is_swimming(self)
    }

    fn is_player(&self) -> bool {
        true
    }

    /// `LocalPlayer.isUnderWater()`: its eyes were in water when its tick began
    /// (`wasUnderwater`).
    fn is_under_water(&self) -> bool {
        self.was_underwater
    }

    fn update_swimming(&mut self, level: &dyn Level) {
        player::update_swimming(self, level);
    }

    fn max_up_step(&self) -> f32 {
        living_entity::max_up_step(self)
    }

    fn get_default_gravity(&self) -> f64 {
        living_entity::get_default_gravity(self)
    }

    fn get_effective_gravity(&self) -> f64 {
        living_entity::get_effective_gravity(self)
    }

    fn get_air_drag(&self) -> f32 {
        living_entity::get_air_drag(self)
    }

    fn get_entity_bounciness(&self) -> f64 {
        living_entity::get_entity_bounciness(self)
    }

    fn is_pushed_by_fluid(&self) -> bool {
        !self.get_abilities().flying
    }

    /// `LocalPlayer.isHorizontalCollisionMinor`: the movement within 8 degrees of the input's
    /// direction.
    fn is_horizontal_collision_minor(&self, movement: &Vec3) -> bool {
        let y_rot_in_radians = self.get_y_rot() * (std::f64::consts::PI / 180.0) as f32;
        let y_rot_sin = f64::from(Mth::sin(f64::from(y_rot_in_radians)));
        let y_rot_cos = f64::from(Mth::cos(f64::from(y_rot_in_radians)));
        let global_xa = f64::from(self.xxa) * y_rot_cos - f64::from(self.zza) * y_rot_sin;
        let global_za = f64::from(self.zza) * y_rot_cos + f64::from(self.xxa) * y_rot_sin;
        let a_length_squared = Mth::square_f64(global_xa) + Mth::square_f64(global_za);
        let movement_length_squared = Mth::square_f64(movement.x) + Mth::square_f64(movement.z);
        if !(a_length_squared < f64::from(1.0E-5_f32))
            && !(movement_length_squared < f64::from(1.0E-5_f32))
        {
            let dot_product = global_xa * movement.x + global_za * movement.z;
            let angle_between_desired_and_actual_movement = crate::java::lang::math::Math::acos(
                dot_product / (a_length_squared * movement_length_squared).sqrt(),
            );
            angle_between_desired_and_actual_movement
                < f64::from(LocalPlayer::MINOR_COLLISION_ANGLE_THRESHOLD_RADIAN)
        } else {
            false
        }
    }

    /// Players are moved by their client (`Player.isLocalClientAuthoritative`).
    fn is_local_instance_authoritative(&self) -> bool {
        true
    }

    fn maybe_back_off_from_edge(
        &mut self,
        level: &dyn Level,
        delta: Vec3,
        mover_type: MoverType,
    ) -> Vec3 {
        player::maybe_back_off_from_edge(self, level, delta, mover_type)
    }

    fn check_fall_damage(
        &mut self,
        level: &dyn Level,
        ya: f64,
        on_ground: bool,
        on_state: BlockState,
        pos: BlockPos,
    ) {
        living_entity::check_fall_damage(self, level, ya, on_ground, on_state, pos);
    }

    fn get_block_speed_factor(&mut self, level: &dyn Level) -> f32 {
        player::get_block_speed_factor(self, level)
    }

    fn r#move(&mut self, level: &dyn Level, mover_type: MoverType, delta: Vec3) {
        self.local_move(level, mover_type, delta);
    }
}

impl LivingEntityClass for LocalPlayer {
    fn living_entity(&self) -> &LivingEntity {
        &self.player
    }

    fn living_entity_mut(&mut self) -> &mut LivingEntity {
        &mut self.player
    }

    fn get_speed(&self) -> f32 {
        player::get_speed(self)
    }

    fn get_flying_speed(&self) -> f32 {
        player::get_flying_speed(self)
    }

    fn is_affected_by_fluids(&self) -> bool {
        player::is_affected_by_fluids(self)
    }

    /// `LocalPlayer.applyInput()`: the keyboard's input, modified, as the movement's sideways and
    /// forward parts, and the jump key.
    fn apply_input(&mut self) {
        let modified_input = self.modify_input(self.input.get_move_vector());
        let jumping = self.input.key_presses.jump;
        let living = self.living_entity_mut();
        living.xxa = modified_input.x;
        living.zza = modified_input.y;
        living.jumping = jumping;
    }

    fn on_climbable(&mut self, level: &dyn Level) -> bool {
        player::on_climbable(self, level)
    }

    fn travel(&mut self, level: &dyn Level, input: &Vec3) {
        player::travel(self, level, input);
    }

    fn ai_step(&mut self, level: &dyn Level) {
        self.local_ai_step(level);
    }
}

impl PlayerClass for LocalPlayer {
    fn player(&self) -> &Player {
        &self.player
    }

    fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }
}
