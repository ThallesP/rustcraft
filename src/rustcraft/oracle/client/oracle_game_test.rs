//! oracle/client/src/main/java/rustcraft/oracle/client/OracleGameTest.java, on a server
//! (`runMultiplayer`): joins, puts the world in the scenario's state with the same commands,
//! waits like the oracle client does, sends the scenario-start marker and replays the scenario's
//! ticks, holding each step's keys and setting its look before each tick, and writing
//! `<out>/<scenario>/trace.jsonl` after it, then disconnects. `tools/port_client.py capture` runs
//! it behind `packet-proxy`.
//!
//! Java's test drives the game from its own thread, between ticks; the port's is called after
//! each tick (`TestDriver`), so what Java does before a tick (holding the keys, the look) is done
//! after the one before it, and the trace line is written as the tick ends, like Java's
//! (`TickTrace`). A step's `wait_for` is waited on like Java waits: after the step's last tick and
//! its line, the network runs without the game ticking until the packets have come, then the client
//! handles what's queued, before the next tick. A screenshot step renders a frame at a partial tick
//! of 1 and reads it back like `OracleScreenshot.take`, into
//! `<out>/<scenario>/screenshots/<name>.png`, when the client has a renderer (the runner makes one
//! for a scenario with screenshots). The port has no screens, so waiting for the level's loading
//! screen to close is waiting for the level to be ready, which closes it in vanilla too; "chunks
//! rendered" is every chunk in the render distance being in the level and, with a renderer, every
//! visible section compiled (`levelRenderer.hasRenderedAllSections()`).

use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::net::minecraft::client::minecraft::{Minecraft, TestDriver};
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::renderer::game_renderer::RenderOptions;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::protocol::common::custom::discarded_payload::DiscardedPayload;
use crate::net::minecraft::network::protocol::common::serverbound_custom_payload_packet::ServerboundCustomPayloadPacket;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::level::particle_status::ParticleStatus;
use crate::net::minecraft::server::network::event_loop_group_holder::EventLoopGroupHolder;
use crate::rustcraft::oracle::client::scenario::{self, Scenario};
use crate::rustcraft::oracle::client::trace_writer::{self, TraceWriter};

/// Where the run is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    WaitingForLevel,
    WaitingForPosition,
    WaitingForChunks,
    Settling,
    Replaying,
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

    /// Every chunk within the render distance around the player is in the level. Java then waits
    /// for the renderer to have compiled their sections, which comes with the render track.
    fn are_chunks_rendered(&self, minecraft: &Minecraft) -> bool {
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
        }) && minecraft
            .game_renderer
            .borrow()
            .as_ref()
            .is_none_or(|game_renderer| game_renderer.level_renderer().has_rendered_all_sections())
    }

    /// Whether the scenario takes screenshots, which needs a renderer.
    pub fn has_screenshots(scenario: &Scenario) -> bool {
        scenario.steps.iter().any(|step| step.screenshot.is_some())
    }

    /// The screenshot steps that come after `ticks` of the scenario's ticks and before the next.
    fn screenshots_after(&self, ticks: i32) -> Vec<String> {
        let mut elapsed = 0;
        let mut names = Vec::new();
        for step in &self.scenario.steps {
            match &step.screenshot {
                Some(name) if elapsed == ticks => names.push(name.clone()),
                _ => elapsed += step.ticks,
            }
        }
        names
    }

    /// `OracleScreenshot.take` for each screenshot due after `ticks` ticks: a frame at a partial
    /// tick of 1, read back.
    fn take_screenshots(&mut self, minecraft: &Minecraft, ticks: i32) {
        for name in self.screenshots_after(ticks) {
            if !minecraft.render_level(1.0) {
                continue;
            }
            let image = minecraft
                .game_renderer
                .borrow()
                .as_ref()
                .map(|game_renderer| game_renderer.take_screenshot());
            let Some(image) = image else {
                continue;
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
                return;
            }
        }
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
    fn before_tick(&mut self, minecraft: &Minecraft, tick: i32) {
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
        OracleGameTest::hold_only(&mut minecraft.options.borrow_mut(), &step.keys);
        if let Some([y_rot, x_rot]) = step.look
            && let Some(player) = minecraft.player.borrow_mut().as_mut()
        {
            player.set_y_rot(y_rot);
            player.set_x_rot(x_rot);
        }
    }

    /// `holdOnly`: the keys named held, the others released.
    pub fn hold_only(options: &mut Options, keys: &[String]) {
        let held = |name: &str| keys.iter().any(|key| key == name);
        options.key_up.set_down(held("forward"));
        options.key_down.set_down(held("back"));
        options.key_left.set_down(held("left"));
        options.key_right.set_down(held("right"));
        options.key_jump.set_down(held("jump"));
        options.key_shift.set_down(held("sneak"));
        options.key_sprint.set_down(held("sprint"));
    }

    /// After the last tick of a step that waits for packets and its trace line: runs the network,
    /// the game not ticking, until each has come since the step began, then handles what the game
    /// thread has queued. Whether they came in time.
    fn wait_for_packets(&self, minecraft: &Minecraft, tick: i32) -> bool {
        let (step, first) = self.step_of(tick);
        if step.wait_for.is_empty() || tick != first + step.ticks - 1 {
            return true;
        }

        let deadline = Instant::now() + OracleGameTest::WAIT_FOR_TIMEOUT;
        let arrived = || {
            self.waiting_since
                .iter()
                .all(|(packet, before)| minecraft.packet_processor().arrivals(packet) > *before)
        };
        while !arrived() {
            if Instant::now() >= deadline {
                return false;
            }
            EventLoopGroupHolder::remote().run_until(Instant::now() + Duration::from_millis(5));
        }
        minecraft.packet_processor().process_queued_packets();
        true
    }

    /// The player's trace line after the scenario's tick.
    fn write_trace(&mut self, minecraft: &Minecraft, tick: i32) -> std::io::Result<()> {
        let line = trace_writer::line(
            tick,
            minecraft
                .player
                .borrow()
                .as_ref()
                .expect("the player is in a level"),
        );
        match self.trace.as_mut() {
            Some(trace) => trace.write(&line),
            None => Ok(()),
        }
    }
}

impl TestDriver for OracleGameTest {
    fn after_tick(&mut self, minecraft: &Rc<Minecraft>) {
        self.ticks_in_stage += 1;
        let waiting = matches!(
            self.stage,
            Stage::WaitingForLevel | Stage::WaitingForPosition | Stage::WaitingForChunks
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
                    details.reason()
                ),
            );
            return;
        }

        match self.stage {
            Stage::WaitingForLevel => {
                let loaded = minecraft.level.borrow().is_some()
                    && minecraft.player.borrow().is_some()
                    && !minecraft.is_level_loading();
                if loaded {
                    // The world itself (seed, flat preset) comes from the server's properties;
                    // commands set the rest.
                    let connection = minecraft
                        .get_connection()
                        .expect("the player is in a level");
                    for command in OracleGameTest::setup_commands(&self.scenario, "@s") {
                        connection.borrow().send_command(&command);
                    }
                    self.next(Stage::WaitingForPosition);
                }
            }
            Stage::WaitingForPosition => {
                if self.is_at_scenario_position(minecraft) {
                    self.next(Stage::WaitingForChunks);
                }
            }
            Stage::WaitingForChunks => {
                if self.are_chunks_rendered(minecraft) {
                    self.next(Stage::Settling);
                }
            }
            Stage::Settling => {
                // `settle`: the ticks, then until the player is on the ground.
                let on_ground = minecraft
                    .player
                    .borrow()
                    .as_ref()
                    .is_some_and(|player| player.on_ground());
                if self.ticks_in_stage
                    > OracleGameTest::SETTLE_TICKS + OracleGameTest::LOAD_TIMEOUT_TICKS
                {
                    self.fail(
                        minecraft,
                        "timed out waiting for the player to land".to_string(),
                    );
                    return;
                }
                if self.ticks_in_stage >= OracleGameTest::SETTLE_TICKS && on_ground {
                    let path = self.out_dir.join("trace.jsonl");
                    match TraceWriter::new(&path) {
                        Ok(trace) => self.trace = Some(trace),
                        Err(error) => {
                            self.fail(minecraft, format!("{}: {error}", path.display()));
                            return;
                        }
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
                    self.next(Stage::Replaying);
                    self.take_screenshots(minecraft, 0);
                    if self.stage == Stage::Done {
                        return;
                    }
                    if self.scenario_ticks() > 0 {
                        self.before_tick(minecraft, 0);
                    }
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
                self.take_screenshots(minecraft, self.replayed_ticks);
                if self.stage == Stage::Done {
                    return;
                }
                if self.replayed_ticks < self.scenario_ticks() {
                    self.before_tick(minecraft, self.replayed_ticks);
                    return;
                }

                OracleGameTest::hold_only(&mut minecraft.options.borrow_mut(), &[]);
                if let Some(Err(error)) = self.trace.take().map(TraceWriter::close) {
                    self.fail(minecraft, format!("writing the trace: {error}"));
                    return;
                }
                let connection = minecraft
                    .get_connection()
                    .expect("the player is in a level");
                let connection = connection.borrow().get_connection();
                connection.disconnect(DisconnectionDetails::new("Scenario finished"));
                // The game stops at the disconnected screen, which is where this run ends.
                self.outcome.set(Some(Outcome::Finished));
                self.next(Stage::Done);
                minecraft.disconnect(&DisconnectionDetails::new("Scenario finished"), false);
            }
            Stage::Done => {}
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
}
