//! oracle/client/src/main/java/rustcraft/oracle/client/ServerTicks.java: the client's ticks in
//! step with the server's, for a scenario with a `tick_rate` (TESTING.md, "Paced scenarios"). The
//! server is slowed down to the scenario's rate, its ticks are found from a packet it sends as a
//! tick ends (`set_time`), and each of the client's ticks is released at a slot halfway between two
//! of the server's. Java's test thread sleeps until the slot while the client is parked, has the
//! client handle what it has queued, then releases the tick; the port's driver does the same where
//! Fabric parks the client (`TestDriver::before_tick`), the network running on its own thread.

use std::time::{Duration, Instant};

use crate::net::minecraft::client::minecraft::Minecraft;

/// The slots of a paced scenario's ticks.
#[derive(Clone, Debug)]
pub struct ServerTicks {
    /// The server's tick.
    period: Duration,
    next_slot: Instant,
    /// The slot of the tick about to run, once it's been waited for.
    slot: Option<Instant>,
    /// Whether the ticks are the scenario's, from the marker on, and how many of them missed
    /// their slot.
    replaying: bool,
    missed: i32,
}

impl ServerTicks {
    /// What a run that fell behind fails with, which `tools/port_client.py` runs again.
    pub const FELL_BEHIND: &'static str = "fell behind the server's ticks";
    /// The frame rate a paced client is held to, Java's `FRAMES_PER_SECOND`: it only ticks at the
    /// slots, so it needs no more frames than this, and the frames llvmpipe draws at 120 a second
    /// take the CPU the server's ticks and the client's own need to keep to the slots.
    pub const FRAMES_PER_SECOND: i32 = 10;

    /// The ticks of a server at `tick_rate` ticks a second, one of which ended as a packet sent
    /// then came (`reference`): its slots are half a tick after that, plus whole ticks, from the
    /// first one still to come.
    pub fn new(tick_rate: f32, reference: Instant) -> ServerTicks {
        // `TickRateManager.setTickRate`'s nanoseconds per tick.
        let period = Duration::from_nanos((1.0E9 / f64::from(tick_rate)) as u64);
        let mut next_slot = reference + period / 2;
        let now = Instant::now();
        while next_slot < now {
            next_slot += period;
        }
        ServerTicks {
            period,
            next_slot,
            slot: None,
            replaying: false,
            missed: 0,
        }
    }

    /// From the scenario's first tick on, a tick that misses its slot fails the run.
    pub fn start_scenario(&mut self) {
        self.replaying = true;
    }

    /// How many of the scenario's ticks missed their slot.
    pub fn missed(&self) -> i32 {
        self.missed
    }

    /// `awaitSlot`: sleeps until the next slot, then has the client handle what it has queued, so
    /// whatever the server sent before the slot is handled before the tick. A client that gets
    /// there more than a third of a tick after the slot waits for the next slot instead, and the
    /// server ticks without it.
    pub fn await_slot(&mut self, minecraft: &Minecraft) {
        loop {
            let now = Instant::now();
            if now < self.next_slot {
                std::thread::sleep(self.next_slot - now);
            }
            minecraft.packet_processor().process_queued_packets();
            let slot = self.next_slot;
            let late = Instant::now().saturating_duration_since(slot);
            self.next_slot += self.period;
            if late <= self.period / 3 {
                self.slot = Some(slot);
                return;
            }
            self.miss("got to its slot", late);
        }
    }

    /// As the tick ends: one that ends more than a third of a tick after its slot may have come
    /// to the server after its next tick.
    pub fn tick_ended(&mut self) {
        if let Some(slot) = self.slot.take() {
            let late = Instant::now().saturating_duration_since(slot);
            if late > self.period / 3 {
                self.miss("ended its tick", late);
            }
        }
    }

    fn miss(&mut self, what: &str, late: Duration) {
        if self.replaying {
            self.missed += 1;
        }
        println!(
            "rustcraft oracle: {what} {} ms after its slot, {}",
            late.as_millis(),
            if self.replaying {
                "in the scenario"
            } else {
                "before the scenario"
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::ServerTicks;

    #[test]
    fn slots_are_halfway_between_the_server_s_ticks() {
        let reference = Instant::now();
        let ticks = ServerTicks::new(2.0, reference);
        assert_eq!(ticks.period, Duration::from_millis(500));
        assert_eq!(ticks.next_slot, reference + Duration::from_millis(250));

        // A reference from long ago: the first slot still to come, on the same grid.
        let reference = Instant::now() - Duration::from_millis(1_010);
        let ticks = ServerTicks::new(2.0, reference);
        assert_eq!(ticks.next_slot, reference + Duration::from_millis(1_250));
    }
}
