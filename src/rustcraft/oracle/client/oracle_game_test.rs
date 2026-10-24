//! oracle/client/src/main/java/rustcraft/oracle/client/OracleGameTest.java, on a server
//! (`runMultiplayer`): joins, puts the world in the scenario's state with the same commands,
//! waits like the oracle client does, sends the scenario-start marker and replays the scenario's
//! ticks, holding each step's keys and setting its look before each tick, and writing
//! `<out>/<scenario>/trace.jsonl` after it, then disconnects. `tools/port_client.py capture` runs
//! it behind `packet-proxy`.
//!
//! Java's test drives the game from its own thread while Fabric parks the client before a tick;
//! the port's is called there (`TestDriver::before_tick`), where it checks what Java's `waitFor`s
//! check and does what Java does before a tick (commands, the marker, holding the keys, the look),
//! going through every stage whose condition already holds like Java's consecutive `waitFor`s, and
//! as each tick ends (`after_tick`), where the trace line is written, like Java's (`TickTrace`). A
//! step's `wait_for` is waited on like Java waits: after the step's last tick and its line, the
//! network runs without the game ticking until the packets have come, then the client handles
//! what's queued, before the next tick. A paced scenario's ticks each wait for their slot first
//! (`ServerTicks`). Between ticks, the steps that come there run in order as each tick ends: a
//! screenshot renders a frame at a partial tick of 1 and reads it back like
//! `OracleScreenshot.take`, into `<out>/<scenario>/screenshots/<name>.png`, when the client has a
//! renderer (the runner makes one for a scenario with screenshots); a key is pressed and released
//! through the keyboard handler, characters typed, the cursor moved and commands sent, as Fabric's
//! test input does them, each waiting for its `wait_for` right after it. Keys held while a screen
//! is up go to the screen, like Java's, so they hold nothing down. "Chunks rendered" is every chunk
//! the server sends being in the level with its light and, with a renderer, every visible section
//! compiled (`levelRenderer.hasRenderedAllSections()`), and the settling ends with the camera at
//! rest too.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::net::minecraft::client::input::character_event::CharacterEvent;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::keyboard_handler::KeyAction;
use crate::net::minecraft::client::minecraft::{Minecraft, TestDriver};
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::renderer::game_renderer::RenderOptions;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::protocol::common::custom::discarded_payload::DiscardedPayload;
use crate::net::minecraft::network::protocol::common::serverbound_custom_payload_packet::ServerboundCustomPayloadPacket;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::level::particle_status::ParticleStatus;
use crate::net::minecraft::world::level::lighting::light_event_listener::LightEventListener;
use crate::rustcraft::oracle::client::scenario::{self, Scenario};
use crate::rustcraft::oracle::client::server_ticks::ServerTicks;
use crate::rustcraft::oracle::client::trace_writer::{self, TraceWriter};

/// Where the run is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    WaitingForLevel,
    /// Every chunk within the render distance in the level, before the setup changes blocks.
    LoadingChunks,
    /// A paced scenario's `tick rate` sent: waiting for the client to have it (more
    /// `ticking_state`s than it had), then for the server's next `set_time` (more than it had
    /// once the rate came).
    Pacing {
        ticking_states: i32,
        set_times: Option<i32>,
    },
    /// Sending the setup commands, the next one's index: one that places blocks waits for the
    /// server to send them before the next goes.
    SendingSetup(usize),
    WaitingForPosition,
    WaitingForChunks,
    Settling,
    Replaying,
    /// Past the scenario's last tick: it leaves before the next.
    Leaving,
    Done,
}

/// How a run ended, for the exit status.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Running,
    Finished,
    Failed(String),
}

/// The driver of one scenario's run on a server.
pub struct OracleGameTest {
    scenario: Scenario,
    out_dir: PathBuf,
    stage: Stage,
    ticks_in_stage: i32,
    replayed_ticks: i32,
    trace: Option<TraceWriter>,
    outcome: Rc<Cell<Option<Outcome>>>,
    /// The counts of the packets the current step waits for, as they were when it began.
    waiting_since: HashMap<String, i32>,
    /// The setup's commands, and the block changes the client had had when the last one that
    /// places blocks went.
    setup: Vec<String>,
    block_updates_before: Option<i32>,
    /// A paced scenario's slots, once the server's ticks are known.
    server_ticks: Option<ServerTicks>,
}

impl OracleGameTest {
    pub const SETTLE_TICKS: i32 = 20;
    pub const LOAD_TIMEOUT_TICKS: i32 = 60 * 20;
    /// How long a step waits for its packets, like Java's `WAIT_FOR_TIMEOUT`.
    const WAIT_FOR_TIMEOUT: Duration = Duration::from_secs(30);

    pub fn scenario_start() -> Identifier {
        Identifier::from_namespace_and_path("rustcraft", "scenario_start")
    }

    /// Sets the options the oracle client sets (`applyOptions`) that the port has, and returns the
    /// driver, which writes into `<out_dir>/<scenario>/`, and where its outcome goes.
    pub fn new(
        minecraft: &Minecraft,
        scenario: Scenario,
        out_dir: &Path,
    ) -> (OracleGameTest, Rc<Cell<Option<Outcome>>>) {
        {
            let mut options = minecraft.options.borrow_mut();
            options.set_render_distance(scenario.options.render_distance);
            options.set_particles(ParticleStatus::MINIMAL);
            options.joined_first_server = true;
            options.fov = scenario.options.fov;
            // oracle/client/options.txt's, which differ from the defaults.
            options.chunk_section_fade_in_time = 0.0;
            options.cloud_range = 64;
            options.texture_filtering_rgss = true;
            options.vignette = false;
            options.enable_vsync = false;
            options.gui_scale = scenario.options.gui_scale;
        }
        minecraft.hud_hidden.set(scenario.options.hide_gui);
        let outcome = Rc::new(Cell::new(Some(Outcome::Running)));
        (
            OracleGameTest {
                out_dir: out_dir.join(&scenario.name),
                scenario,
                stage: Stage::WaitingForLevel,
                ticks_in_stage: 0,
                replayed_ticks: 0,
                trace: None,
                outcome: Rc::clone(&outcome),
                waiting_since: HashMap::new(),
                setup: Vec::new(),
                block_updates_before: None,
                server_ticks: None,
            },
            outcome,
        )
    }

    /// The options the oracle client renders with: its `options.txt` (`oracle/client/options.txt`:
    /// smooth lighting, cut-out leaves, no chunk fade, gamma 0.5, glint 0.75, RGSS texture
    /// filtering, cloud range 64) with `applyOptions`' (the scenario's field of view and render
    /// distance, no bobbing or clouds).
    pub fn render_options(scenario: &Scenario) -> RenderOptions {
        RenderOptions {
            render_distance: scenario.options.render_distance,
            fov: scenario.options.fov,
            fov_effect_scale: 1.0,
            cloud_range: 64,
            ambient_occlusion: true,
            cutout_leaves: true,
            chunk_section_fade_in_time: 0.0,
            gamma: 0.5,
            glint_strength: 0.75,
            use_rgss: true,
            hide_gui: scenario.options.hide_gui,
        }
    }

    /// The commands that put the world in the scenario's state, the same in both modes: the
    /// scenario's own build its course before the player is put in place.
    pub fn setup_commands(scenario: &Scenario, player: &str) -> Vec<String> {
        let mut commands = vec![
            "gamerule advance_time false".to_string(),
            "gamerule advance_weather false".to_string(),
            "gamerule random_tick_speed 0".to_string(),
            "gamerule spawn_mobs false".to_string(),
            "difficulty normal".to_string(),
            format!("time set {}", scenario.world.time),
            "weather clear".to_string(),
            format!("gamemode {} {player}", scenario.world.game_mode),
            "tp @e[type=!minecraft:player] 0 -1000 0".to_string(),
        ];
        commands.extend(scenario.setup.iter().cloned());
        commands.push(format!(
            "tp {player} {}",
            OracleGameTest::position(&scenario.player)
        ));
        commands
    }

    /// `String.format("%s %s %s %s %s", …)` of the doubles and floats, which prints them with
    /// `Double.toString` and `Float.toString`: shortest digits with a decimal point.
    fn position(player: &scenario::Player) -> String {
        format!(
            "{} {} {} {} {}",
            java_to_string(player.x),
            java_to_string(player.y),
            java_to_string(player.z),
            java_to_string(f64::from(player.y_rot)),
            java_to_string(f64::from(player.x_rot)),
        )
    }

    fn next(&mut self, stage: Stage) {
        self.stage = stage;
        self.ticks_in_stage = 0;
    }

    fn fail(&mut self, minecraft: &Minecraft, reason: String) {
        eprintln!("scenario {}: {reason}", self.scenario.name);
        self.outcome.set(Some(Outcome::Failed(reason)));
        self.stage = Stage::Done;
        minecraft.stop();
    }

    fn is_at_scenario_position(&self, minecraft: &Minecraft) -> bool {
        minecraft.player.borrow().as_ref().is_some_and(|player| {
            (player.get_x() - self.scenario.player.x).abs() < 1.0E-6
                && (player.get_z() - self.scenario.player.z).abs() < 1.0E-6
        })
    }

    /// The level as the screenshots need it, like the oracle client's `areChunksRendered`: every
    /// chunk within the render distance in the level, and every chunk the server sends a player at
    /// that distance (`ChunkTrackingView.isWithinDistance`, with the neighbours it includes, which
    /// the sections at the render distance's edge need to compile), the level's light caught up
    /// (nothing queued, which `poll_light_updates` takes a tenth of a frame, and no work left for
    /// its engine), and, with a renderer, every visible section compiled (`hasRenderedAllSections`
    /// only counts the queued ones, not those still waiting for their neighbours).
    fn are_chunks_rendered(&self, minecraft: &Minecraft) -> bool {
        self.are_chunks_loaded(minecraft)
            && self.are_tracked_chunks_loaded(minecraft)
            && minecraft.level.borrow().as_ref().is_some_and(|level| {
                !level.has_light_updates()
                    && !level.get_chunk_source().get_light_engine().has_light_work()
            })
            && minecraft
                .game_renderer
                .borrow()
                .as_ref()
                .is_none_or(|game_renderer| {
                    game_renderer.level_renderer().has_rendered_all_sections()
                })
    }

    /// The camera at rest, like the oracle client's `isCameraAtRest`: its eye height and field of
    /// view ease toward the player's a tick at a time from 0 (`Camera.tick` halves what's left),
    /// and stop some 25 ticks after the level came. A client that renders slowly ticks less while
    /// the chunks come, and took the spawn screenshot with the field of view a hair short of the
    /// player's (TESTING.md, "Screenshots"). Without a renderer there's no camera.
    fn is_camera_at_rest(&self, minecraft: &Minecraft) -> bool {
        minecraft
            .game_renderer
            .borrow()
            .as_ref()
            .is_none_or(|game_renderer| game_renderer.main_camera().is_at_rest())
    }

    /// `ChunkTrackingView.isWithinDistance`'s chunks around the player, with their neighbours.
    fn are_tracked_chunks_loaded(&self, minecraft: &Minecraft) -> bool {
        let level = minecraft.level.borrow();
        let player = minecraft.player.borrow();
        let (Some(level), Some(player)) = (level.as_ref(), player.as_ref()) else {
            return false;
        };

        let center = player.block_position();
        let (center_x, center_z) = (
            SectionPos::block_to_section_coord(center.get_x()),
            SectionPos::block_to_section_coord(center.get_z()),
        );
        let distance = minecraft.options.borrow().get_effective_render_distance();
        (-distance - 2..=distance + 2).all(|dz| {
            (-distance - 2..=distance + 2).all(|dx| {
                let delta_x = i64::from((dx.abs() - 2).max(0));
                let delta_z = i64::from((dz.abs() - 2).max(0));
                let tracked = delta_x * delta_x + delta_z * delta_z
                    < i64::from(distance) * i64::from(distance);
                !tracked
                    || level
                        .get_chunk_source()
                        .get_chunk(center_x + dx, center_z + dz)
                        .is_some()
            })
        })
    }

    /// Every chunk within the render distance around the player is in the level.
    fn are_chunks_loaded(&self, minecraft: &Minecraft) -> bool {
        let level = minecraft.level.borrow();
        let player = minecraft.player.borrow();
        let (Some(level), Some(player)) = (level.as_ref(), player.as_ref()) else {
            return false;
        };

        let center = player.block_position();
        let (center_x, center_z) = (
            SectionPos::block_to_section_coord(center.get_x()),
            SectionPos::block_to_section_coord(center.get_z()),
        );
        let distance = minecraft.options.borrow().get_effective_render_distance();
        (-distance..=distance).all(|dz| {
            (-distance..=distance).all(|dx| {
                level
                    .get_chunk_source()
                    .get_chunk(center_x + dx, center_z + dz)
                    .is_some()
            })
        })
    }

    /// Whether the scenario takes screenshots, which needs a renderer.
    pub fn has_screenshots(scenario: &Scenario) -> bool {
        scenario.steps.iter().any(|step| step.screenshot.is_some())
    }

    /// The steps that come after `ticks` of the scenario's ticks and before the next: screenshots
    /// and input.
    fn steps_after(&self, ticks: i32) -> Vec<scenario::Step> {
        let mut elapsed = 0;
        let mut steps = Vec::new();
        for step in &self.scenario.steps {
            if step.screenshot.is_some() || step.is_input() {
                if elapsed == ticks {
                    steps.push(step.clone());
                }
            } else {
                elapsed += step.ticks;
            }
        }
        steps
    }

    /// The steps due after `ticks` ticks, in order: screenshots (`OracleScreenshot.take`, a frame
    /// at a partial tick of 1, read back), and input as Fabric's test input gives it, each waiting
    /// for its answer right after it.
    fn run_steps_between_ticks(&mut self, minecraft: &Minecraft, ticks: i32) {
        for step in self.steps_after(ticks) {
            let answer_since: HashMap<String, i32> = step
                .wait_for
                .iter()
                .map(|packet| {
                    (
                        packet.clone(),
                        minecraft.packet_processor().arrivals(packet),
                    )
                })
                .collect();
            if let Some(key) = &step.key {
                let scancode = Scenario::PRESSABLE_KEYS
                    .iter()
                    .find(|(name, _)| name == key)
                    .map(|(_, scancode)| *scancode)
                    .expect("a key the scenario checked");
                let event = KeyEvent::of_scancode(scancode);
                minecraft.key_press(&event, KeyAction::PRESS);
                minecraft.key_press(&event, KeyAction::RELEASE);
            }
            if let Some(text) = &step.r#type {
                // String.chars(): UTF-16 units.
                for unit in text.encode_utf16() {
                    minecraft.char_typed(&CharacterEvent::new(i32::from(unit)));
                }
            }
            if let Some([x, y]) = step.cursor {
                minecraft.mouse_moved_to(x, y);
            }
            if let Some(command) = &step.command
                && let Some(connection) = minecraft.get_connection()
            {
                connection.borrow().send_command(command);
            }
            if step.is_input()
                && !answer_since.is_empty()
                && !OracleGameTest::wait_for_answer(minecraft, &answer_since)
            {
                self.fail(
                    minecraft,
                    format!("timed out waiting for {:?}", step.wait_for),
                );
                return;
            }
            if let Some(name) = &step.screenshot {
                self.take_screenshot(minecraft, name);
                if self.stage == Stage::Done {
                    return;
                }
            }
        }
    }

    fn take_screenshot(&mut self, minecraft: &Minecraft, name: &str) {
        if !minecraft.render_level(1.0) {
            return;
        }
        let image = minecraft
            .game_renderer
            .borrow()
            .as_ref()
            .map(|game_renderer| game_renderer.take_screenshot());
        let Some(image) = image else {
            return;
        };
        let path = self.out_dir.join("screenshots").join(format!("{name}.png"));
        let written = std::fs::create_dir_all(path.parent().expect("a directory"))
            .map_err(|error| error.to_string())
            .and_then(|()| {
                image
                    .write_to_file(&path)
                    .map_err(|error| error.to_string())
            });
        if let Err(error) = written {
            self.fail(minecraft, format!("{}: {error}", path.display()));
        }
    }

    /// Waits, the game not ticking, until each packet has been queued more often than `since`
    /// says, then handles what's queued. Whether they came in time.
    fn wait_for_answer(minecraft: &Minecraft, since: &HashMap<String, i32>) -> bool {
        let deadline = Instant::now() + OracleGameTest::WAIT_FOR_TIMEOUT;
        let arrived = || {
            since
                .iter()
                .all(|(packet, before)| minecraft.packet_processor().arrivals(packet) > *before)
        };
        while !arrived() {
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        minecraft.packet_processor().process_queued_packets();
        true
    }

    fn places_blocks(command: &str) -> bool {
        command.starts_with("fill ") || command.starts_with("setblock ")
    }

    /// The block changes the server has sent so far, one block's or a section's.
    fn block_updates(minecraft: &Minecraft) -> i32 {
        minecraft
            .packet_processor()
            .arrivals("minecraft:block_update")
            + minecraft
                .packet_processor()
                .arrivals("minecraft:section_blocks_update")
    }

    fn scenario_ticks(&self) -> i32 {
        self.scenario.steps.iter().map(|step| step.ticks).sum()
    }

    /// The step the scenario's tick is in, with the step's first tick.
    fn step_of(&self, tick: i32) -> (&scenario::Step, i32) {
        let mut first = 0;
        for step in &self.scenario.steps {
            if tick < first + step.ticks {
                return (step, first);
            }
            first += step.ticks;
        }
        panic!("the scenario has {first} ticks, not {}", tick + 1)
    }

    /// What `replay` does before the scenario's tick: holds its step's keys (`holdOnly`) and sets
    /// its look (`TestInput.lookAt`), and as a step begins, counts the packets it waits for.
    fn hold_inputs(&mut self, minecraft: &Minecraft, tick: i32) {
        let (step, first) = self.step_of(tick);
        if tick == first {
            self.waiting_since = step
                .wait_for
                .iter()
                .map(|packet| {
                    (
                        packet.clone(),
                        minecraft.packet_processor().arrivals(packet),
                    )
                })
                .collect();
        }
        let (step, _) = self.step_of(tick);
        let screen_shown = minecraft.screen().is_some();
        OracleGameTest::hold_only(
            &mut minecraft.options.borrow_mut(),
            &step.keys,
            screen_shown,
        );
        if let Some([y_rot, x_rot]) = step.look
            && let Some(player) = minecraft.player.borrow_mut().as_mut()
        {
            player.set_y_rot(y_rot);
            player.set_x_rot(x_rot);
        }
    }

    /// `holdOnly`: the keys named held, the others released. A key pressed while a screen is up
    /// goes to the screen, which holds nothing down.
    pub fn hold_only(options: &mut Options, keys: &[String], screen_shown: bool) {
        let held = |name: &str| !screen_shown && keys.iter().any(|key| key == name);
        options.key_up.set_down(held("forward"));
        options.key_down.set_down(held("back"));
        options.key_left.set_down(held("left"));
        options.key_right.set_down(held("right"));
        options.key_jump.set_down(held("jump"));
        options.key_shift.set_down(held("sneak"));
        options.key_sprint.set_down(held("sprint"));
    }

    /// Sends the setup's commands from `next` on, until one that places blocks, and whether it sent
    /// them all: the server sends a command's blocks at the end of the tick it ran in, so the next
    /// waits for them, and each command's blocks come in a tick of their own, grouped the same way
    /// every run, like Java's.
    fn send_setup(&mut self, minecraft: &Minecraft, next: usize) -> bool {
        if let Some(before) = self.block_updates_before {
            if OracleGameTest::block_updates(minecraft) <= before {
                return false;
            }
            self.block_updates_before = None;
        }

        let connection = minecraft
            .get_connection()
            .expect("the player is in a level");
        for (index, command) in self.setup.iter().enumerate().skip(next) {
            let before = OracleGameTest::block_updates(minecraft);
            connection.borrow().send_command(command);
            if OracleGameTest::places_blocks(command) {
                self.block_updates_before = Some(before);
                self.next(Stage::SendingSetup(index + 1));
                return false;
            }
        }
        self.next(Stage::WaitingForPosition);
        true
    }

    /// `ServerTicks.pace`, as far as it goes before the tick: sends `tick rate`, then waits for
    /// the client to have the rate and for the server's next `set_time`, and from it finds the
    /// server's ticks. Whether they're known.
    fn pace(&mut self, minecraft: &Minecraft, ticking_states: i32, set_times: Option<i32>) -> bool {
        let processor = minecraft.packet_processor();
        let set_times = match set_times {
            Some(set_times) => set_times,
            None if processor.arrivals("minecraft:ticking_state") > ticking_states => {
                // The server synchronizes the time every 20 ticks, as the tick ends
                // (`MinecraftServer.tickChildren`), and no command has made it send one out of turn
                // since the rate changed.
                let set_times = processor.arrivals("minecraft:set_time");
                self.stage = Stage::Pacing {
                    ticking_states,
                    set_times: Some(set_times),
                };
                set_times
            }
            None => return false,
        };
        if processor.arrivals("minecraft:set_time") <= set_times {
            return false;
        }
        let reference = processor
            .last_arrival("minecraft:set_time")
            .expect("a set_time came");
        self.server_ticks = Some(ServerTicks::new(self.scenario.world.tick_rate, reference));
        true
    }

    /// `tick rate <rate>`, the rate as Java's `Float.toString` prints it.
    pub fn tick_rate_command(tick_rate: f32) -> String {
        format!("tick rate {}", java_to_string(f64::from(tick_rate)))
    }

    /// After the last tick of a step that waits for packets and its trace line: runs the network,
    /// the game not ticking, until each has come since the step began, then handles what the game
    /// thread has queued. Whether they came in time.
    fn wait_for_packets(&self, minecraft: &Minecraft, tick: i32) -> bool {
        let (step, first) = self.step_of(tick);
        if step.wait_for.is_empty() || tick != first + step.ticks - 1 {
            return true;
        }

        OracleGameTest::wait_for_answer(minecraft, &self.waiting_since)
    }

    /// The player's trace line after the scenario's tick, with the level's other entities.
    fn write_trace(&mut self, minecraft: &Minecraft, tick: i32) -> std::io::Result<()> {
        let level = minecraft.level.borrow();
        let line = trace_writer::line(
            tick,
            minecraft
                .player
                .borrow()
                .as_ref()
                .expect("the player is in a level"),
            level
                .as_ref()
                .into_iter()
                .flat_map(|level| level.entities_for_rendering()),
        );
        drop(level);
        match self.trace.as_mut() {
            Some(trace) => trace.write(&line),
            None => Ok(()),
        }
    }
}

impl TestDriver for OracleGameTest {
    fn before_tick(&mut self, minecraft: &Rc<Minecraft>) {
        if let Some(server_ticks) = self.server_ticks.as_mut() {
            server_ticks.await_slot(minecraft);
        }

        // Each stage's condition is checked here, before the tick, like Java's `waitFor`s, and a
        // stage whose condition already holds goes straight on to the next.
        loop {
            match self.stage {
                Stage::WaitingForLevel => {
                    let loaded = minecraft.level.borrow().is_some()
                        && minecraft.player.borrow().is_some()
                        && !minecraft.is_level_loading();
                    if !loaded {
                        return;
                    }
                    self.next(Stage::LoadingChunks);
                }
                Stage::LoadingChunks => {
                    // Every chunk the client shows is in before the setup changes blocks: the
                    // server sends a chunk's light only while the chunk is at the edge of what the
                    // client has (`ChunkHolder.broadcastChanges`), and a chunk still to come would
                    // come with the setup's blocks and light (TESTING.md).
                    if !self.are_chunks_loaded(minecraft) {
                        return;
                    }
                    // The world itself (seed, flat preset) comes from the server's properties;
                    // commands set the rest.
                    self.setup = OracleGameTest::setup_commands(&self.scenario, "@s");
                    if self.scenario.world.tick_rate > 0.0 {
                        let ticking_states = minecraft
                            .packet_processor()
                            .arrivals("minecraft:ticking_state");
                        let command =
                            OracleGameTest::tick_rate_command(self.scenario.world.tick_rate);
                        let connection = minecraft
                            .get_connection()
                            .expect("the player is in a level");
                        connection.borrow().send_command(&command);
                        self.next(Stage::Pacing {
                            ticking_states,
                            set_times: None,
                        });
                        return;
                    }
                    self.next(Stage::SendingSetup(0));
                }
                Stage::Pacing {
                    ticking_states,
                    set_times,
                } => {
                    if !self.pace(minecraft, ticking_states, set_times) {
                        return;
                    }
                    // From here on every tick takes its slot, this one too.
                    if let Some(server_ticks) = self.server_ticks.as_mut() {
                        server_ticks.await_slot(minecraft);
                    }
                    self.next(Stage::SendingSetup(0));
                }
                Stage::SendingSetup(next) => {
                    if !self.send_setup(minecraft, next) {
                        return;
                    }
                }
                Stage::WaitingForPosition => {
                    if !self.is_at_scenario_position(minecraft) {
                        return;
                    }
                    self.next(Stage::WaitingForChunks);
                }
                Stage::WaitingForChunks => {
                    if !self.are_chunks_rendered(minecraft) {
                        return;
                    }
                    self.next(Stage::Settling);
                }
                Stage::Settling => {
                    // `settle`: the ticks, then until the player is on the ground and, with a
                    // renderer, the camera at rest.
                    let on_ground = minecraft
                        .player
                        .borrow()
                        .as_ref()
                        .is_some_and(|player| player.on_ground());
                    if self.ticks_in_stage < OracleGameTest::SETTLE_TICKS
                        || !on_ground
                        || !self.is_camera_at_rest(minecraft)
                    {
                        return;
                    }
                    // `settle`'s end: the setup's chat feedback cleared, so it doesn't show up in
                    // screenshots (the toasts aren't ported).
                    minecraft.gui.borrow_mut().hud.chat.clear_messages(false);
                    let path = self.out_dir.join("trace.jsonl");
                    match TraceWriter::new(&path) {
                        Ok(trace) => self.trace = Some(trace),
                        Err(error) => {
                            self.fail(minecraft, format!("{}: {error}", path.display()));
                            return;
                        }
                    }
                    // The texture animations from their first frames, like the oracle client's
                    // `restartTextureAnimations`: how many ticks the join took varies.
                    if let Some(game_renderer) = minecraft.game_renderer.borrow_mut().as_mut() {
                        game_renderer.restart_texture_animations();
                    }
                    // Marks scenario tick 0 in packet captures: the join before it depends on the
                    // server's real-time ticking, everything after must match.
                    let connection = minecraft
                        .get_connection()
                        .expect("the player is in a level");
                    connection
                        .borrow()
                        .send(ServerboundCustomPayloadPacket::new(Arc::new(
                            DiscardedPayload::new(OracleGameTest::scenario_start()),
                        )));
                    if let Some(server_ticks) = self.server_ticks.as_mut() {
                        server_ticks.start_scenario();
                    }
                    self.next(Stage::Replaying);
                    self.run_steps_between_ticks(minecraft, 0);
                    if self.stage == Stage::Done {
                        return;
                    }
                    if self.scenario_ticks() == 0 {
                        self.next(Stage::Leaving);
                    }
                }
                Stage::Replaying => {
                    self.hold_inputs(minecraft, self.replayed_ticks);
                    return;
                }
                Stage::Leaving => {
                    OracleGameTest::hold_only(&mut minecraft.options.borrow_mut(), &[], false);
                    if let Some(Err(error)) = self.trace.take().map(TraceWriter::close) {
                        self.fail(minecraft, format!("writing the trace: {error}"));
                        return;
                    }
                    let connection = minecraft
                        .get_connection()
                        .expect("the player is in a level");
                    let connection = connection.borrow().get_connection();
                    connection.disconnect(DisconnectionDetails::new(Component::literal(
                        "Scenario finished",
                    )));
                    // The game stops at the disconnected screen, which is where this run ends.
                    self.outcome.set(Some(Outcome::Finished));
                    self.next(Stage::Done);
                    minecraft.disconnect(
                        &DisconnectionDetails::new(Component::literal("Scenario finished")),
                        false,
                    );
                    let missed = self.server_ticks.as_ref().map_or(0, ServerTicks::missed);
                    if missed > 0 {
                        let reason = format!(
                            "the client {} {missed} times in the scenario",
                            ServerTicks::FELL_BEHIND
                        );
                        self.fail(minecraft, reason);
                    }
                    return;
                }
                Stage::Done => return,
            }
        }
    }

    fn after_tick(&mut self, minecraft: &Rc<Minecraft>) {
        if let Some(server_ticks) = self.server_ticks.as_mut() {
            server_ticks.tick_ended();
        }
        self.ticks_in_stage += 1;
        let waiting = matches!(
            self.stage,
            Stage::WaitingForLevel
                | Stage::LoadingChunks
                | Stage::Pacing { .. }
                | Stage::SendingSetup(_)
                | Stage::WaitingForPosition
                | Stage::WaitingForChunks
        );
        if waiting && self.ticks_in_stage > OracleGameTest::LOAD_TIMEOUT_TICKS {
            let stage = self.stage;
            self.fail(minecraft, format!("timed out at {stage:?}"));
            return;
        }
        if self.stage != Stage::Done
            && let Some((title, details)) = minecraft.disconnection()
        {
            self.fail(
                minecraft,
                format!(
                    "disconnected at {:?}: {title}: {}",
                    self.stage,
                    details.reason_text()
                ),
            );
            return;
        }

        match self.stage {
            Stage::Settling => {
                if self.ticks_in_stage
                    > OracleGameTest::SETTLE_TICKS + OracleGameTest::LOAD_TIMEOUT_TICKS
                {
                    self.fail(
                        minecraft,
                        "timed out waiting for the player to land".to_string(),
                    );
                }
            }
            Stage::Replaying => {
                let tick = self.replayed_ticks;
                if let Err(error) = self.write_trace(minecraft, tick) {
                    self.fail(minecraft, format!("writing the trace: {error}"));
                    return;
                }
                if !self.wait_for_packets(minecraft, tick) {
                    let (step, _) = self.step_of(tick);
                    let reason = format!("timed out waiting for {:?}", step.wait_for);
                    self.fail(minecraft, reason);
                    return;
                }
                self.replayed_ticks += 1;
                self.run_steps_between_ticks(minecraft, self.replayed_ticks);
                if self.stage == Stage::Done {
                    return;
                }
                if self.replayed_ticks == self.scenario_ticks() {
                    self.next(Stage::Leaving);
                }
            }
            _ => {}
        }
    }
}

/// Java's `Double.toString` for the values a scenario has: shortest digits, with `.0` for a
/// whole number. Outside 10^-3 to 10^7 Java switches to an exponent, which isn't ported.
fn java_to_string(value: f64) -> String {
    assert!(
        value == 0.0 || (1.0E-3..1.0E7).contains(&value.abs()),
        "Double.toString of {value} isn't ported"
    );
    format!("{value:?}")
}

#[cfg(test)]
mod tests {
    use super::OracleGameTest;
    use crate::rustcraft::oracle::client::scenario::Scenario;

    #[test]
    fn sends_the_oracle_clients_commands() {
        let scenario = Scenario::load(std::path::Path::new("oracle/scenarios/walk.json")).unwrap();
        // vanilla's walk capture.
        assert_eq!(
            OracleGameTest::setup_commands(&scenario, "@s"),
            [
                "gamerule advance_time false",
                "gamerule advance_weather false",
                "gamerule random_tick_speed 0",
                "gamerule spawn_mobs false",
                "difficulty normal",
                "time set 6000",
                "weather clear",
                "gamemode survival @s",
                "tp @e[type=!minecraft:player] 0 -1000 0",
                "tp @s 0.5 -60.0 0.5 0.0 0.0",
            ]
        );
    }

    #[test]
    fn sends_vanilla_s_tick_rate() {
        let scenario = Scenario::load(std::path::Path::new("oracle/scenarios/water.json")).unwrap();
        // vanilla's water capture.
        assert_eq!(
            OracleGameTest::tick_rate_command(scenario.world.tick_rate),
            "tick rate 2.0"
        );
    }
}
