//! net/minecraft/client/renderer/Panorama.java
//!
//! The title screen's turning panorama: its spin, a tenth of a degree a tick of real time at the
//! options' panorama speed while it spins, and the overlay that darkens its top and bottom. Java's
//! game renderer holds it; the port's game does (`Minecraft::panorama`), since screens draw while
//! the game renderer is borrowed, and the game's real time and options are arguments.

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::client::renderer::state::gui::panorama_render_state::PanoramaRenderState;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::mth::Mth;

pub struct Panorama {
    spin: f32,
    should_spin: bool,
}

impl Default for Panorama {
    fn default() -> Panorama {
        Panorama::new()
    }
}

impl Panorama {
    pub const PANORAMA_OVERLAY: &'static str = "textures/gui/title/background/panorama_overlay.png";

    pub fn new() -> Panorama {
        Panorama {
            spin: 0.0,
            should_spin: true,
        }
    }

    pub fn start_spin(&mut self) {
        self.should_spin = true;
    }

    pub fn hold_spin(&mut self) {
        self.should_spin = false;
    }

    /// `extractRenderState`: the spin this frame, `realtime_delta_ticks` of real time on at
    /// `panorama_speed`, and the overlay over the screen.
    pub fn extract_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        width: i32,
        height: i32,
        realtime_delta_ticks: f32,
        panorama_speed: f64,
    ) {
        if self.should_spin {
            let delta = (f64::from(realtime_delta_ticks) * panorama_speed) as f32;
            self.spin = Mth::wrap_degrees_f32(self.spin + delta * 0.1);
        }
        graphics.set_panorama_render_state(PanoramaRenderState { spin: -self.spin });
        graphics.blit_region(
            &RenderPipelines::get().GUI_TEXTURED,
            &Identifier::with_default_namespace(Panorama::PANORAMA_OVERLAY),
            0,
            0,
            0.0,
            0.0,
            width,
            height,
            16,
            128,
            16,
            128,
        );
    }
}
