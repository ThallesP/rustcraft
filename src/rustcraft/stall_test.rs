//! The connection's own thread, tested: once the level is ready, the game thread stalls, the
//! way a long frame on a slow machine does, for longer than a server waits for its keep-alive's
//! answer (15 seconds, twice). Vanilla answers keep-alives on Netty's thread meanwhile, and so
//! does the port (`Connection`), so the client is still connected a few seconds after: then it
//! disconnects and the run has finished. `rustcraft --stall SECONDS` runs it, and
//! `tools/port_client.py stall` on a fresh vanilla server.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::net::minecraft::client::minecraft::{Minecraft, TestDriver};
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::rustcraft::oracle::client::oracle_game_test::Outcome;

/// How long the client goes on after the stall before it checks it's still connected: long
/// enough for the server's next keep-alive (every 15 seconds) to have come and been answered.
const AFTER_STALL: Duration = Duration::from_secs(20);

/// Where the run is.
enum Stage {
    WaitingForLevel,
    Stalled(Instant),
    Done,
}

/// Stalls the game thread once, then checks the connection held.
pub struct StallTest {
    stall: Duration,
    stage: Stage,
    outcome: Rc<Cell<Option<Outcome>>>,
}

impl StallTest {
    /// The driver, and where it says how the run ended.
    pub fn new(stall: Duration) -> (StallTest, Rc<Cell<Option<Outcome>>>) {
        let outcome = Rc::new(Cell::new(Some(Outcome::Running)));
        let test = StallTest {
            stall,
            stage: Stage::WaitingForLevel,
            outcome: Rc::clone(&outcome),
        };
        (test, outcome)
    }
}

impl TestDriver for StallTest {
    fn after_tick(&mut self, minecraft: &Rc<Minecraft>) {
        match self.stage {
            Stage::WaitingForLevel => {
                if minecraft.get_connection().is_some() && !minecraft.is_level_loading() {
                    println!(
                        "stalling the game thread for {} s",
                        self.stall.as_secs_f64()
                    );
                    std::thread::sleep(self.stall);
                    println!("stalled for {} s", self.stall.as_secs_f64());
                    self.stage = Stage::Stalled(Instant::now());
                }
            }
            Stage::Stalled(since) => {
                if since.elapsed() < AFTER_STALL {
                    return;
                }
                let connection = minecraft
                    .get_connection()
                    .expect("the player is still in its level")
                    .borrow()
                    .get_connection();
                println!(
                    "still connected {} s after the stall",
                    AFTER_STALL.as_secs_f64()
                );
                connection.disconnect(DisconnectionDetails::new("Stall test finished"));
                self.outcome.set(Some(Outcome::Finished));
                self.stage = Stage::Done;
                minecraft.disconnect(&DisconnectionDetails::new("Stall test finished"), false);
            }
            Stage::Done => {}
        }
    }
}
