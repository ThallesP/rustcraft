//! net/minecraft/client/Minecraft.java
//!
//! Not ported yet: nearly all of it: the window and rendering (the render track's), sound, the
//! integrated server, resources, the GUI and its screens, input, telemetry and the profiler. What's
//! here is the game loop's timing and what a tick does with a connection to a server: run the
//! packets queued for the game thread, then up to 10 ticks, each ticking the connection (the game
//! mode's, or the connect screen's before there's a level), the player on its level, what the
//! player sends, and ending with `client_tick_end`; then the frame's update of the level's light
//! (`renderFrame`'s `level.update()`).
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

use crate::net::minecraft::client::delta_tracker::Timer;
use crate::net::minecraft::client::gui::screens::connect_screen::ConnectScreen;
use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
use crate::net::minecraft::client::multiplayer::multi_player_game_mode::MultiPlayerGameMode;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::client::user::User;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_processor::PacketProcessor;
use crate::net::minecraft::network::protocol::game::serverbound_client_tick_end_packet::ServerboundClientTickEndPacket;
use crate::net::minecraft::server::network::event_loop_group_holder::EventLoopGroupHolder;
use crate::net::minecraft::util::util::Util;

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
}

impl Minecraft {
    /// Vanilla's default `framerateLimit`: frames are at most this often, and each processes the
    /// packets queued for the game thread.
    const FRAMERATE_LIMIT: u64 = 120;

    pub fn new(user: User, options: Options) -> Rc<Minecraft> {
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

        self.tick_screen();

        if self.level.borrow().is_some() {
            if !self.pause.get() {
                // `level.tickEntities()`: the player is the only entity the level has
                // (`ClientLevel.tickNonPassenger`).
                {
                    let level = self.level.borrow();
                    let mut player = self.player.borrow_mut();
                    if let (Some(level), Some(player)) = (level.as_ref(), player.as_mut()) {
                        player.common_tick();
                        player.tick(level, &self.options.borrow());
                    }
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
