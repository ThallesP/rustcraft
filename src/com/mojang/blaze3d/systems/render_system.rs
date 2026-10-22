//! com/mojang/blaze3d/systems/RenderSystem.java
//!
//! Only `initBackendSystem` so far: SDL's hints and app metadata, then its video subsystem.
//! `SdlDebug.init()` (SDL's log callback) and the timer it returns come with the window loop;
//! the device (`DEVICE`) is passed around until the game's renderer holds it.

use crate::org::lwjgl::sdl::sdl_error::SDLError;
use crate::org::lwjgl::sdl::sdl_hints::SDLHints;
use crate::org::lwjgl::sdl::sdl_init::SDLInit;

pub struct RenderSystem;

impl RenderSystem {
    /// Panics where Java throws `IllegalStateException`.
    pub fn init_backend_system() {
        let version = include_str!("../../../../../MINECRAFT_VERSION").trim();
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.name", "Minecraft");
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.version", version);
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.identifier", "com.mojang.minecraft");
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.creator", "Mojang Studios");
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.copyright", "Copyright Mojang AB.");
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.url", "https://www.minecraft.net");
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.type", "game");
        SDLHints::SDL_SetHint("SDL_NO_SIGNAL_HANDLERS", "1");
        SDLHints::SDL_SetHint("SDL_VIDEO_MINIMIZE_ON_FOCUS_LOSS", "0");
        SDLHints::SDL_SetHint("SDL_QUIT_ON_LAST_WINDOW_CLOSE", "0");
        SDLHints::SDL_SetHint("SDL_MOUSE_FOCUS_CLICKTHROUGH", "1");
        SDLHints::SDL_SetHint("SDL_ENABLE_SCREEN_KEYBOARD", "0");
        SDLHints::SDL_SetHint("SDL_IME_IMPLEMENTED_UI", "composition");
        assert!(
            SDLInit::SDL_Init(SDLInit::SDL_INIT_VIDEO),
            "Unable to initialize SDL: {}",
            SDLError::SDL_GetError().unwrap_or_default()
        );
    }
}
