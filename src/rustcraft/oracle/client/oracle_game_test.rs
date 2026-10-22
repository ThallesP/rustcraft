//! oracle/client/src/main/java/rustcraft/oracle/client/OracleGameTest.java, on a server
//! (`runMultiplayer`): joins, puts the world in the scenario's state with the same commands,
//! waits like the oracle client does, sends the scenario-start marker and replays the scenario's
//! ticks, then disconnects. `tools/port_client.py capture` runs it behind `packet-proxy`.
//!
//! Not yet like the oracle client's: after the settling ticks it waits for the player to be on
//! the ground, which only the player track's movement decides, so it settles for the ticks alone;
//! and the replay holds no keys and writes no trace or screenshots, which come with movement and
//! the renderer (gates 2 and 3). The port has no screens, so waiting for the level's loading
//! screen to close is waiting for the level to be ready, which closes it in vanilla too, and with
//! no chunk storage yet "chunks rendered" is as many chunks as the render distance spans having
//! come in the finished chunk batches.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use crate::net::minecraft::client::minecraft::{Minecraft, TestDriver};
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::protocol::common::custom::discarded_payload::DiscardedPayload;
use crate::net::minecraft::network::protocol::common::serverbound_custom_payload_packet::ServerboundCustomPayloadPacket;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::level::particle_status::ParticleStatus;
use crate::rustcraft::oracle::client::scenario::{self, Scenario};

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
    stage: Stage,
    ticks_in_stage: i32,
    replayed_ticks: i32,
    outcome: Rc<Cell<Option<Outcome>>>,
}

impl OracleGameTest {
    const SETTLE_TICKS: i32 = 20;
    const LOAD_TIMEOUT_TICKS: i32 = 60 * 20;

    pub fn scenario_start() -> Identifier {
        Identifier::from_namespace_and_path("rustcraft", "scenario_start")
    }

    /// Sets the options the oracle client sets (`applyOptions`) that the port has, and returns the
    /// driver and where its outcome goes.
    pub fn new(
        minecraft: &Minecraft,
        scenario: Scenario,
    ) -> (OracleGameTest, Rc<Cell<Option<Outcome>>>) {
        {
            let mut options = minecraft.options.borrow_mut();
            options.set_render_distance(scenario.options.render_distance);
            options.set_particles(ParticleStatus::MINIMAL);
            options.joined_first_server = true;
        }
        let outcome = Rc::new(Cell::new(Some(Outcome::Running)));
        (
            OracleGameTest {
                scenario,
                stage: Stage::WaitingForLevel,
                ticks_in_stage: 0,
                replayed_ticks: 0,
                outcome: Rc::clone(&outcome),
            },
            outcome,
        )
    }

    /// The commands that put the world in the scenario's state, the same in both modes.
    pub fn setup_commands(scenario: &Scenario, player: &str) -> Vec<String> {
        vec![
            "gamerule advance_time false".to_string(),
            "gamerule advance_weather false".to_string(),
            "gamerule random_tick_speed 0".to_string(),
            "gamerule spawn_mobs false".to_string(),
            "difficulty normal".to_string(),
            format!("time set {}", scenario.world.time),
            "weather clear".to_string(),
            format!("gamemode {} {player}", scenario.world.game_mode),
            "tp @e[type=!minecraft:player] 0 -1000 0".to_string(),
            format!("tp {player} {}", OracleGameTest::position(&scenario.player)),
        ]
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

    /// Every chunk within the render distance has come: (2d + 1)² of them, which the client's
    /// finished chunk batches count until chunk storage comes.
    fn are_chunks_rendered(&self, minecraft: &Minecraft) -> bool {
        let distance = minecraft.options.borrow().get_effective_render_distance();
        let chunks = (2 * distance + 1) * (2 * distance + 1);
        minecraft.level.borrow().is_some()
            && minecraft.get_connection().is_some_and(|connection| {
                connection.borrow().chunks_in_finished_batches() >= chunks
            })
    }

    fn scenario_ticks(&self) -> i32 {
        self.scenario.steps.iter().map(|step| step.ticks).sum()
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
                if self.ticks_in_stage >= OracleGameTest::SETTLE_TICKS {
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
                }
            }
            Stage::Replaying => {
                self.replayed_ticks += 1;
                if self.replayed_ticks >= self.scenario_ticks() {
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
