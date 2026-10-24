//! net/minecraft/client/Options.java
//!
//! Not ported yet: nearly all of it. The options the server is told about
//! (`buildPlayerInformation`) are here with vanilla's defaults, and the render distance's
//! server side. Loading and saving `options.txt` comes with the rest; until then a run has the
//! defaults, and what the runner sets (the scenario's render distance, the oracle's particles).
//! The player's model parts are their masks combined (`PlayerModelPart`, all seven by default),
//! and the text filter is the user's profanity filter flag, off for an offline user. The
//! movement keys (as `KeyMapping`s held down or not), `autoJump` and `sprintWindow` are here for
//! the player, with `keyChat` and `keyCommand` (T and /), which open the chat, and `framerateLimit`
//! and `enableVsync` for the game loop. What the screens and the chat read are here with their
//! defaults: `menuBackgroundBlurriness`, the chat's opacity, line spacing, width, heights and scale,
//! and `chatColors`.

use crate::net::minecraft::client::gui::components::chat_component::ChatComponent;
use crate::net::minecraft::client::key_mapping::KeyMapping;
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
    pub key_up: KeyMapping,
    pub key_left: KeyMapping,
    pub key_down: KeyMapping,
    pub key_right: KeyMapping,
    pub key_jump: KeyMapping,
    pub key_shift: KeyMapping,
    pub key_sprint: KeyMapping,
    pub key_chat: KeyMapping,
    pub key_command: KeyMapping,
    auto_jump: bool,
    sprint_window: i32,
    biome_blend_radius: i32,
    /// What the renderer reads (`GameRenderer.extractOptions`): `fov()`, `fovEffectScale()`,
    /// `cloudRange()`, `ambientOcclusion()`, `cutoutLeaves()`, `chunkSectionFadeInTime()` (in
    /// seconds), `gamma()`, `glintStrength()`, and whether `textureFiltering()` is RGSS.
    pub fov: i32,
    pub fov_effect_scale: f64,
    pub cloud_range: i32,
    pub ambient_occlusion: bool,
    pub cutout_leaves: bool,
    pub chunk_section_fade_in_time: f64,
    pub gamma: f64,
    pub glint_strength: f64,
    pub texture_filtering_rgss: bool,
    /// `guiScale()`: the largest GUI scale, 0 for as large as fits.
    pub gui_scale: i32,
    /// `vignette()`, which the HUD doesn't draw yet.
    pub vignette: bool,
    /// `textBackgroundOpacity()` and `backgroundForChatOnly()`, behind the GUI's text.
    pub text_background_opacity: f64,
    pub background_for_chat_only: bool,
    /// `framerateLimit()`, `maxFps` in `options.txt`: frames a second at most, 260 for no limit.
    pub framerate_limit: i32,
    /// `enableVsync()`: frames wait for the display's refresh (the surface's present mode).
    pub enable_vsync: bool,
    /// `menuBackgroundBlurriness()`: the menu blur's radius, 0 to 10.
    pub menu_background_blurriness: i32,
    /// `chatOpacity()`, `chatLineSpacing()`, `chatWidth()`, `chatHeightFocused()`,
    /// `chatHeightUnfocused()` and `chatScale()`.
    pub chat_opacity: f64,
    pub chat_line_spacing: f64,
    pub chat_width: f64,
    pub chat_height_focused: f64,
    pub chat_height_unfocused: f64,
    pub chat_scale: f64,
}

impl Default for Options {
    fn default() -> Options {
        Options::new()
    }
}

impl Options {
    /// Every `PlayerModelPart`'s mask.
    const ALL_MODEL_PARTS: i32 = 0x7f;
    /// `biomeBlendRadius`'s default, which the oracle's `options.txt` keeps.
    pub const DEFAULT_BIOME_BLEND_RADIUS: i32 = 2;

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
            key_up: KeyMapping::new("key.forward"),
            key_left: KeyMapping::new("key.left"),
            key_down: KeyMapping::new("key.back"),
            key_right: KeyMapping::new("key.right"),
            key_jump: KeyMapping::new("key.jump"),
            key_shift: KeyMapping::new("key.sneak"),
            key_sprint: KeyMapping::new("key.sprint"),
            key_chat: KeyMapping::bound("key.chat", 23),
            key_command: KeyMapping::bound("key.command", 56),
            auto_jump: false,
            sprint_window: 7,
            biome_blend_radius: Options::DEFAULT_BIOME_BLEND_RADIUS,
            fov: 70,
            fov_effect_scale: 1.0,
            cloud_range: 128,
            ambient_occlusion: true,
            cutout_leaves: true,
            chunk_section_fade_in_time: 0.75,
            gamma: 0.5,
            glint_strength: 0.75,
            texture_filtering_rgss: false,
            gui_scale: 0,
            vignette: true,
            text_background_opacity: 0.5,
            background_for_chat_only: true,
            framerate_limit: 120,
            enable_vsync: true,
            menu_background_blurriness: 5,
            chat_opacity: 1.0,
            chat_line_spacing: 0.0,
            chat_width: 1.0,
            chat_height_focused: 1.0,
            chat_height_unfocused: ChatComponent::default_unfocused_pct(),
            chat_scale: 1.0,
        }
    }

    /// `getMenuBackgroundBlurriness`.
    pub fn get_menu_background_blurriness(&self) -> i32 {
        self.menu_background_blurriness
    }

    /// `chatColors().get()`.
    pub fn chat_colors(&self) -> bool {
        self.chat_colors
    }

    /// `chatVisibility().get()`.
    pub fn chat_visibility(&self) -> ChatVisiblity {
        self.chat_visibility
    }

    /// The key mappings a key goes to (`KeyMapping.MAP`): the movement keys by their defaults, and
    /// the chat's.
    pub fn key_mappings_mut(&mut self) -> [&mut KeyMapping; 2] {
        [&mut self.key_chat, &mut self.key_command]
    }

    /// `KeyMapping.releaseAll()`.
    pub fn release_all_keys(&mut self) {
        for mapping in [
            &mut self.key_up,
            &mut self.key_left,
            &mut self.key_down,
            &mut self.key_right,
            &mut self.key_jump,
            &mut self.key_shift,
            &mut self.key_sprint,
            &mut self.key_chat,
            &mut self.key_command,
        ] {
            mapping.release();
        }
    }

    /// `getBackgroundOpacity(float)`: the default, unless the background isn't only the chat's.
    pub fn get_background_opacity(&self, default_opacity: f32) -> f32 {
        if self.background_for_chat_only {
            default_opacity
        } else {
            self.text_background_opacity as f32
        }
    }

    /// `getBackgroundColor(float)`: black at the background's opacity.
    pub fn get_background_color(&self, default_opacity: f32) -> i32 {
        crate::net::minecraft::util::argb::ARGB::color_from_float(
            self.get_background_opacity(default_opacity),
            0.0,
            0.0,
            0.0,
        )
    }

    /// `getBackgroundColor(int)`: the default, unless the background isn't only the chat's.
    pub fn get_background_color_or(&self, default_color: i32) -> i32 {
        if self.background_for_chat_only {
            default_color
        } else {
            crate::net::minecraft::util::argb::ARGB::color_from_float(
                self.text_background_opacity as f32,
                0.0,
                0.0,
                0.0,
            )
        }
    }

    /// `autoJump().get()`.
    pub fn auto_jump(&self) -> bool {
        self.auto_jump
    }

    /// `sprintWindow().get()`: the ticks a second forward press has to sprint.
    /// `biomeBlendRadius()`: how many blocks around a block its tints blend over, 0 to 7.
    pub fn biome_blend_radius(&self) -> i32 {
        self.biome_blend_radius
    }

    pub fn set_biome_blend_radius(&mut self, biome_blend_radius: i32) {
        self.biome_blend_radius = biome_blend_radius;
    }

    pub fn sprint_window(&self) -> i32 {
        self.sprint_window
    }

    /// `renderDistance().get()`.
    pub fn render_distance(&self) -> i32 {
        self.render_distance
    }

    /// `renderDistance().set(value)`, kept in its range of 2 to 32 chunks.
    pub fn set_render_distance(&mut self, render_distance: i32) {
        self.render_distance = render_distance.clamp(2, 32);
    }

    /// `particles().get()`.
    pub fn particles(&self) -> ParticleStatus {
        self.particles
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
