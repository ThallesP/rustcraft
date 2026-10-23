//! net/minecraft/client/player/ClientInput.java

use crate::net::minecraft::world::entity::player::input::Input;
use crate::net::minecraft::world::phys::vec2::Vec2;

/// The player's input this tick: the keys, and the direction they make.
#[derive(Clone, Debug)]
pub struct ClientInput {
    pub key_presses: Input,
    pub(crate) move_vector: Vec2,
}

impl Default for ClientInput {
    fn default() -> ClientInput {
        ClientInput {
            key_presses: Input::EMPTY,
            move_vector: Vec2::ZERO,
        }
    }
}

impl ClientInput {
    pub fn get_move_vector(&self) -> Vec2 {
        self.move_vector
    }

    pub fn has_forward_impulse(&self) -> bool {
        self.move_vector.y > 1.0E-5
    }

    pub fn make_jump(&mut self) {
        self.key_presses.jump = true;
    }
}
