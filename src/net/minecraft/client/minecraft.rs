//! net/minecraft/client/Minecraft.java
//!
//! Not ported yet: nearly all of it: sound, the integrated server, resources, telemetry and the
//! profiler, and most of the input's bindings. What's here is the game loop's timing and what a
//! tick does with a connection to a server: run the packets queued for the game thread, then up
//! to 10 ticks, each ticking the connection (the game mode's, or the connect screen's before
//! there's a level), the game renderer, the player on its level, what the player sends, the
//! level's own tick and its animations (`animateTick`, whose particles and sounds go to the
//! particle engine and sound manager, which drop them until theirs are ported), and ending with
//! `client_tick_end`; then the frame's update of the level's light (`renderFrame`'s
//! `level.update()`) and, with a renderer, a frame of the level, offscreen or presented to the
//! window. The renderer is made on demand (`init_renderer`): a headless client that takes no
//! screenshots runs without one.
//!
//! The game is shared by its listeners, its player and its screens, like Java's, so its state is in
//! cells, it keeps a weak reference to itself (`this`) for the screens it makes, and its other
//! back-references are parameters (`LocalPlayer::send_changes`). The screen (`Gui.screen`, Java's
//! `gui.screen()`) is the gui's, taken out of the gui's cell while it handles input, ticks or
//! draws, so it can reach the game; a screen replaced while it's handling something is told it was
//! removed (`removed`) once it's done. A client with no window stops at the disconnected screen.
//! Frames come at most at the options' frame rate limit (`FramerateLimiter`, none at 260), the
//! game thread sleeping until the next is due, while the connection's own thread goes on reading
//! it (`Connection`), like Java's Netty threads. The surface's present mode follows `enableVsync`
//! from the start. The benchmark times each frame, the end of its GL calls, each tick and each
//! frame's light updates (`bench`).

use std::cell::{Cell, OnceCell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::time::Instant;

use crate::com::mojang::blaze3d::platform::window::Window;
use crate::com::mojang::renderpearl::api::device::gpu_surface::{GpuSurface, PresentMode};
use crate::net::minecraft::client::color::block::block_colors::BlockColors;
use crate::net::minecraft::client::delta_tracker::Timer;
use crate::net::minecraft::client::framerate_limiter::FramerateLimiter;
use crate::net::minecraft::client::gui::components::chat_component::{ChatContext, ChatMethod};
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::font::font_manager::FontManager;
use crate::net::minecraft::client::gui::gui::Gui;
use crate::net::minecraft::client::gui::hud::HudContext;
use crate::net::minecraft::client::gui::screens::chat_screen::ChatScreen;
use crate::net::minecraft::client::gui::screens::disconnected_screen::DisconnectedScreen;
use crate::net::minecraft::client::gui::screens::level_loading_screen::LevelLoadingScreen;
use crate::net::minecraft::client::gui::screens::pause_screen::PauseScreen;
use crate::net::minecraft::client::gui::screens::screen::{self, ScreenRef};
use crate::net::minecraft::client::input::character_event::CharacterEvent;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::input::mouse_button_event::MouseButtonEvent;
use crate::net::minecraft::client::input::mouse_button_info::MouseButtonInfo;
use crate::net::minecraft::client::input_type::{self, InputType};
use crate::net::minecraft::client::keyboard_handler::{KeyAction, KeyboardHandler};
use crate::net::minecraft::client::mouse_handler::MouseHandler;
use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::multiplayer::client_level::{LevelEffects, LevelTicking};
use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
use crate::net::minecraft::client::multiplayer::multi_player_game_mode::MultiPlayerGameMode;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::particle::particle_engine::{NoParticles, ParticleEngine};
use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::client::renderer::chunk::render_region_cache::LevelEnvironment;
use crate::net::minecraft::client::renderer::game_renderer::{CameraEntityState, GameRenderer};
use crate::net::minecraft::client::resources::server::downloaded_pack_source::DownloadedPackSource;
use crate::net::minecraft::client::sounds::sound_manager::{NoSound, SoundManager};
use crate::net::minecraft::client::user::User;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_processor::PacketProcessor;
use crate::net::minecraft::network::protocol::game::serverbound_client_tick_end_packet::ServerboundClientTickEndPacket;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::util::Util;
use crate::net::minecraft::world::attribute::attribute_type::Value;
use crate::net::minecraft::world::attribute::environment_attribute_layer::LayerLevel;
use crate::net::minecraft::world::attribute::environment_attribute_system::EnvironmentAttributeSystem;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::flowing_fluid;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::org::lwjgl::sdl::sdl_events::{SDLEvents, SdlEvent};
use crate::org::lwjgl::sdl::sdl_keyboard::SDLKeyboard;
use crate::org::lwjgl::sdl::sdl_mouse::SDLMouse;
use crate::rustcraft::bench;

/// Something that drives the game from outside, like Fabric's client game tests drive vanilla
/// (`tools/port_client.py`'s scenarios). Fabric runs at most one tick a frame and parks the client
/// before it, after the frame's queued packets and tasks, for the test to act on it
/// (`captureTicksPerFrame`, `postRunTasks`): that's `before_tick`. `after_tick` is called as the
/// tick ends.
pub trait TestDriver {
    fn before_tick(&mut self, _minecraft: &Rc<Minecraft>) {}

    fn after_tick(&mut self, minecraft: &Rc<Minecraft>);
}

/// The client.
pub struct Minecraft {
    /// The game itself, for the screens it makes, which keep it.
    this: Weak<Minecraft>,
    pub options: RefCell<Options>,
    user: User,
    packet_processor: Arc<PacketProcessor>,
    pub player: RefCell<Option<LocalPlayer>>,
    pub level: RefCell<Option<ClientLevel>>,
    pub game_mode: RefCell<Option<Rc<MultiPlayerGameMode>>>,
    camera_entity: Cell<Option<i32>>,
    /// Screens replaced while they were handling something, told so when they're done.
    removed_screens: RefCell<Vec<ScreenRef>>,
    delta_tracker: RefCell<Timer>,
    running: Cell<bool>,
    pause: Cell<bool>,
    client_tick_count: Cell<i32>,
    test_driver: RefCell<Option<Box<dyn TestDriver>>>,
    pub game_renderer: RefCell<Option<GameRenderer>>,
    /// The window frames are presented to, and its surface; none for an offscreen renderer.
    window: RefCell<Option<(Window, Box<dyn GpuSurface>)>>,
    /// `gui.hud.isHidden()`: F1.
    pub hud_hidden: Cell<bool>,
    pub mouse_handler: RefCell<MouseHandler>,
    pub gui: RefCell<Gui>,
    /// `downloadedPackSource`: the server's resource packs.
    pub downloaded_pack_source: RefCell<DownloadedPackSource>,
    /// `particleEngine`: what the level's particles are handed to, the render track's to come.
    pub particle_engine: RefCell<Box<dyn ParticleEngine>>,
    /// `getSoundManager()`: what the level's sounds are handed to; sound isn't ported.
    sound_manager: Box<dyn SoundManager>,
    /// `getBlockColors()`, which leaves' particles take their colour from.
    block_colors: BlockColors,
    /// The window's size in pixels (`getWindow().getWidth()`), the renderer's when there's one.
    window_size: Cell<(i32, i32)>,
    /// `font`, for a client without a renderer: the fonts laid out without pages on a device.
    headless_font: OnceCell<Rc<Font>>,
    /// `missTime`.
    miss_time: Cell<i32>,
}

impl Minecraft {
    pub fn new(user: User, options: Options) -> Rc<Minecraft> {
        // A new game's fluids keep no shapes yet (`FlowingFluid.shapes`, a JVM's in Java).
        flowing_fluid::clear_shapes();
        input_type::set_last_input_type(InputType::NONE);
        Rc::new_cyclic(|this| Minecraft {
            this: this.clone(),
            options: RefCell::new(options),
            user,
            packet_processor: Arc::new(PacketProcessor::new()),
            player: RefCell::new(None),
            level: RefCell::new(None),
            game_mode: RefCell::new(None),
            camera_entity: Cell::new(None),
            removed_screens: RefCell::new(Vec::new()),
            // Java's starts at 0, so the first frame runs its 10 ticks at once.
            delta_tracker: RefCell::new(Timer::new(20.0, 0)),
            running: Cell::new(true),
            pause: Cell::new(false),
            client_tick_count: Cell::new(0),
            test_driver: RefCell::new(None),
            game_renderer: RefCell::new(None),
            window: RefCell::new(None),
            hud_hidden: Cell::new(false),
            mouse_handler: RefCell::new(MouseHandler::new()),
            gui: RefCell::new(Gui::new()),
            downloaded_pack_source: RefCell::new(DownloadedPackSource::new()),
            particle_engine: RefCell::new(Box::new(NoParticles)),
            sound_manager: Box::new(NoSound),
            block_colors: BlockColors::create_default(),
            window_size: Cell::new((854, 480)),
            headless_font: OnceCell::new(),
            miss_time: Cell::new(0),
        })
    }

    /// The game as its screens hold it.
    pub fn rc(&self) -> Rc<Minecraft> {
        self.this.upgrade().expect("the game is alive")
    }

    /// `font`: the game renderer's, or one laid out without a device for a client that has none.
    pub fn font(&self) -> Rc<Font> {
        if let Some(game_renderer) = self.game_renderer.borrow().as_ref() {
            return Rc::clone(game_renderer.font());
        }
        Rc::clone(self.headless_font.get_or_init(|| {
            let resources = GameRenderer::client_resources();
            Rc::new(Font::new(
                Rc::new(FontManager::load(None, &resources, &[])),
                false,
            ))
        }))
    }

    /// `fontFilterFishy`: the game renderer's, or the headless font, which has every glyph.
    pub fn font_filter_fishy(&self) -> Rc<Font> {
        match self.game_renderer.borrow().as_ref() {
            Some(game_renderer) => Rc::clone(game_renderer.font_filter_fishy()),
            None => self.font(),
        }
    }

    /// The window's size in pixels.
    pub fn window_size(&self) -> (i32, i32) {
        self.window_size.get()
    }

    /// `getWindow().getGuiScaledWidth()` and `getGuiScaledHeight()`, for the options' GUI scale.
    pub fn gui_scaled_size(&self) -> (i32, i32) {
        let (width, height) = self.window_size.get();
        let scale = Window::calculate_scale(width, height, self.options.borrow().gui_scale, false);
        Window::gui_scaled_size(width, height, scale)
    }

    /// `gui.screen()`.
    pub fn screen(&self) -> Option<ScreenRef> {
        self.gui.borrow().screen.clone()
    }

    /// `gui.setScreen`: the screen shown (none goes back to the game, or to the chat screen a
    /// sleeping player left), the last one removed and the new one added and made at the window's
    /// GUI size, the mouse released for it or grabbed for the game.
    pub fn set_screen(&self, screen: Option<ScreenRef>) {
        let old = self.gui.borrow_mut().screen.take();
        match old {
            Some(old) => {
                let removed = match old.try_borrow_mut() {
                    Ok(mut old) => {
                        old.screen_mut().clear_focus();
                        old.removed();
                        true
                    }
                    Err(_) => false,
                };
                // One handling what replaced it is told when it's done.
                if !removed {
                    self.removed_screens.borrow_mut().push(old);
                }
            }
            None => input_type::set_last_input_type(InputType::NONE),
        }

        let screen = match screen {
            Some(screen) => Some(screen),
            // The title screen, when there's no level, isn't ported yet; a dead player's death
            // screen isn't either.
            None if self.level.borrow().is_none() => None,
            None => self.gui.borrow_mut().hud.chat.restore_chat_screen(),
        };
        {
            let mut gui = self.gui.borrow_mut();
            gui.screen = screen.clone();
            gui.chat_screen_shown = screen
                .as_ref()
                .is_some_and(|screen| screen.borrow().as_any().is::<ChatScreen>());
        }
        if let Some(screen) = &screen {
            screen.borrow_mut().added();
        }
        // The text input manager's: typed text comes while a screen is up (Java starts it when an
        // edit box gets the focus).
        if let Some((window, _)) = self.window.borrow().as_ref() {
            if screen.is_some() {
                SDLKeyboard::SDL_StartTextInput(window.handle());
            } else {
                SDLKeyboard::SDL_StopTextInput(window.handle());
            }
        }
        match screen {
            Some(screen) => {
                self.release_mouse();
                self.options.borrow_mut().release_all_keys();
                let (width, height) = self.gui_scaled_size();
                screen::init(&mut *screen.borrow_mut(), width, height);
            }
            None => self.grab_mouse(),
        }
        self.mouse_handler.borrow_mut().resync_mouse_position();
    }

    /// The screens replaced while they were handling something, told now that they're done.
    fn finish_removed_screens(&self) {
        let removed: Vec<ScreenRef> = self.removed_screens.borrow_mut().drain(..).collect();
        for screen in removed {
            let mut screen = screen.borrow_mut();
            screen.screen_mut().clear_focus();
            screen.removed();
        }
    }

    /// Runs `handle` on the screen, when there's one, and then tells a screen it replaced so.
    pub fn with_screen<R>(
        &self,
        handle: impl FnOnce(&mut dyn screen::ScreenClass) -> R,
    ) -> Option<R> {
        let screen = self.screen()?;
        let result = handle(&mut *screen.borrow_mut());
        self.finish_removed_screens();
        Some(result)
    }

    /// `pauseGame`: the pause menu, whose game doesn't pause on a server
    /// (`canGameReallyBePaused` needs a singleplayer server).
    pub fn pause_game(&self, suppress_pause_menu_if_we_really_are_pausing: bool) {
        let _ = suppress_pause_menu_if_we_really_are_pausing;
        if self.screen().is_none() {
            let pause_screen = PauseScreen::new(&self.rc(), true);
            self.set_screen(Some(Rc::new(RefCell::new(pause_screen))));
        }
    }

    /// `isLocalServer`: the port only joins servers.
    pub fn is_local_server(&self) -> bool {
        false
    }

    /// `allowsTelemetry`: the user's properties' `TELEMETRY_ENABLED` flag, which an offline user's
    /// haven't got (`UserApiService.OFFLINE`).
    pub fn allows_telemetry(&self) -> bool {
        false
    }

    /// `mouseHandler.grabMouse()`: the window's mouse grabbed for turning, the screen gone.
    pub fn grab_mouse(&self) {
        if self.mouse_handler.borrow().is_mouse_grabbed() {
            return;
        }
        let window_size = self.window_size.get();
        self.mouse_handler.borrow_mut().grab_mouse(window_size);
        if let Some((window, _)) = self.window.borrow().as_ref() {
            SDLMouse::SDL_SetWindowRelativeMouseMode(window.handle(), true);
        }
        if self.screen().is_some() {
            self.set_screen(None);
        }
        self.miss_time.set(10000);
    }

    /// `mouseHandler.releaseMouse()`.
    pub fn release_mouse(&self) {
        if !self.mouse_handler.borrow().is_mouse_grabbed() {
            return;
        }
        let window_size = self.window_size.get();
        self.mouse_handler.borrow_mut().release_mouse(window_size);
        if let Some((window, _)) = self.window.borrow().as_ref() {
            SDLMouse::SDL_SetWindowRelativeMouseMode(window.handle(), false);
        }
    }

    /// `disconnectFromWorld`: leaves the server, for the title screen, which isn't ported yet, so
    /// the game stops at the disconnected screen.
    pub fn disconnect_from_world(&self) {
        if let Some(connection) = self.get_connection() {
            let connection = connection.borrow().get_connection();
            connection.disconnect(DisconnectionDetails::translatable(
                "multiplayer.status.quitting",
            ));
        }
        self.disconnect(
            &DisconnectionDetails::translatable("multiplayer.status.quitting"),
            false,
        );
    }

    /// `gui.openChatScreen`: the chat screen with the draft or the method's prefix.
    pub fn open_chat_screen(&self, chat_method: ChatMethod) {
        if self.player.borrow().is_some() {
            let (text, is_draft) = self.gui.borrow().hud.chat.screen_text(chat_method);
            let chat_screen = ChatScreen::new(&self.rc(), text, is_draft);
            self.set_screen(Some(Rc::new(RefCell::new(chat_screen))));
        }
    }

    /// What the chat reads of the game.
    pub fn chat_context<R>(&self, f: impl FnOnce(&ChatContext) -> R) -> R {
        let options = self.options.borrow();
        let font = self.font();
        let chat_focused = self.is_chat_focused();
        let gui_ticks = self.gui.borrow().hud.get_gui_ticks();
        f(&ChatContext {
            options: &options,
            font: &font,
            gui_ticks,
            chat_focused,
        })
    }

    /// `ChatListener.handleSystemMessage`'s chat line: the server's system message added to the
    /// chat.
    pub fn add_server_system_message(
        &self,
        message: crate::net::minecraft::network::chat::component::Component,
    ) {
        let options = self.options.borrow();
        let font = self.font();
        let chat_focused = self.is_chat_focused();
        let mut gui = self.gui.borrow_mut();
        let gui_ticks = gui.hud.get_gui_ticks();
        gui.hud.chat.add_server_system_message(
            message,
            &ChatContext {
                options: &options,
                font: &font,
                gui_ticks,
                chat_focused,
            },
        );
    }

    /// `isChatFocused`: the chat screen is shown.
    pub fn is_chat_focused(&self) -> bool {
        self.gui.borrow().chat_screen_shown
    }

    pub fn get_user(&self) -> &User {
        &self.user
    }

    pub fn packet_processor(&self) -> &Arc<PacketProcessor> {
        &self.packet_processor
    }

    pub fn set_test_driver(&self, driver: Box<dyn TestDriver>) {
        *self.test_driver.borrow_mut() = Some(driver);
    }

    /// Frames until the game stops: each runs a tick if one is due and processes the packets
    /// queued for the game thread, then waits for the next frame (`renderFrame`'s frame rate
    /// limit, under 260).
    pub fn run(self: &Rc<Minecraft>) {
        while self.running.get() {
            let frame = bench::now();
            self.run_tick(true);
            self.handle_accumulated_movement();
            self.render_frame();
            self.poll_events();
            let framerate_limit = self.options.borrow().framerate_limit;
            if framerate_limit < 260 {
                FramerateLimiter::limit_display_fps(framerate_limit);
            }
            bench::record(bench::Kind::Frame, frame);
        }
    }

    pub fn stop(&self) {
        self.running.set(false);
    }

    pub fn is_running(&self) -> bool {
        self.running.get()
    }

    fn run_tick(self: &Rc<Minecraft>, advance_game_time: bool) {
        let mut ticks_to_do = if advance_game_time {
            self.delta_tracker
                .borrow_mut()
                .advance_game_time(Util::get_millis())
        } else {
            0
        };
        // The oracle client runs under Fabric's client game tests, which do at most one tick a
        // frame (its `MinecraftMixin.captureTicksPerFrame`); the port's scenario runs do the same.
        if self.test_driver.borrow().is_some() {
            ticks_to_do = ticks_to_do.min(1);
        }
        if advance_game_time {
            self.packet_processor.process_queued_packets();
            // `isLevelRunningNormally()`: the level's tick rate isn't ported, so it always runs.
            if ticks_to_do > 0
                && let Some(game_renderer) = self.game_renderer.borrow_mut().as_mut()
            {
                game_renderer.tick_textures();
            }
            for _ in 0..ticks_to_do.min(10) {
                self.drive(|driver, minecraft| driver.before_tick(minecraft));
                self.tick();
            }
        }

        // `renderFrame`'s update, before the frame is drawn: the level's light.
        if advance_game_time && let Some(level) = self.level.borrow_mut().as_mut() {
            let start = bench::now();
            level.update();
            bench::record(bench::Kind::Light, start);
        }

        self.pause.set(false);
        let mut delta_tracker = self.delta_tracker.borrow_mut();
        delta_tracker.update_pause_state(self.pause.get());
        delta_tracker.update_frozen_state(false);
    }

    pub fn tick(self: &Rc<Minecraft>) {
        let start = bench::now();
        self.tick_game();
        bench::record(bench::Kind::Tick, start);
    }

    fn tick_game(self: &Rc<Minecraft>) {
        self.client_tick_count
            .set(self.client_tick_count.get().wrapping_add(1));
        let has_level = self.level.borrow().is_some();
        if !self.pause.get() && has_level {
            let game_mode = self.game_mode.borrow().clone();
            if let Some(game_mode) = game_mode {
                game_mode.tick();
            }
        }

        self.gui.borrow_mut().tick(self.pause.get());
        // `gui.tick()`'s screen tick, outside the gui's cell.
        self.with_screen(|screen| screen.tick());
        if self.screen().is_some() {
            self.miss_time.set(10000);
        }
        if self.screen().is_none() {
            self.handle_keybinds();
            if self.miss_time.get() > 0 {
                self.miss_time.set(self.miss_time.get() - 1);
            }
        }

        if self.level.borrow().is_some() {
            if !self.pause.get() {
                if let Some(game_renderer) = self.game_renderer.borrow_mut().as_mut() {
                    let options = GameRenderer::extract_options(
                        &self.options.borrow(),
                        self.hud_hidden.get(),
                    );
                    let level = self.level.borrow();
                    let level = level.as_ref().expect("checked above");
                    self.ensure_environment_attributes(game_renderer, level);
                    let environment = self.level_environment(level);
                    game_renderer.tick(
                        true,
                        self.player.borrow().as_ref(),
                        Some(&environment),
                        &options,
                    );
                }
                // `level.tickEntities()` (`ClientLevel.tickNonPassenger`): the player first, as
                // the first entity the level has, then the others in the order they came.
                {
                    let level = self.level.borrow();
                    let mut player = self.player.borrow_mut();
                    if let (Some(level), Some(player)) = (level.as_ref(), player.as_mut()) {
                        player.common_tick();
                        player.tick(level, &self.options.borrow());
                    }
                }
                if let Some(level) = self.level.borrow_mut().as_mut() {
                    level.tick_entities();
                }
                // `level.tickBlockEntities()`.
                self.with_level_ticking(|level| {
                    let (level, effects, player) = (level.level, level.effects, level.player);
                    level.tick_block_entities(effects, player);
                });
                // `playerSync`.
                let mut player = self.player.borrow_mut();
                if let Some(player) = player.as_mut() {
                    player.send_changes(self);
                }
            }

            if !self.pause.get() {
                if !self.options.borrow().joined_first_server {
                    self.options.borrow_mut().joined_first_server = true;
                }

                // `level.tick`, with the listener's clocks, the sky's darkening first
                // (`updateSkyBrightness`) from the environment attributes.
                let sky_light_level = self.sky_light_level();
                if let (Some(level), Some(connection)) =
                    (self.level.borrow_mut().as_mut(), self.get_connection())
                {
                    level.update_sky_brightness(sky_light_level);
                    level.tick(connection.borrow_mut().clock_manager_mut());
                }
            }

            // `level.animateTick` around the player; `isLevelRunningNormally()`'s tick rate
            // isn't ported, so it always runs.
            if !self.pause.get() {
                self.animate_tick();
            }

            if let Some(connection) = self.get_connection()
                && !self.pause.get()
            {
                connection
                    .borrow()
                    .send(ServerboundClientTickEndPacket::INSTANCE);
            }
        }

        self.drive(|driver, minecraft| driver.after_tick(minecraft));
    }

    /// The environment attributes' `SKY_LIGHT_LEVEL` for the level (`getDimensionValue`), its
    /// default before the game renderer has made them, or without one.
    fn sky_light_level(&self) -> f32 {
        let attribute = &EnvironmentAttributes::SKY_LIGHT_LEVEL;
        let level = self.level.borrow();
        let game_renderer = self.game_renderer.borrow();
        match (
            level.as_ref(),
            game_renderer
                .as_ref()
                .and_then(GameRenderer::environment_attributes),
        ) {
            (Some(level), Some(system)) => {
                system.get_dimension_value(attribute, &self.level_environment(level))
            }
            _ => {
                f32::from_attribute_value(EnvironmentAttributes::info(attribute.id()).default_value)
            }
        }
    }

    /// `level.animateTick(player.getBlockX(), player.getBlockY(), player.getBlockZ())`, its
    /// particles and sounds handed to the game's particle engine and sound manager, near the
    /// camera: the game renderer's, or the player's eyes for a headless client, which has none.
    /// Java's marker particles show for a barrier or light block held in creative; items aren't
    /// ported, so there's none.
    fn animate_tick(&self) {
        let level = self.level.borrow();
        let player = self.player.borrow();
        let (Some(level), Some(player)) = (level.as_ref(), player.as_ref()) else {
            return;
        };
        let game_renderer = self.game_renderer.borrow();
        let environment = self.level_environment(level);
        let particle_engine = self.particle_engine.borrow();
        let effects = LevelEffects {
            particle_engine: particle_engine.as_ref(),
            sound_manager: self.sound_manager.as_ref(),
            camera_position: game_renderer.as_ref().map_or_else(
                || Vec3::new(player.get_x(), player.get_eye_y(), player.get_z()),
                |game_renderer| game_renderer.main_camera().position(),
            ),
            particles: self.options.borrow().particles(),
            player: Some(player.get_id()),
            environment: game_renderer
                .as_ref()
                .and_then(GameRenderer::environment_attributes)
                .map(|system| (system, &environment as &dyn LayerLevel)),
            block_colors: &self.block_colors,
        };
        let pos = player.block_position();
        level.animate_tick(pos.get_x(), pos.get_y(), pos.get_z(), None, &effects);
    }

    /// The level as a block event or a block entity's ticker reaches it (`LevelTicking`): the
    /// level, the player, and the game's particle engine and sound manager, near the camera (the
    /// game renderer's, or the player's eyes for a headless client). Its effects have no
    /// environment attributes, which borrow the level: no ticker or block event ported reads
    /// them. None before the level and the player are there.
    pub fn with_level_ticking<R>(&self, f: impl FnOnce(LevelTicking<'_>) -> R) -> Option<R> {
        let mut level = self.level.borrow_mut();
        let mut player = self.player.borrow_mut();
        let (Some(level), Some(player)) = (level.as_mut(), player.as_mut()) else {
            return None;
        };
        let game_renderer = self.game_renderer.borrow();
        let particle_engine = self.particle_engine.borrow();
        let effects = LevelEffects {
            particle_engine: particle_engine.as_ref(),
            sound_manager: self.sound_manager.as_ref(),
            camera_position: game_renderer.as_ref().map_or_else(
                || Vec3::new(player.get_x(), player.get_eye_y(), player.get_z()),
                |game_renderer| game_renderer.main_camera().position(),
            ),
            particles: self.options.borrow().particles(),
            player: Some(player.get_id()),
            environment: None,
            block_colors: &self.block_colors,
        };
        Some(f(LevelTicking {
            level,
            effects: &effects,
            player: Some(player),
        }))
    }

    /// Hands the game to the test driver, if there's one.
    fn drive(self: &Rc<Minecraft>, call: impl FnOnce(&mut dyn TestDriver, &Rc<Minecraft>)) {
        let driver = self.test_driver.borrow_mut().take();
        if let Some(mut driver) = driver {
            call(driver.as_mut(), self);
            let mut slot = self.test_driver.borrow_mut();
            if slot.is_none() {
                *slot = Some(driver);
            }
        }
    }

    /// `handleKeybinds`, as far as it's ported: F1 is the keyboard's, and the chat's keys open the
    /// chat screen (`gui.handleKeybinds`).
    fn handle_keybinds(&self) {
        while self.options.borrow_mut().key_chat.consume_click() {
            self.open_chat_screen(ChatMethod::MESSAGE);
        }
        if self.screen().is_none() && self.options.borrow_mut().key_command.consume_click() {
            self.open_chat_screen(ChatMethod::COMMAND);
        }
    }

    /// The client's listener of the game protocol, once the player is in a level:
    /// `player.connection`.
    pub fn get_connection(&self) -> Option<Rc<RefCell<ClientPacketListener>>> {
        self.player
            .borrow()
            .as_ref()
            .and_then(|player| player.connection.clone())
    }

    pub fn get_camera_entity(&self) -> Option<i32> {
        self.camera_entity.get()
    }

    pub fn set_camera_entity(&self, camera_entity: Option<i32>) {
        self.camera_entity.set(camera_entity);
    }

    pub fn set_level(&self, level: ClientLevel) {
        *self.level.borrow_mut() = Some(level);
        self.camera_entity.set(None);
        if let Some(game_renderer) = self.game_renderer.borrow_mut().as_mut() {
            game_renderer.set_level(true);
        }
    }

    /// Makes the renderer, of the scenario's or the window's size, on the device, and the
    /// window frames are presented to, if there's one.
    pub fn init_renderer(
        &self,
        game_renderer: GameRenderer,
        window: Option<(Window, Box<dyn GpuSurface>)>,
    ) {
        let target = game_renderer.main_render_target();
        self.window_size.set((target.width, target.height));
        *self.game_renderer.borrow_mut() = Some(game_renderer);
        // `renderFrame`'s first configuration of the surface: the present mode vsync picks.
        if let Some((_, surface)) = &window {
            surface.configure(PresentMode::get_supported_vsync_mode(
                surface.supported_present_modes(),
                self.options.borrow().enable_vsync,
            ));
        }
        *self.window.borrow_mut() = window;
    }

    /// `GameRenderer.update`, `extract` and `render` of a frame of the level at `partial_ticks`,
    /// when there's a renderer and a level; true if it rendered.
    pub fn render_level(&self, partial_ticks: f32) -> bool {
        let mut game_renderer = self.game_renderer.borrow_mut();
        let level = self.level.borrow();
        let player = self.player.borrow();
        let (Some(game_renderer), Some(level)) = (game_renderer.as_mut(), level.as_ref()) else {
            return false;
        };
        if let Some((window, _)) = self.window.borrow().as_ref() {
            game_renderer.follow_window_size(window.get_width(), window.get_height());
        }
        let options = GameRenderer::extract_options(&self.options.borrow(), self.hud_hidden.get());
        self.ensure_environment_attributes(game_renderer, level);
        game_renderer.update(
            player.as_ref(),
            Some(level as &dyn BlockGetter),
            partial_ticks,
            &options,
        );
        let delta_ticks = self.delta_tracker.borrow().get_game_time_delta_ticks();
        let eye_y = player.as_ref().map_or(0.0, |player| {
            Mth::lerp_f64(
                f64::from(partial_ticks),
                player.old_position().y,
                player.get_y(),
            ) + f64::from(player.get_eye_height())
        });
        game_renderer.render_level_frame(
            level,
            level,
            &self.level_environment(level),
            player.as_ref(),
            &options,
            partial_ticks,
            delta_ticks,
            level.get_game_time(),
            eye_y,
            &CameraEntityState::of(player.as_ref()),
        );
        self.render_gui(game_renderer, level, player.as_ref(), partial_ticks);
        true
    }

    /// The frame's GUI: `Gui.extractRenderState` (the HUD, with the level and its player, then the
    /// screen at the mouse's scaled position), then `GameRenderer.render`'s `guiRenderer.render()`.
    fn render_gui(
        &self,
        game_renderer: &mut GameRenderer,
        level: &ClientLevel,
        player: Option<&LocalPlayer>,
        partial_ticks: f32,
    ) {
        let gui_scale = self.options.borrow().gui_scale;
        let game_mode = self.game_mode.borrow().clone();
        let font = std::rc::Rc::clone(game_renderer.font());
        let level_loading = self.screen().is_some_and(|screen| {
            screen
                .try_borrow()
                .is_ok_and(|screen| screen.as_any().is::<LevelLoadingScreen>())
        });
        let hud_context = match (player, game_mode.as_ref()) {
            (Some(player), Some(game_mode)) if !level_loading => Some(HudContext {
                font: &font,
                is_hidden: self.hud_hidden.get(),
                player,
                player_mode: game_mode.get_player_mode(),
                can_hurt_player: game_mode.can_hurt_player(),
                has_experience: game_mode.has_experience(),
                is_hardcore: level.is_hardcore(),
                options: &self.options.borrow(),
                chat_focused: self.is_chat_focused(),
            }),
            _ => None,
        };
        let (mouse_x, mouse_y) = self.scaled_mouse_position();
        let screen = self.screen();
        let text_background_color = self.options.borrow().get_background_color(0.0);
        game_renderer.extract_gui(
            gui_scale,
            text_background_color,
            mouse_x,
            mouse_y,
            |graphics| {
                self.gui
                    .borrow_mut()
                    .extract_render_state(graphics, hud_context.as_ref());
                if let Some(screen) = &screen {
                    screen::extract_render_state_with_tooltip_and_subtitles(
                        &mut *screen.borrow_mut(),
                        graphics,
                        mouse_x,
                        mouse_y,
                        partial_ticks,
                    );
                }
            },
        );
        self.finish_removed_screens();
        game_renderer.render_gui(gui_scale);
    }

    /// The mouse's place in the GUI's scaled coordinates, as `Gui.extractRenderState` reads it.
    pub fn scaled_mouse_position(&self) -> (i32, i32) {
        let window_size = self.window_size.get();
        let gui_size = self.gui_scaled_size();
        let mouse_handler = self.mouse_handler.borrow();
        (
            mouse_handler.get_scaled_x_pos(window_size, gui_size) as i32,
            mouse_handler.get_scaled_y_pos(window_size, gui_size) as i32,
        )
    }

    /// `MouseHandler.handleAccumulatedMovement`: the mouse moved over the screen, or turning the
    /// player while grabbed.
    fn handle_accumulated_movement(&self) {
        let window_size = self.window_size.get();
        let gui_size = self.gui_scaled_size();
        let moved = self.mouse_handler.borrow().has_accumulated_movement();
        if moved && self.screen().is_some() {
            let (x, y) = {
                let mouse_handler = self.mouse_handler.borrow();
                (
                    mouse_handler.get_scaled_x_pos(window_size, gui_size),
                    mouse_handler.get_scaled_y_pos(window_size, gui_size),
                )
            };
            self.with_screen(|screen| screen.mouse_moved(x, y));
        }
        self.mouse_handler
            .borrow_mut()
            .handle_accumulated_movement(self);
    }

    /// A key as the window or a scenario gives it (`KeyboardHandler.keyPress`).
    pub fn key_press(&self, event: &KeyEvent, action: KeyAction) {
        KeyboardHandler::key_press(self, event, action);
    }

    /// A character typed (`KeyboardHandler.charTyped`): the screen's.
    pub fn char_typed(&self, event: &CharacterEvent) {
        self.with_screen(|screen| screen.char_typed(event));
    }

    /// `MouseHandler.onMove` at a place in the window.
    pub fn mouse_moved_to(&self, x: f64, y: f64) {
        self.mouse_handler.borrow_mut().on_move_to(x, y);
    }

    /// `MouseHandler.onButton`: a click or a release over the screen.
    pub fn mouse_button(&self, button: i32, pressed: bool) {
        if self.screen().is_none() {
            if pressed && !self.mouse_handler.borrow().is_mouse_grabbed() {
                self.grab_mouse();
            }
            return;
        }
        input_type::set_last_input_type(InputType::MOUSE);
        let window_size = self.window_size.get();
        let gui_size = self.gui_scaled_size();
        let (x, y) = {
            let mouse_handler = self.mouse_handler.borrow();
            (
                mouse_handler.get_scaled_x_pos(window_size, gui_size),
                mouse_handler.get_scaled_y_pos(window_size, gui_size),
            )
        };
        let event = MouseButtonEvent::new(x, y, MouseButtonInfo::new(button, 0));
        if pressed {
            let Some(screen) = self.screen() else {
                return;
            };
            let now = Instant::now();
            let double_click = self
                .mouse_handler
                .borrow()
                .is_double_click(&screen, button, now);
            if self.with_screen(|screen| screen.mouse_clicked(&event, double_click)) == Some(true) {
                self.mouse_handler
                    .borrow_mut()
                    .set_last_click(&screen, button, now);
            }
        } else {
            self.with_screen(|screen| screen.mouse_released(&event));
        }
    }

    /// What the level's environment attributes read of it, with the listener's clocks.
    fn level_environment<'a>(&self, level: &'a ClientLevel) -> LevelEnvironment<'a> {
        let clocks = self
            .get_connection()
            .map(|connection| connection.borrow().clock_manager().clone())
            .unwrap_or_default();
        LevelEnvironment::new(level, clocks)
    }

    /// The level's environment attributes, made once it's there: its dimension type's, from the
    /// vanilla pack, with the server's biome registry. Java's level makes them in its constructor.
    fn ensure_environment_attributes(&self, game_renderer: &mut GameRenderer, level: &ClientLevel) {
        if !game_renderer.needs_environment_attributes() {
            return;
        }
        let Some(connection) = self.get_connection() else {
            return;
        };
        let connection = connection.borrow();
        let biomes = connection
            .registry_access()
            .lookup_or_throw(Registries::BIOME);
        let biome_keys: Vec<ResourceKey> = (0..biomes.size())
            .map(|id| {
                biomes
                    .holder_by_id_or_throw(id)
                    .unwrap_key()
                    .expect("a registry's holder has its key")
                    .clone()
            })
            .collect();
        let dimension_type = level
            .dimension_type_registration()
            .unwrap_key()
            .expect("a registry's holder has its key")
            .identifier()
            .clone();
        // Level.canHaveWeather: sky light, no ceiling, not the end; of the vanilla types, the
        // overworld's.
        let can_have_weather = dimension_type == Identifier::with_default_namespace("overworld");
        match EnvironmentAttributeSystem::for_client_level(
            &dimension_type,
            &biome_keys,
            can_have_weather,
        ) {
            Ok(system) => game_renderer.set_environment_attributes(system),
            Err(error) => panic!("the level's environment attributes: {error}"),
        }
    }

    /// `RenderSystem.pollEvents`, with the window's input: the keyboard's and the mouse's
    /// (`KeyboardHandler`, `MouseHandler`), text typed, losing the focus, and closing the window.
    fn poll_events(&self) {
        let mut events = Vec::new();
        {
            let window = self.window.borrow();
            if window.is_none() {
                return;
            }
            let mut event = SdlEvent::default();
            while SDLEvents::SDL_PollEvent(&mut event) {
                // The text's SDL's until the next poll.
                let text = (event.event_type == SDLEvents::SDL_EVENT_TEXT_INPUT)
                    .then(|| event.text_input());
                events.push((std::mem::take(&mut event), text));
            }
        }
        for (event, text) in events {
            match event.event_type {
                SDLEvents::SDL_EVENT_QUIT | SDLEvents::SDL_EVENT_WINDOW_CLOSE_REQUESTED => {
                    self.stop();
                }
                SDLEvents::SDL_EVENT_KEY_DOWN | SDLEvents::SDL_EVENT_KEY_UP => {
                    let action = if event.event_type == SDLEvents::SDL_EVENT_KEY_UP {
                        KeyAction::RELEASE
                    } else if event.key_repeat() {
                        KeyAction::REPEAT
                    } else {
                        KeyAction::PRESS
                    };
                    let key_event = KeyEvent::new(
                        event.key_scancode(),
                        event.key_keycode(),
                        event.key_modifiers(),
                    );
                    self.key_press(&key_event, action);
                }
                SDLEvents::SDL_EVENT_TEXT_INPUT => {
                    for c in text.unwrap_or_default().chars() {
                        self.char_typed(&CharacterEvent::new(c as i32));
                    }
                }
                SDLEvents::SDL_EVENT_MOUSE_MOTION => {
                    let (x, y) = event.motion_position();
                    let (x_relative, y_relative) = event.motion_relative();
                    self.mouse_handler.borrow_mut().on_move(
                        f64::from(x),
                        f64::from(y),
                        f64::from(x_relative),
                        f64::from(y_relative),
                    );
                }
                SDLEvents::SDL_EVENT_MOUSE_BUTTON_DOWN | SDLEvents::SDL_EVENT_MOUSE_BUTTON_UP => {
                    let pressed = event.event_type == SDLEvents::SDL_EVENT_MOUSE_BUTTON_DOWN;
                    self.mouse_button(i32::from(event.mouse_button()), pressed);
                }
                SDLEvents::SDL_EVENT_WINDOW_RESIZED => {
                    let (width, height) = event.window_data();
                    if let Some((window, _)) = self.window.borrow_mut().as_mut() {
                        window.on_resize(width, height);
                    }
                }
                SDLEvents::SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED => {
                    // framebufferSizeChanged: the surface follows the window's pixels when it
                    // blits, and the GUI's scale, the main target and the screen follow its size
                    // each frame.
                    let (width, height) = event.window_data();
                    if let Some((window, _)) = self.window.borrow_mut().as_mut() {
                        window.on_framebuffer_resize(width, height);
                    }
                }
                SDLEvents::SDL_EVENT_WINDOW_FOCUS_LOST => {
                    // `onFocusChanged`: a game without a screen pauses (the pause menu).
                    if self.screen().is_none() && self.level.borrow().is_some() {
                        self.pause_game(false);
                    }
                    self.release_mouse();
                }
                _ => {}
            }
        }
    }

    /// `resizeGui`, when the window's size changed: the GUI's size follows it, and the screen is
    /// placed again (`screen.resize`).
    fn follow_window_size(&self) {
        let size = self
            .window
            .borrow()
            .as_ref()
            .map(|(window, _)| (window.get_width(), window.get_height()));
        if let Some(size) = size
            && size != self.window_size.get()
        {
            self.window_size.set(size);
            let (width, height) = self.gui_scaled_size();
            self.with_screen(|screen| screen::resize(screen, width, height));
        }
    }

    /// `renderFrame`'s render: a frame of the level, blitted to the window if there's one, the
    /// frame's commands submitted, then the window's image presented.
    fn render_frame(&self) {
        self.follow_window_size();
        let partial_ticks = self
            .delta_tracker
            .borrow()
            .get_game_time_delta_partial_tick(false);
        if !self.render_level(partial_ticks) {
            return;
        }
        if let Some((_, surface)) = self.window.borrow().as_ref()
            && let Some(game_renderer) = self.game_renderer.borrow().as_ref()
        {
            surface.blit_from_texture(
                game_renderer
                    .main_render_target()
                    .get_color_texture_view()
                    .expect("the main target has color"),
            );
            bench::mark(bench::Kind::Submit);
            game_renderer.device().create_command_encoder().submit();
            surface.present();
        } else if let Some(game_renderer) = self.game_renderer.borrow().as_ref() {
            game_renderer.device().create_command_encoder().submit();
        }
    }

    /// `setScreenAndShow(new LevelLoadingScreen(…))`.
    pub fn show_level_loading(&self) {
        let screen = LevelLoadingScreen::new(&self.rc());
        self.set_screen(Some(Rc::new(RefCell::new(screen))));
    }

    /// Whether the level's loading screen is up, which closes once the level is ready.
    pub fn is_level_loading(&self) -> bool {
        let loading_screen = self.screen().is_some_and(|screen| {
            screen
                .try_borrow()
                .is_ok_and(|screen| screen.as_any().is::<LevelLoadingScreen>())
        });
        loading_screen
            && self
                .get_connection()
                .is_some_and(|connection| !connection.borrow().has_client_loaded())
    }

    /// The disconnected screen, with its title and the reason. A client with no screens to go on
    /// to stops there.
    pub fn show_disconnected(&self, title: &str, details: &DisconnectionDetails) {
        eprintln!("{title}: {}", details.reason_text());
        let screen = DisconnectedScreen::new(&self.rc(), title, details.clone());
        self.set_screen(Some(Rc::new(RefCell::new(screen))));
        self.stop();
    }

    /// Why the game was disconnected, if it was.
    pub fn disconnection(&self) -> Option<(String, DisconnectionDetails)> {
        let screen = self.screen()?;
        let screen = screen.try_borrow().ok()?;
        let disconnected = screen.as_any().downcast_ref::<DisconnectedScreen>()?;
        Some((disconnected.title_key.clone(), disconnected.details.clone()))
    }

    /// `disconnect(Screen, boolean)`, going to the disconnected screen. The listener being closed
    /// is the one calling this when the connection ends, and closes itself.
    pub fn disconnect(&self, details: &DisconnectionDetails, keep_resource_packs: bool) {
        let _ = keep_resource_packs;
        if let Some(connection) = self.get_connection()
            && let Ok(mut connection) = connection.try_borrow_mut()
        {
            connection.close();
        }

        *self.game_mode.borrow_mut() = None;
        *self.level.borrow_mut() = None;
        *self.player.borrow_mut() = None;
        self.camera_entity.set(None);
        self.show_disconnected("disconnect.lost", details);
    }

    /// `clearClientLevel(Screen)`, going back to configuration. Java clears the listener's level
    /// first, which the listener calling this does itself.
    pub fn clear_client_level(&self) {
        *self.game_mode.borrow_mut() = None;
        *self.level.borrow_mut() = None;
        *self.player.borrow_mut() = None;
        self.camera_entity.set(None);
        let old = self.gui.borrow_mut().screen.take();
        if let Some(old) = old {
            let removed = match old.try_borrow_mut() {
                Ok(mut screen) => {
                    screen.removed();
                    true
                }
                Err(_) => false,
            };
            if !removed {
                self.removed_screens.borrow_mut().push(old);
            }
        }
        self.gui.borrow_mut().chat_screen_shown = false;
    }
}
