//! net/minecraft/client/gui/contextualbar/ExperienceBar.java

use crate::net::minecraft::client::gui::contextualbar::contextual_bar::ContextualBar;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::resources::identifier::Identifier;

pub struct ExperienceBar;

impl ExperienceBar {
    const EXPERIENCE_BAR_BACKGROUND_SPRITE: &str = "hud/experience_bar_background";
    const EXPERIENCE_BAR_PROGRESS_SPRITE: &str = "hud/experience_bar_progress";

    pub fn extract_background(&self, graphics: &mut GuiGraphicsExtractor, player: &LocalPlayer) {
        let left = ContextualBar::left(graphics.gui_width());
        let top = ContextualBar::top(graphics.gui_height());
        let xp_needed_for_next_level = player.get_xp_needed_for_next_level();
        if xp_needed_for_next_level > 0 {
            let progress = (player.experience_progress * 183.0) as i32;
            graphics.blit_sprite(
                &RenderPipelines::get().GUI_TEXTURED,
                &Identifier::with_default_namespace(
                    ExperienceBar::EXPERIENCE_BAR_BACKGROUND_SPRITE,
                ),
                left,
                top,
                182,
                5,
            );
            if progress > 0 {
                graphics.blit_sprite_region(
                    &RenderPipelines::get().GUI_TEXTURED,
                    &Identifier::with_default_namespace(
                        ExperienceBar::EXPERIENCE_BAR_PROGRESS_SPRITE,
                    ),
                    182,
                    5,
                    0,
                    0,
                    left,
                    top,
                    progress,
                    5,
                );
            }
        }
    }

    pub fn extract_render_state(
        &self,
        _graphics: &mut GuiGraphicsExtractor,
        _player: &LocalPlayer,
    ) {
    }
}
