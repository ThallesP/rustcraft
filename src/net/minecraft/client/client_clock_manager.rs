//! net/minecraft/client/ClientClockManager.java
//!
//! Java keeps its clocks in a `HashMap` of their holders, which hash by identity, so it ticks them
//! in another order every run; each clock's tick is its own, so the order doesn't show. Here they
//! are in the order they were first asked for.

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::network::protocol::game::clientbound_set_time_packet::ClockUpdates;
use crate::net::minecraft::util::mth::Mth;

/// The client's clocks (the overworld's day, the end's), which the server sets and the level's
/// ticks advance between its updates.
#[derive(Clone, Debug, Default)]
pub struct ClientClockManager {
    clocks: Vec<(Holder, ClientClockInstance)>,
    last_tick_game_time: i64,
}

impl ClientClockManager {
    pub fn new() -> ClientClockManager {
        ClientClockManager::default()
    }

    /// The clock's state, a new one (no ticks, at rate 1) the first time it's asked for.
    pub fn get_instance(&mut self, definition: &Holder) -> &mut ClientClockInstance {
        let index = match self
            .clocks
            .iter()
            .position(|(clock, _)| clock == definition)
        {
            Some(index) => index,
            None => {
                self.clocks
                    .push((definition.clone(), ClientClockInstance::new()));
                self.clocks.len() - 1
            }
        };
        &mut self.clocks[index].1
    }

    /// Advances every clock by the game ticks since the last call, times its rate, keeping the
    /// part of a tick past its whole ticks.
    pub fn tick(&mut self, game_time: i64) {
        let game_time_delta = game_time.wrapping_sub(self.last_tick_game_time);
        self.last_tick_game_time = game_time;

        for (_, instance) in &mut self.clocks {
            let new_partial_ticks = f64::from(instance.partial_tick)
                + game_time_delta as f64 * f64::from(instance.rate);
            let full_ticks = i64::from(Mth::floor_f64(new_partial_ticks));
            instance.partial_tick = (new_partial_ticks - full_ticks as f64) as f32;
            instance.total_ticks = instance.total_ticks.wrapping_add(full_ticks);
        }
    }

    /// Ticks to the server's game time, then sets the clocks it sent.
    pub fn handle_updates(&mut self, game_time: i64, updates: &ClockUpdates) {
        self.tick(game_time);
        for (definition, state) in updates {
            let clock = self.get_instance(definition);
            clock.total_ticks = state.total_ticks();
            clock.partial_tick = state.partial_tick();
            clock.rate = state.rate();
        }
    }
}

/// `ClientClockManager.ClientClockInstance`: a clock's ticks, the part of a tick past them, and
/// its ticks per game tick.
#[derive(Clone, Copy, Debug)]
pub struct ClientClockInstance {
    total_ticks: i64,
    partial_tick: f32,
    rate: f32,
}

impl ClientClockInstance {
    fn new() -> ClientClockInstance {
        ClientClockInstance {
            total_ticks: 0,
            partial_tick: 0.0,
            rate: 1.0,
        }
    }

    pub fn total_ticks(&self) -> i64 {
        self.total_ticks
    }

    pub fn partial_tick(&self) -> f32 {
        self.partial_tick
    }

    pub fn rate(&self) -> f32 {
        self.rate
    }

    pub fn is_paused(&self) -> bool {
        self.rate == 0.0
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ClientClockManager;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::network::protocol::game::clientbound_set_time_packet::ClockUpdates;
    use crate::net::minecraft::world::clock::clock_network_state::ClockNetworkState;

    // Each op's output: null for `tick` and `handleUpdates`, and a clock's
    // [totalTicks, partialTick, rate, isPaused] for `getInstance`.
    fn run(program: &Value) -> Value {
        let mut manager = ClientClockManager::new();
        let outputs = Vec::<Value>::decode(program)
            .iter()
            .map(|op| {
                let op = op.as_array().expect("an op is an array");
                match op[0].as_str() {
                    Some("tick") => {
                        manager.tick(i64::decode(&op[1]));
                        Value::Null
                    }
                    Some("handleUpdates") => {
                        let mut updates = ClockUpdates::new();
                        for (clock, state) in Vec::<(Holder, ClockNetworkState)>::decode(&op[2]) {
                            updates.put(clock, state);
                        }
                        manager.handle_updates(i64::decode(&op[1]), &updates);
                        Value::Null
                    }
                    Some("getInstance") => {
                        let clock = manager.get_instance(&Holder::decode(&op[1]));
                        Value::from(vec![
                            clock.total_ticks().encode(),
                            clock.partial_tick().encode(),
                            clock.rate().encode(),
                            clock.is_paused().encode(),
                        ])
                    }
                    _ => panic!("unknown op {op:?}"),
                }
            })
            .collect::<Vec<_>>();
        Value::from(outputs)
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.client.ClientClockManager");
        golden.check("programs", |program: Value| run(&program));
        golden.finish();
    }
}
