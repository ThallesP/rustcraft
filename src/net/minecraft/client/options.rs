//! net/minecraft/client/Options.java
//!
//! Not ported yet: nearly all of it. The options the server is told about
//! (`buildPlayerInformation`) are here with vanilla's defaults, and the render distance's
//! server side. Loading and saving `options.txt` comes with the rest; until then a run has the
//! defaults, and what the runner sets (the scenario's render distance, the oracle's particles).
//! The player's model parts are their masks combined (`PlayerModelPart`, all seven by default),
//! and the text filter is the user's profanity filter flag, off for an offline user.

use crate::net::minecraft::server::level::client_information::ClientInformation;
use crate::net::minecraft::server::level::particle_status::ParticleStatus;
use crate::net::minecraft::world::entity::humanoid_arm::HumanoidArm;
use crate::net::minecraft::world::entity::player::chat_visiblity::ChatVisiblity;

/// The player's settings.
#[derive(Clone, Debug)]
pub struct Options {
    pub language_code: String,
    render_distance: i32,
    server_render_distance: i32,
    chat_visibility: ChatVisiblity,
    chat_colors: bool,
    model_parts: i32,
    main_hand: HumanoidArm,
    allow_server_listing: bool,
    particles: ParticleStatus,
    pub joined_first_server: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options::new()
    }
}

impl Options {
    /// Every `PlayerModelPart`'s mask.
    const ALL_MODEL_PARTS: i32 = 0x7f;

    pub fn new() -> Options {
        Options {
            language_code: "en_us".to_string(),
            render_distance: 12,
            server_render_distance: 0,
            chat_visibility: ChatVisiblity::FULL,
            chat_colors: true,
            model_parts: Options::ALL_MODEL_PARTS,
            main_hand: HumanoidArm::RIGHT,
            allow_server_listing: true,
            particles: ParticleStatus::ALL,
            joined_first_server: false,
        }
    }

    /// `renderDistance().get()`.
    pub fn render_distance(&self) -> i32 {
        self.render_distance
    }

    /// `renderDistance().set(value)`, kept in its range of 2 to 32 chunks.
    pub fn set_render_distance(&mut self, render_distance: i32) {
        self.render_distance = render_distance.clamp(2, 32);
    }

    /// `particles().set(value)`.
    pub fn set_particles(&mut self, particles: ParticleStatus) {
        self.particles = particles;
    }

    pub fn build_player_information(&self) -> ClientInformation {
        ClientInformation::new(
            self.language_code.clone(),
            self.render_distance,
            self.chat_visibility,
            self.chat_colors,
            self.model_parts,
            self.main_hand,
            false,
            self.allow_server_listing,
            self.particles,
        )
    }

    pub fn set_server_render_distance(&mut self, server_render_distance: i32) {
        self.server_render_distance = server_render_distance;
    }

    pub fn get_effective_render_distance(&self) -> i32 {
        if self.server_render_distance > 0 {
            self.render_distance.min(self.server_render_distance)
        } else {
            self.render_distance
        }
    }
}
