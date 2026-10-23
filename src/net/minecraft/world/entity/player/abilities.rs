//! net/minecraft/world/entity/player/Abilities.java
//!
//! Not ported yet: `pack`, `apply` and `Packed`, which come with the abilities packets.

/// What a player may do: fly, build, take no damage.
#[derive(Clone, Debug)]
pub struct Abilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub mayfly: bool,
    pub instabuild: bool,
    pub may_build: bool,
    flying_speed: f32,
    walking_speed: f32,
}

impl Default for Abilities {
    fn default() -> Abilities {
        Abilities {
            invulnerable: false,
            flying: false,
            mayfly: false,
            instabuild: false,
            may_build: true,
            flying_speed: 0.05,
            walking_speed: 0.1,
        }
    }
}

impl Abilities {
    pub fn get_flying_speed(&self) -> f32 {
        self.flying_speed
    }

    pub fn set_flying_speed(&mut self, value: f32) {
        self.flying_speed = value;
    }

    pub fn get_walking_speed(&self) -> f32 {
        self.walking_speed
    }

    pub fn set_walking_speed(&mut self, value: f32) {
        self.walking_speed = value;
    }
}
