//! net/minecraft/client/player/LocalPlayer.java
//!
//! Not ported yet: nearly all of it, and everything it extends (`AbstractClientPlayer`, `Player`,
//! `LivingEntity`, `Entity`), which the player track ports with movement. What's here is what a
//! player standing still in the world needs: where it is and looks, and `sendChanges`, which tells
//! the server about it each tick. Until movement comes, `tick` doesn't move it, so it stays where
//! the server put it, and it's never on the ground or against a wall, which only collisions
//! decide. Its input never changes, so `sendChanges` never has a `player_input` to send.
//!
//! Java's player holds the game (`minecraft`), which is a parameter here, and its connection is
//! the client listener, shared with the game mode and the game.

use std::cell::RefCell;
use std::rc::Rc;

use crate::java::lang::math::Math;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
use crate::net::minecraft::network::protocol::game::serverbound_move_player_packet::{
    Pos, PosRot, Rot, StatusOnly,
};
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// The player the client plays as.
pub struct LocalPlayer {
    pub connection: Rc<RefCell<ClientPacketListener>>,
    id: i32,
    position: Vec3,
    old_position: Vec3,
    delta_movement: Vec3,
    y_rot: f32,
    x_rot: f32,
    pub y_rot_o: f32,
    pub x_rot_o: f32,
    on_ground: bool,
    pub horizontal_collision: bool,
    x_last: f64,
    y_last: f64,
    z_last: f64,
    y_rot_last: f32,
    x_rot_last: f32,
    last_on_ground: bool,
    last_horizontal_collision: bool,
    position_reminder: i32,
    was_sprinting: bool,
    auto_jump_enabled: bool,
}

impl LocalPlayer {
    /// Java's starts at the level's origin, like any entity, until the server says where it is.
    pub fn new(connection: Rc<RefCell<ClientPacketListener>>) -> LocalPlayer {
        LocalPlayer {
            connection,
            id: 0,
            position: Vec3::ZERO,
            old_position: Vec3::ZERO,
            delta_movement: Vec3::ZERO,
            y_rot: 0.0,
            x_rot: 0.0,
            y_rot_o: 0.0,
            x_rot_o: 0.0,
            on_ground: false,
            horizontal_collision: false,
            x_last: 0.0,
            y_last: 0.0,
            z_last: 0.0,
            y_rot_last: 0.0,
            x_rot_last: 0.0,
            last_on_ground: false,
            last_horizontal_collision: false,
            position_reminder: 0,
            was_sprinting: false,
            auto_jump_enabled: true,
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn set_id(&mut self, id: i32) {
        self.id = id;
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

    pub fn set_pos(&mut self, position: Vec3) {
        self.position = position;
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

    /// `getKnownMovement()`, which is the delta movement of a player that rides nothing.
    pub fn get_known_movement(&self) -> Vec3 {
        self.delta_movement
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

    /// Java's `setXRot` refuses an infinite or NaN rotation, with a warning, and keeps the rest
    /// within straight down and straight up (`Math.clamp(xRot % 360, -90, 90)`).
    pub fn set_x_rot(&mut self, x_rot: f32) {
        if x_rot.is_finite() {
            self.x_rot = Math::min_f32(90.0, Math::max_f32(x_rot % 360.0, -90.0));
        } else {
            eprintln!("Invalid entity rotation: {x_rot}, discarding.");
        }
    }

    /// `setOldPosAndRot(Vec3, float, float)`.
    pub fn set_old_pos_and_rot(&mut self, position: Vec3, y_rot: f32, x_rot: f32) {
        self.old_position = position;
        self.y_rot_o = y_rot;
        self.x_rot_o = x_rot;
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn is_passenger(&self) -> bool {
        false
    }

    pub fn is_sprinting(&self) -> bool {
        false
    }

    /// `resetPos()`: standing, with no movement and looking straight ahead. Java also moves it up
    /// out of any block it's in, at the lowest height the level's collisions let it be, which
    /// waits for the world track's blocks; the server's teleport puts it in place after anyway.
    pub fn reset_pos(&mut self) {
        self.set_delta_movement(Vec3::ZERO);
        self.set_x_rot(0.0);
    }

    /// The start of an entity's tick, the part of `Entity.baseTick` that remembers where it was
    /// (`setOldPosAndRot`). Its movement comes with the player track.
    pub fn tick(&mut self) {
        if self.connection.borrow().has_client_loaded() {
            self.old_position = self.position;
            self.y_rot_o = self.y_rot;
            self.x_rot_o = self.x_rot;
        }
    }

    /// Tells the server where the player is, once the level has loaded.
    pub fn send_changes(&mut self, minecraft: &Minecraft) {
        if self.connection.borrow().has_client_loaded() {
            if self.is_passenger() {
                self.connection.borrow().send(Rot::new(
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
            let connection = self.connection.borrow();
            if r#move && rot {
                connection.send(PosRot::new(
                    self.position(),
                    self.get_y_rot(),
                    self.get_x_rot(),
                    self.on_ground(),
                    self.horizontal_collision,
                ));
            } else if r#move {
                connection.send(Pos::new(
                    self.position(),
                    self.on_ground(),
                    self.horizontal_collision,
                ));
            } else if rot {
                connection.send(Rot::new(
                    self.get_y_rot(),
                    self.get_x_rot(),
                    self.on_ground(),
                    self.horizontal_collision,
                ));
            } else if self.last_on_ground != self.on_ground()
                || self.last_horizontal_collision != self.horizontal_collision
            {
                connection.send(StatusOnly::new(self.on_ground(), self.horizontal_collision));
            }
            drop(connection);

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
            self.auto_jump_enabled = true;
        }
    }

    /// `player_command` with the sprint's start or end, when it changed. The player never sprints
    /// until movement comes.
    fn send_is_sprinting_if_needed(&mut self) {
        let is_sprinting = self.is_sprinting();
        if is_sprinting != self.was_sprinting {
            self.was_sprinting = is_sprinting;
        }
    }

    fn is_controlled_camera(&self, minecraft: &Minecraft) -> bool {
        minecraft.get_camera_entity() == Some(self.id)
    }
}
