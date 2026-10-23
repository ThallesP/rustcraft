//! net/minecraft/client/Minecraft.java
//!
//! Not ported yet: nearly all of it: sound, the integrated server, resources, the GUI and its
//! screens, input, telemetry and the profiler. What's here is the game loop's timing and what a
//! tick does with a connection to a server: run the packets queued for the game thread, then up
//! to 10 ticks, each ticking the connection (the game mode's, or the connect screen's before
//! there's a level), the game renderer, the player on its level, what the player sends, and
//! ending with `client_tick_end`; then the frame's update of the level's light (`renderFrame`'s
//! `level.update()`) and, with a renderer, a frame of the level, offscreen or presented to the
//! window. The renderer is made on demand (`init_renderer`): a headless client that takes no
//! screenshots runs without one.
//!
//! The game is shared by its listeners and its player, like Java's, so its state is in cells, and
//! its back-references are parameters (`LocalPlayer::send_changes`). The screen is one of the few
//! the port has, without drawing anything: the connect screen, the level's loading screen, or the
//! disconnected screen, which a headless client stops at. Between frames, Java's Netty threads go
//! on reading the connection; here the game runs the connections' event loops until the next
//! frame is due (`EventLoopGroupHolder`), at vanilla's default frame rate limit.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::com::mojang::blaze3d::platform::window::Window;
use crate::com::mojang::renderpearl::api::device::gpu_surface::GpuSurface;
use crate::net::minecraft::client::delta_tracker::Timer;
use crate::net::minecraft::client::gui::gui::Gui;
use crate::net::minecraft::client::gui::hud::HudContext;
use crate::net::minecraft::client::gui::screens::connect_screen::ConnectScreen;
use crate::net::minecraft::client::keyboard_handler::{KeyAction, KeyboardHandler};
use crate::net::minecraft::client::mouse_handler::MouseHandler;
use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
use crate::net::minecraft::client::multiplayer::multi_player_game_mode::MultiPlayerGameMode;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::client::renderer::chunk::render_region_cache::LevelEnvironment;
use crate::net::minecraft::client::renderer::game_renderer::GameRenderer;
use crate::net::minecraft::client::user::User;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_processor::PacketProcessor;
use crate::net::minecraft::network::protocol::game::serverbound_client_tick_end_packet::ServerboundClientTickEndPacket;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::server::network::event_loop_group_holder::EventLoopGroupHolder;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::util::Util;
use crate::net::minecraft::world::attribute::environment_attribute_system::EnvironmentAttributeSystem;
use crate::net::minecraft::world::level::material::flowing_fluid;
use crate::org::lwjgl::sdl::sdl_events::{SDLEvents, SdlEvent};
use crate::org::lwjgl::sdl::sdl_mouse::SDLMouse;

/// The screen shown, of the ones the port has.
enum Screen {
    None,
    Connect(ConnectScreen),
    LevelLoading,
    Disconnected {
        title: String,
        details: DisconnectionDetails,
    },
}

/// Something that drives the game from outside, like Fabric's client game tests drive vanilla:
/// called after every tick (`tools/port_client.py`'s scenarios).
pub trait TestDriver {
    fn after_tick(&mut self, minecraft: &Rc<Minecraft>);
}

/// The client.
pub struct Minecraft {
    pub options: RefCell<Options>,
    user: User,
    packet_processor: PacketProcessor,
    pub player: RefCell<Option<LocalPlayer>>,
    pub level: RefCell<Option<ClientLevel>>,
    pub game_mode: RefCell<Option<Rc<MultiPlayerGameMode>>>,
    camera_entity: Cell<Option<i32>>,
    screen: RefCell<Screen>,
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
    mouse_handler: RefCell<MouseHandler>,
    pub gui: RefCell<Gui>,
}

impl Minecraft {
    /// Vanilla's default `framerateLimit`: frames are at most this often, and each processes the
    /// packets queued for the game thread.
    const FRAMERATE_LIMIT: u64 = 120;

    pub fn new(user: User, options: Options) -> Rc<Minecraft> {
        // A new game's fluids keep no shapes yet (`FlowingFluid.shapes`, a JVM's in Java).
        flowing_fluid::clear_shapes();
        Rc::new(Minecraft {
            options: RefCell::new(options),
            user,
            packet_processor: PacketProcessor::new(),
            player: RefCell::new(None),
            level: RefCell::new(None),
            game_mode: RefCell::new(None),
            camera_entity: Cell::new(None),
            screen: RefCell::new(Screen::None),
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
        })
    }

    pub fn get_user(&self) -> &User {
        &self.user
    }

    pub fn packet_processor(&self) -> &PacketProcessor {
        &self.packet_processor
    }

    pub fn set_test_driver(&self, driver: Box<dyn TestDriver>) {
        *self.test_driver.borrow_mut() = Some(driver);
    }

    /// Frames until the game stops: each runs a tick if one is due, then the network until the
    /// next frame.
    pub fn run(self: &Rc<Minecraft>) {
        let frame = Duration::from_nanos(1_000_000_000 / Minecraft::FRAMERATE_LIMIT);
        let mut next_frame = Instant::now();
        while self.running.get() {
            self.run_tick(true);
            self.mouse_handler
                .borrow_mut()
                .handle_accumulated_movement(self);
            self.render_frame();
            self.poll_events();
            next_frame += frame;
            let now = Instant::now();
            if next_frame < now {
                next_frame = now;
            }
            EventLoopGroupHolder::remote().run_until(next_frame);
        }
    }

    pub fn stop(&self) {
        self.running.set(false);
    }

    pub fn is_running(&self) -> bool {
        self.running.get()
    }

    fn run_tick(self: &Rc<Minecraft>, advance_game_time: bool) {
        let ticks_to_do = if advance_game_time {
            self.delta_tracker
                .borrow_mut()
                .advance_game_time(Util::get_millis())
        } else {
            0
        };
        if advance_game_time {
            self.packet_processor.process_queued_packets();
            for _ in 0..ticks_to_do.min(10) {
                self.tick();
            }
        }

        // `renderFrame`'s update, before the frame is drawn: the level's light.
        if advance_game_time && let Some(level) = self.level.borrow_mut().as_mut() {
            level.update();
        }

        self.pause.set(false);
        let mut delta_tracker = self.delta_tracker.borrow_mut();
        delta_tracker.update_pause_state(self.pause.get());
        delta_tracker.update_frozen_state(false);
    }

    pub fn tick(self: &Rc<Minecraft>) {
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
        self.tick_screen();

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
                // `playerSync`.
                let mut player = self.player.borrow_mut();
                if let Some(player) = player.as_mut() {
                    player.send_changes(self);
                }
            }

            if !self.options.borrow().joined_first_server {
                self.options.borrow_mut().joined_first_server = true;
            }

            if let Some(connection) = self.get_connection()
                && !self.pause.get()
            {
                connection
                    .borrow()
                    .send(ServerboundClientTickEndPacket::INSTANCE);
            }
        }

        let driver = self.test_driver.borrow_mut().take();
        if let Some(mut driver) = driver {
            driver.after_tick(self);
            let mut slot = self.test_driver.borrow_mut();
            if slot.is_none() {
                *slot = Some(driver);
            }
        }
    }

    /// `gui.tick()`'s screen tick: the connect screen ticks its connection.
    fn tick_screen(&self) {
        let connect_screen = match &*self.screen.borrow() {
            Screen::Connect(connect_screen) => Some(connect_screen.clone()),
            _ => None,
        };
        if let Some(connect_screen) = connect_screen {
            connect_screen.tick();
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
        *self.game_renderer.borrow_mut() = Some(game_renderer);
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
        let options = GameRenderer::extract_options(&self.options.borrow(), self.hud_hidden.get());
        self.ensure_environment_attributes(game_renderer, level);
        game_renderer.update(player.as_ref(), partial_ticks, &options);
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
            &options,
            partial_ticks,
            delta_ticks,
            level.get_game_time(),
            eye_y,
        );
        self.render_gui(game_renderer, level, player.as_ref());
        true
    }

    /// The frame's GUI: `Gui.extractRenderState` (the HUD, with the level and its player), then
    /// `GameRenderer.render`'s `guiRenderer.render()`. The mouse is grabbed in a level, which only
    /// screens read, so its scaled position is the window's corner until screens come.
    fn render_gui(
        &self,
        game_renderer: &mut GameRenderer,
        level: &ClientLevel,
        player: Option<&LocalPlayer>,
    ) {
        let gui_scale = self.options.borrow().gui_scale;
        let game_mode = self.game_mode.borrow().clone();
        let hud_context = match (player, game_mode.as_ref()) {
            (Some(player), Some(game_mode)) => Some(HudContext {
                is_hidden: self.hud_hidden.get(),
                player,
                player_mode: game_mode.get_player_mode(),
                can_hurt_player: game_mode.can_hurt_player(),
                has_experience: game_mode.has_experience(),
                is_hardcore: level.is_hardcore(),
            }),
            _ => None,
        };
        let mut gui = self.gui.borrow_mut();
        game_renderer.extract_gui(gui_scale, 0, 0, |graphics| {
            gui.extract_render_state(graphics, hud_context.as_ref());
        });
        game_renderer.render_gui(gui_scale);
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
    /// (`KeyboardHandler`, `MouseHandler`), a click grabbing the mouse, and closing the window.
    fn poll_events(&self) {
        let window = self.window.borrow();
        let Some((window, _)) = window.as_ref() else {
            return;
        };
        let mut event = SdlEvent::default();
        while SDLEvents::SDL_PollEvent(&mut event) {
            match event.event_type {
                SDLEvents::SDL_EVENT_QUIT | SDLEvents::SDL_EVENT_WINDOW_CLOSE_REQUESTED => {
                    self.stop();
                }
                SDLEvents::SDL_EVENT_KEY_DOWN | SDLEvents::SDL_EVENT_KEY_UP => {
                    let down = event.event_type == SDLEvents::SDL_EVENT_KEY_DOWN;
                    let action = KeyboardHandler::key_press(
                        self,
                        event.key_scancode(),
                        down,
                        event.key_repeat(),
                    );
                    if action == KeyAction::ReleaseMouse {
                        self.mouse_handler.borrow_mut().set_mouse_grabbed(false);
                        SDLMouse::SDL_SetWindowRelativeMouseMode(window.handle(), false);
                    }
                }
                SDLEvents::SDL_EVENT_MOUSE_MOTION => {
                    let (x, y) = event.motion_relative();
                    self.mouse_handler
                        .borrow_mut()
                        .on_move(f64::from(x), f64::from(y));
                }
                SDLEvents::SDL_EVENT_MOUSE_BUTTON_DOWN => {
                    if !self.mouse_handler.borrow().is_mouse_grabbed() {
                        self.mouse_handler.borrow_mut().set_mouse_grabbed(true);
                        SDLMouse::SDL_SetWindowRelativeMouseMode(window.handle(), true);
                    }
                }
                SDLEvents::SDL_EVENT_WINDOW_FOCUS_LOST => {
                    self.mouse_handler.borrow_mut().set_mouse_grabbed(false);
                    SDLMouse::SDL_SetWindowRelativeMouseMode(window.handle(), false);
                }
                _ => {}
            }
        }
    }

    /// `renderFrame`'s render: a frame of the level, presented to the window if there's one.
    fn render_frame(&self) {
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
            surface.present();
        }
    }

    pub fn set_connect_screen(&self, screen: ConnectScreen) {
        if matches!(*self.screen.borrow(), Screen::None) {
            *self.screen.borrow_mut() = Screen::Connect(screen);
        }
    }

    pub fn show_level_loading(&self) {
        *self.screen.borrow_mut() = Screen::LevelLoading;
    }

    /// Whether the level's loading screen is up, which closes once the level is ready.
    pub fn is_level_loading(&self) -> bool {
        matches!(*self.screen.borrow(), Screen::LevelLoading)
            && self
                .get_connection()
                .is_some_and(|connection| !connection.borrow().has_client_loaded())
    }

    /// The disconnected screen, with its title and the reason. A client with no screens to go on
    /// to stops there.
    pub fn show_disconnected(&self, title: &str, details: &DisconnectionDetails) {
        eprintln!("{title}: {}", details.reason());
        *self.screen.borrow_mut() = Screen::Disconnected {
            title: title.to_string(),
            details: details.clone(),
        };
        self.stop();
    }

    /// Why the game was disconnected, if it was.
    pub fn disconnection(&self) -> Option<(String, DisconnectionDetails)> {
        match &*self.screen.borrow() {
            Screen::Disconnected { title, details } => Some((title.clone(), details.clone())),
            _ => None,
        }
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
        *self.screen.borrow_mut() = Screen::None;
    }
}
