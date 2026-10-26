//! net/minecraft/client/DeltaTracker.java
//!
//! `Timer`'s target milliseconds per tick come from the level's tick rate manager
//! (`Minecraft.getTickTargetMillis`), which isn't ported yet, so it's always the default 50.

/// `DeltaTracker.Timer`: how many game ticks are due, from the milliseconds that went by.
#[derive(Clone, Debug)]
pub struct Timer {
    delta_ticks: f32,
    delta_tick_residual: f32,
    realtime_delta_ticks: f32,
    paused_delta_tick_residual: f32,
    last_ms: i64,
    last_ui_ms: i64,
    ms_per_tick: f32,
    paused: bool,
    frozen: bool,
}

impl Timer {
    pub fn new(ticks_per_second: f32, current_ms: i64) -> Timer {
        Timer {
            delta_ticks: 0.0,
            delta_tick_residual: 0.0,
            realtime_delta_ticks: 0.0,
            paused_delta_tick_residual: 0.0,
            last_ms: current_ms,
            last_ui_ms: current_ms,
            ms_per_tick: 1000.0 / ticks_per_second,
            paused: false,
            frozen: false,
        }
    }

    /// The ticks due since the last call, keeping the fraction for the next.
    pub fn advance_game_time(&mut self, current_ms: i64) -> i32 {
        self.delta_ticks = (current_ms - self.last_ms) as f32 / self.ms_per_tick;
        self.last_ms = current_ms;
        self.delta_tick_residual += self.delta_ticks;
        let ticks = self.delta_tick_residual as i32;
        self.delta_tick_residual -= ticks as f32;
        ticks
    }

    /// `advanceRealTime`: the ticks' worth of milliseconds since the last frame, whatever the
    /// game's ticks did (the panorama's spin).
    pub fn advance_real_time(&mut self, current_ms: i64) {
        self.realtime_delta_ticks = (current_ms - self.last_ui_ms) as f32 / self.ms_per_tick;
        self.last_ui_ms = current_ms;
    }

    pub fn get_realtime_delta_ticks(&self) -> f32 {
        self.realtime_delta_ticks
    }

    /// When the next tick is due, in the clock's milliseconds: what the game waits for between
    /// frames when there's nothing to draw.
    pub fn next_tick_ms(&self) -> f64 {
        self.last_ms as f64 + f64::from((1.0 - self.delta_tick_residual) * self.ms_per_tick)
    }

    pub fn update_pause_state(&mut self, pause_state: bool) {
        if pause_state {
            self.pause();
        } else {
            self.un_pause();
        }
    }

    fn pause(&mut self) {
        if !self.paused {
            self.paused_delta_tick_residual = self.delta_tick_residual;
        }

        self.paused = true;
    }

    fn un_pause(&mut self) {
        if self.paused {
            self.delta_tick_residual = self.paused_delta_tick_residual;
        }

        self.paused = false;
    }

    pub fn update_frozen_state(&mut self, frozen: bool) {
        self.frozen = frozen;
    }

    pub fn get_game_time_delta_ticks(&self) -> f32 {
        self.delta_ticks
    }

    /// How far into the next tick the frame is: 1 while the game is frozen, unless that's ignored.
    pub fn get_game_time_delta_partial_tick(&self, ignore_frozen_game: bool) -> f32 {
        if !ignore_frozen_game && self.frozen {
            1.0
        } else if self.paused {
            self.paused_delta_tick_residual
        } else {
            self.delta_tick_residual
        }
    }
}
