//! net/minecraft/client/gui/Hud.java
//!
//! The in-game HUD: the crosshair, the hotbar and its selection, the hearts (blinking after damage
//! or healing, jittering at 4 health or less, and the regeneration wave), the food (jittering when
//! the saturation runs out), the armor, the air bubbles, and the contextual bar (the experience
//! bar). Its `RandomSource` is reseeded from the tick count each frame, as Java's is, so the
//! jitters fall the same way at the same tick. What Java reads of its `Minecraft` (the player, the
//! game mode, the options, the level's hardcore flag, whether the HUD is hidden) is `HudContext`,
//! the frame's.
//!
//! Not ported yet, each with what it needs: the text it draws (the experience level, the held
//! item's name, overlay messages, titles, the demo's and the autosave's text, the scoreboard), the
//! font; the items in the hotbar's slots and the offhand, items; the chat, the tab list, the boss
//! bars, the subtitles, the spectator's hotbar and the debug overlay, their components; the
//! vignette (which the oracle client turns off), the spyglass, portal, nausea, powder snow and
//! equipment overlays, and the mob effects' icons and heart and food sprites, which need the
//! world border, the level's light, textures by location and mob effects; and the attack
//! indicator, which needs the player's attack strength (the player track's attacking): it's drawn
//! as full, which is what a player who hasn't attacked or switched items for a few ticks sees.
//! The locator and jumpable vehicle bars come with waypoints and vehicles.

use crate::net::minecraft::client::gui::contextualbar::contextual_bar::ContextualBar;
use crate::net::minecraft::client::gui::contextualbar::experience_bar::ExperienceBar;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::tags::fluid_tags::FluidTags;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::{self, RandomSource};
use crate::net::minecraft::util::util::Util;
use crate::net::minecraft::world::entity::ai::attributes::attributes::Attributes;
use crate::net::minecraft::world::level::game_type::GameType;

/// What a frame's HUD reads of the game, which Java reads from its `Minecraft`. `is_hidden` is
/// Java's `isHidden`, F1's, which the port keeps in `Minecraft.hud_hidden`.
pub struct HudContext<'a> {
    /// `Minecraft.font`.
    pub font: &'a std::rc::Rc<crate::net::minecraft::client::gui::font::Font>,
    pub is_hidden: bool,
    pub player: &'a LocalPlayer,
    pub player_mode: GameType,
    pub can_hurt_player: bool,
    pub has_experience: bool,
    pub is_hardcore: bool,
}

pub struct Hud {
    random: Box<dyn RandomSource>,
    tick_count: i32,
    last_health: i32,
    display_health: i32,
    last_health_time: i64,
    health_blink_time: i64,
    contextual_info_bar: (ContextualInfo, ContextualBar),
}

/// `Hud.ContextualInfo`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContextualInfo {
    EMPTY,
    EXPERIENCE,
}

const CROSSHAIR_SPRITE: &str = "hud/crosshair";
const HOTBAR_SPRITE: &str = "hud/hotbar";
const HOTBAR_SELECTION_SPRITE: &str = "hud/hotbar_selection";
const ARMOR_EMPTY_SPRITE: &str = "hud/armor_empty";
const ARMOR_HALF_SPRITE: &str = "hud/armor_half";
const ARMOR_FULL_SPRITE: &str = "hud/armor_full";
const FOOD_EMPTY_SPRITE: &str = "hud/food_empty";
const FOOD_HALF_SPRITE: &str = "hud/food_half";
const FOOD_FULL_SPRITE: &str = "hud/food_full";
const AIR_SPRITE: &str = "hud/air";
const AIR_POPPING_SPRITE: &str = "hud/air_bursting";
const AIR_EMPTY_SPRITE: &str = "hud/air_empty";

fn sprite(path: &str) -> Identifier {
    Identifier::with_default_namespace(path)
}

impl Default for Hud {
    fn default() -> Hud {
        Hud::new()
    }
}

impl Hud {
    pub fn new() -> Hud {
        Hud {
            random: random_source::create(),
            tick_count: 0,
            last_health: 0,
            display_health: 0,
            last_health_time: 0,
            health_blink_time: 0,
            contextual_info_bar: (ContextualInfo::EMPTY, ContextualBar::Empty),
        }
    }

    pub fn get_gui_ticks(&self) -> i32 {
        self.tick_count
    }

    /// `extractRenderState`, in a level with no loading screen up.
    pub fn extract_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        context: &HudContext,
    ) {
        if !context.is_hidden {
            self.extract_crosshair(graphics, context);
            graphics.next_stratum();
            self.extract_hotbar_and_decorations(graphics, context);
        }
        if !context.is_hidden {
            // The chat's stratum, which an empty chat draws nothing in.
            graphics.next_stratum();
        }
    }

    fn extract_crosshair(&mut self, graphics: &mut GuiGraphicsExtractor, context: &HudContext) {
        // The camera is first person, and the spectator's crosshair needs a hit result, which
        // isn't ported.
        if context.player_mode != GameType::SPECTATOR {
            graphics.next_stratum();
            let gui_width = graphics.gui_width();
            let gui_height = graphics.gui_height();
            graphics.blit_sprite(
                &RenderPipelines::get().CROSSHAIR,
                &sprite(CROSSHAIR_SPRITE),
                (gui_width - 15) / 2,
                (gui_height - 15) / 2,
                15,
                15,
            );
        }
    }

    fn extract_hotbar_and_decorations(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        context: &HudContext,
    ) {
        if context.player_mode == GameType::SPECTATOR {
            panic!("the spectator's hotbar isn't ported");
        }
        self.extract_item_hotbar(graphics, context);
        if context.can_hurt_player {
            self.extract_player_health(graphics, context);
        }

        let next_contextual_info = if context.has_experience {
            ContextualInfo::EXPERIENCE
        } else {
            ContextualInfo::EMPTY
        };
        if next_contextual_info != self.contextual_info_bar.0 {
            let bar = match next_contextual_info {
                ContextualInfo::EMPTY => ContextualBar::Empty,
                ContextualInfo::EXPERIENCE => ContextualBar::Experience(ExperienceBar),
            };
            self.contextual_info_bar = (next_contextual_info, bar);
        }
        self.contextual_info_bar
            .1
            .extract_background(graphics, context.player);
        if context.has_experience && context.player.experience_level > 0 {
            ContextualBar::extract_experience_level(
                graphics,
                context.font,
                context.player.experience_level,
            );
        }
        self.contextual_info_bar
            .1
            .extract_render_state(graphics, context.player);
    }

    fn extract_item_hotbar(&mut self, graphics: &mut GuiGraphicsExtractor, context: &HudContext) {
        let player = context.player;
        let screen_center = graphics.gui_width() / 2;
        let gui_height = graphics.gui_height();
        graphics.blit_sprite(
            &RenderPipelines::get().GUI_TEXTURED,
            &sprite(HOTBAR_SPRITE),
            screen_center - 91,
            gui_height - 22,
            182,
            22,
        );
        graphics.blit_sprite(
            &RenderPipelines::get().GUI_TEXTURED,
            &sprite(HOTBAR_SELECTION_SPRITE),
            screen_center - 91 - 1 + player.selected_slot * 20,
            gui_height - 22 - 1,
            24,
            23,
        );
        // The offhand and the slots: the inventory isn't ported, so they're empty, which draws
        // nothing.
    }

    fn extract_player_health(&mut self, graphics: &mut GuiGraphicsExtractor, context: &HudContext) {
        let player = context.player;
        let current_health = Mth::ceil_f32(player.get_health());
        let blink = self.health_blink_time > i64::from(self.tick_count)
            && (self.health_blink_time - i64::from(self.tick_count)) / 3 % 2 == 1;
        let time_millis = Util::get_millis();
        if current_health < self.last_health && player.damage_cooldown_time > 0 {
            self.last_health_time = time_millis;
            self.health_blink_time = i64::from(self.tick_count + 20);
        } else if current_health > self.last_health && player.damage_cooldown_time > 0 {
            self.last_health_time = time_millis;
            self.health_blink_time = i64::from(self.tick_count + 10);
        }

        if time_millis - self.last_health_time > 1000 {
            self.display_health = current_health;
            self.last_health_time = time_millis;
        }

        self.last_health = current_health;
        let old_health = self.display_health;
        // An int product, widened to the long seed.
        self.random
            .set_seed(i64::from(self.tick_count.wrapping_mul(312871)));
        let x_left = graphics.gui_width() / 2 - 91;
        let x_right = graphics.gui_width() / 2 + 91;
        let y_line_base = graphics.gui_height() - 39;
        let max_health = (player.get_attribute_value(Attributes::MAX_HEALTH) as f32)
            .max(old_health.max(current_health) as f32);
        let total_absorption = Mth::ceil_f32(player.get_absorption_amount());
        let num_health_rows = Mth::ceil_f32((max_health + total_absorption as f32) / 2.0 / 10.0);
        let health_row_height = (10 - (num_health_rows - 2)).max(3);
        let mut y_line_air = y_line_base - 10;
        // Regeneration's wave: mob effects aren't ported, so the player has none.
        let heart_offset_index = -1;

        extract_armor(
            graphics,
            player,
            y_line_base,
            num_health_rows,
            health_row_height,
            x_left,
        );
        self.extract_hearts(
            graphics,
            context,
            x_left,
            y_line_base,
            health_row_height,
            heart_offset_index,
            max_health,
            current_health,
            old_health,
            total_absorption,
            blink,
        );
        // Vehicles aren't ported, so there are no vehicle hearts.
        let vehicle_hearts = 0;
        if vehicle_hearts == 0 {
            self.extract_food(graphics, player, y_line_base, x_right);
            y_line_air -= 10;
        }

        self.extract_air_bubbles(graphics, player, vehicle_hearts, y_line_air, x_right);
    }

    #[allow(clippy::too_many_arguments)]
    fn extract_hearts(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        context: &HudContext,
        x_left: i32,
        y_line_base: i32,
        health_row_height: i32,
        heart_offset_index: i32,
        max_health: f32,
        current_health: i32,
        old_health: i32,
        absorption: i32,
        blink: bool,
    ) {
        let r#type = HeartType::for_player();
        let is_hardcore = context.is_hardcore;
        let health_container_count = Mth::ceil_f64(f64::from(max_health) / 2.0);
        let absorption_container_count = Mth::ceil_f64(f64::from(absorption) / 2.0);
        let max_health_halves_count = health_container_count * 2;

        let mut container_index = health_container_count + absorption_container_count - 1;
        while container_index >= 0 {
            let row = container_index / 10;
            let column = container_index % 10;
            let xo = x_left + column * 8;
            let mut yo = y_line_base - row * health_row_height;
            if current_health + absorption <= 4 {
                yo += self.random.next_int_bounded(2);
            }

            if container_index < health_container_count && container_index == heart_offset_index {
                yo -= 2;
            }

            extract_heart(
                graphics,
                HeartType::CONTAINER,
                xo,
                yo,
                is_hardcore,
                blink,
                false,
            );
            let halves = container_index * 2;
            let is_absorption_heart = container_index >= health_container_count;
            if is_absorption_heart {
                let absorption_halves = halves - max_health_halves_count;
                if absorption_halves < absorption {
                    let half_heart = absorption_halves + 1 == absorption;
                    let absorbing_type = if r#type == HeartType::WITHERED {
                        r#type
                    } else {
                        HeartType::ABSORBING
                    };
                    extract_heart(
                        graphics,
                        absorbing_type,
                        xo,
                        yo,
                        is_hardcore,
                        false,
                        half_heart,
                    );
                }
            }

            if blink && halves < old_health {
                let half_heart = halves + 1 == old_health;
                extract_heart(graphics, r#type, xo, yo, is_hardcore, true, half_heart);
            }

            if halves < current_health {
                let half_heart = halves + 1 == current_health;
                extract_heart(graphics, r#type, xo, yo, is_hardcore, false, half_heart);
            }
            container_index -= 1;
        }
    }

    fn extract_air_bubbles(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        player: &LocalPlayer,
        vehicle_hearts: i32,
        mut y_line_air: i32,
        x_right: i32,
    ) {
        let max_air_supply_ticks = player.get_max_air_supply();
        let current_air_supply_ticks = player.get_air_supply().clamp(0, max_air_supply_ticks);
        let is_under_water = player.is_eye_in_fluid(FluidTags::WATER);
        if is_under_water || current_air_supply_ticks < max_air_supply_ticks {
            y_line_air = self.get_air_bubble_y_line(vehicle_hearts, y_line_air);
            let full_air_bubbles =
                get_current_air_supply_bubble(current_air_supply_ticks, max_air_supply_ticks, -2);
            let popping_air_bubble_position =
                get_current_air_supply_bubble(current_air_supply_ticks, max_air_supply_ticks, 0);
            let empty_air_bubbles = 10
                - get_current_air_supply_bubble(
                    current_air_supply_ticks,
                    max_air_supply_ticks,
                    get_empty_bubble_delay_duration(current_air_supply_ticks, is_under_water),
                );
            let is_popping_bubble = full_air_bubbles != popping_air_bubble_position;
            for air_bubble in 1..=10 {
                let air_bubble_x_pos = x_right - (air_bubble - 1) * 8 - 9;
                if air_bubble <= full_air_bubbles {
                    graphics.blit_sprite(
                        &RenderPipelines::get().GUI_TEXTURED,
                        &sprite(AIR_SPRITE),
                        air_bubble_x_pos,
                        y_line_air,
                        9,
                        9,
                    );
                } else if is_popping_bubble
                    && air_bubble == popping_air_bubble_position
                    && is_under_water
                {
                    // The bubble's pop sound comes with sounds.
                    graphics.blit_sprite(
                        &RenderPipelines::get().GUI_TEXTURED,
                        &sprite(AIR_POPPING_SPRITE),
                        air_bubble_x_pos,
                        y_line_air,
                        9,
                        9,
                    );
                } else if air_bubble > 10 - empty_air_bubbles {
                    let wobble_y_offset = if empty_air_bubbles == 10 && self.tick_count % 2 == 0 {
                        self.random.next_int_bounded(2)
                    } else {
                        0
                    };
                    graphics.blit_sprite(
                        &RenderPipelines::get().GUI_TEXTURED,
                        &sprite(AIR_EMPTY_SPRITE),
                        air_bubble_x_pos,
                        y_line_air + wobble_y_offset,
                        9,
                        9,
                    );
                }
            }
        }
    }

    fn get_air_bubble_y_line(&self, vehicle_hearts: i32, y_line_air: i32) -> i32 {
        // getVisibleVehicleHeartRows.
        let row_offset = (f64::from(vehicle_hearts) / 10.0).ceil() as i32 - 1;
        y_line_air - row_offset * 10
    }

    fn extract_food(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        player: &LocalPlayer,
        y_line_base: i32,
        x_right: i32,
    ) {
        let food = player.food_level;
        for i in 0..10 {
            let mut yo = y_line_base;
            // Hunger is a mob effect, which isn't ported.
            let (empty, half, full) = (FOOD_EMPTY_SPRITE, FOOD_HALF_SPRITE, FOOD_FULL_SPRITE);
            if player.saturation_level <= 0.0 && self.tick_count % (food * 3 + 1) == 0 {
                yo += self.random.next_int_bounded(3) - 1;
            }

            let xo = x_right - i * 8 - 9;
            graphics.blit_sprite(
                &RenderPipelines::get().GUI_TEXTURED,
                &sprite(empty),
                xo,
                yo,
                9,
                9,
            );
            if i * 2 + 1 < food {
                graphics.blit_sprite(
                    &RenderPipelines::get().GUI_TEXTURED,
                    &sprite(full),
                    xo,
                    yo,
                    9,
                    9,
                );
            }

            if i * 2 + 1 == food {
                graphics.blit_sprite(
                    &RenderPipelines::get().GUI_TEXTURED,
                    &sprite(half),
                    xo,
                    yo,
                    9,
                    9,
                );
            }
        }
    }

    /// `tick(boolean)`: the autosave indicator (a singleplayer server's, so it stays at 0 on a
    /// server), then the rest unless the game is paused.
    pub fn tick(&mut self, pause: bool) {
        if !pause {
            self.tick_count = self.tick_count.wrapping_add(1);
            // The vignette's brightness, the held item's highlight and the chat come with them.
        }
    }
}

fn extract_armor(
    graphics: &mut GuiGraphicsExtractor,
    player: &LocalPlayer,
    y_line_base: i32,
    num_health_rows: i32,
    health_row_height: i32,
    x_left: i32,
) {
    let armor = player.get_armor_value();
    if armor > 0 {
        let y_line_armor = y_line_base - (num_health_rows - 1) * health_row_height - 10;
        for i in 0..10 {
            let xo = x_left + i * 8;
            if i * 2 + 1 < armor {
                graphics.blit_sprite(
                    &RenderPipelines::get().GUI_TEXTURED,
                    &sprite(ARMOR_FULL_SPRITE),
                    xo,
                    y_line_armor,
                    9,
                    9,
                );
            }
            if i * 2 + 1 == armor {
                graphics.blit_sprite(
                    &RenderPipelines::get().GUI_TEXTURED,
                    &sprite(ARMOR_HALF_SPRITE),
                    xo,
                    y_line_armor,
                    9,
                    9,
                );
            }
            if i * 2 + 1 > armor {
                graphics.blit_sprite(
                    &RenderPipelines::get().GUI_TEXTURED,
                    &sprite(ARMOR_EMPTY_SPRITE),
                    xo,
                    y_line_armor,
                    9,
                    9,
                );
            }
        }
    }
}

fn extract_heart(
    graphics: &mut GuiGraphicsExtractor,
    r#type: HeartType,
    xo: i32,
    yo: i32,
    is_hardcore: bool,
    blinks: bool,
    half: bool,
) {
    graphics.blit_sprite(
        &RenderPipelines::get().GUI_TEXTURED,
        &sprite(r#type.get_sprite(is_hardcore, half, blinks)),
        xo,
        yo,
        9,
        9,
    );
}

fn get_current_air_supply_bubble(
    current_air_supply_ticks: i32,
    max_air_supply_ticks: i32,
    tick_offset: i32,
) -> i32 {
    Mth::ceil_f32(
        ((current_air_supply_ticks + tick_offset) * 10) as f32 / max_air_supply_ticks as f32,
    )
}

fn get_empty_bubble_delay_duration(current_air_supply_ticks: i32, is_under_water: bool) -> i32 {
    if current_air_supply_ticks != 0 && is_under_water {
        1
    } else {
        0
    }
}

/// `Hud.HeartType`: each kind's eight sprites, by hardcore, half and blinking. Poisoned and frozen
/// hearts come with mob effects and freezing.
#[allow(non_camel_case_types, clippy::upper_case_acronyms, dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeartType {
    CONTAINER,
    NORMAL,
    POISIONED,
    WITHERED,
    ABSORBING,
    FROZEN,
}

impl HeartType {
    /// `full`, `fullBlinking`, `half`, `halfBlinking`, then the hardcore ones the same way.
    fn sprites(self) -> [&'static str; 8] {
        match self {
            HeartType::CONTAINER => [
                "hud/heart/container",
                "hud/heart/container_blinking",
                "hud/heart/container",
                "hud/heart/container_blinking",
                "hud/heart/container_hardcore",
                "hud/heart/container_hardcore_blinking",
                "hud/heart/container_hardcore",
                "hud/heart/container_hardcore_blinking",
            ],
            HeartType::NORMAL => [
                "hud/heart/full",
                "hud/heart/full_blinking",
                "hud/heart/half",
                "hud/heart/half_blinking",
                "hud/heart/hardcore_full",
                "hud/heart/hardcore_full_blinking",
                "hud/heart/hardcore_half",
                "hud/heart/hardcore_half_blinking",
            ],
            HeartType::POISIONED => [
                "hud/heart/poisoned_full",
                "hud/heart/poisoned_full_blinking",
                "hud/heart/poisoned_half",
                "hud/heart/poisoned_half_blinking",
                "hud/heart/poisoned_hardcore_full",
                "hud/heart/poisoned_hardcore_full_blinking",
                "hud/heart/poisoned_hardcore_half",
                "hud/heart/poisoned_hardcore_half_blinking",
            ],
            HeartType::WITHERED => [
                "hud/heart/withered_full",
                "hud/heart/withered_full_blinking",
                "hud/heart/withered_half",
                "hud/heart/withered_half_blinking",
                "hud/heart/withered_hardcore_full",
                "hud/heart/withered_hardcore_full_blinking",
                "hud/heart/withered_hardcore_half",
                "hud/heart/withered_hardcore_half_blinking",
            ],
            HeartType::ABSORBING => [
                "hud/heart/absorbing_full",
                "hud/heart/absorbing_full_blinking",
                "hud/heart/absorbing_half",
                "hud/heart/absorbing_half_blinking",
                "hud/heart/absorbing_hardcore_full",
                "hud/heart/absorbing_hardcore_full_blinking",
                "hud/heart/absorbing_hardcore_half",
                "hud/heart/absorbing_hardcore_half_blinking",
            ],
            HeartType::FROZEN => [
                "hud/heart/frozen_full",
                "hud/heart/frozen_full_blinking",
                "hud/heart/frozen_half",
                "hud/heart/frozen_half_blinking",
                "hud/heart/frozen_hardcore_full",
                "hud/heart/frozen_hardcore_full_blinking",
                "hud/heart/frozen_hardcore_half",
                "hud/heart/frozen_hardcore_half_blinking",
            ],
        }
    }

    fn get_sprite(self, is_hardcore: bool, is_half: bool, is_blink: bool) -> &'static str {
        let index = usize::from(is_hardcore) * 4 + usize::from(is_half) * 2 + usize::from(is_blink);
        self.sprites()[index]
    }

    /// `forPlayer`: poisoned, withered or frozen, which need mob effects and freezing, neither
    /// ported, so normal.
    fn for_player() -> HeartType {
        HeartType::NORMAL
    }
}
