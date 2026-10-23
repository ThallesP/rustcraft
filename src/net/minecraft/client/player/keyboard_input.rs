//! net/minecraft/client/player/KeyboardInput.java
//!
//! Java's holds the game's options and reads its key mappings each tick; here `tick` takes them.

use std::ops::{Deref, DerefMut};

use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::player::client_input::ClientInput;
use crate::net::minecraft::world::entity::player::input::Input;
use crate::net::minecraft::world::phys::vec2::Vec2;

/// The input from the keyboard's movement keys.
#[derive(Clone, Debug, Default)]
pub struct KeyboardInput {
    client_input: ClientInput,
}

impl Deref for KeyboardInput {
    type Target = ClientInput;

    fn deref(&self) -> &ClientInput {
        &self.client_input
    }
}

impl DerefMut for KeyboardInput {
    fn deref_mut(&mut self) -> &mut ClientInput {
        &mut self.client_input
    }
}

impl KeyboardInput {
    fn calculate_impulse(positive: bool, negative: bool) -> f32 {
        if positive == negative {
            0.0
        } else if positive {
            1.0
        } else {
            -1.0
        }
    }

    /// The keys held now, and the direction they make, normalized.
    pub fn tick(&mut self, options: &Options) {
        self.key_presses = Input {
            forward: options.key_up.is_down(),
            backward: options.key_down.is_down(),
            left: options.key_left.is_down(),
            right: options.key_right.is_down(),
            jump: options.key_jump.is_down(),
            shift: options.key_shift.is_down(),
            sprint: options.key_sprint.is_down(),
        };
        let forward_impulse =
            KeyboardInput::calculate_impulse(self.key_presses.forward, self.key_presses.backward);
        let left_impulse =
            KeyboardInput::calculate_impulse(self.key_presses.left, self.key_presses.right);
        self.move_vector = Vec2::new(left_impulse, forward_impulse).normalized();
    }
}
