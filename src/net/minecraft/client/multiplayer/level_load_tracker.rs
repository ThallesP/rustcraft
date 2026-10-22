//! net/minecraft/client/multiplayer/LevelLoadTracker.java
//!
//! Not ported yet: the server's side of the progress (`LevelLoadListener`, which the integrated
//! server reports to, and the status view the loading screen draws).
//!
//! `WaitingForPlayerChunk` waits for the renderer to compile the section the camera is in, which
//! calls back through `getPlayerCompiledSectionCallback`. The port has no renderer yet, so the
//! client listener calls it once the chunk the player is in has arrived, the closest thing a
//! headless client has (`ClientPacketListener`); the render track moves the call to its section
//! compiler. And it only waits while the camera is inside the level's height and the player is
//! alive and not a spectator, which the port can't tell yet (the level's height comes with the
//! dimension types, and the player's state with entities), so it always waits.

use std::cell::Cell;
use std::rc::Rc;

use crate::net::minecraft::util::util::Util;

/// Whether the client is ready to let the player into the level it joined.
#[derive(Debug, Default)]
pub struct LevelLoadTracker {
    client_state: Option<ClientState>,
    close_delay_ms: i64,
}

#[derive(Clone, Debug)]
enum ClientState {
    ClientLevelReady {
        ready_at: i64,
    },
    WaitingForPlayerChunk {
        player_section_ready: Rc<Cell<bool>>,
        timeout_after: i64,
    },
    WaitingForServer {
        timeout_after: i64,
    },
}

impl ClientState {
    fn tick(self) -> ClientState {
        match self {
            ClientState::WaitingForPlayerChunk {
                ref player_section_ready,
                timeout_after,
            } => {
                let ready = if Util::get_millis() > timeout_after {
                    eprintln!(
                        "Timed out while waiting for the client to load chunks, letting the \
                         player into the world anyway"
                    );
                    true
                } else {
                    player_section_ready.get()
                };
                if ready {
                    ClientState::ClientLevelReady {
                        ready_at: Util::get_millis(),
                    }
                } else {
                    self
                }
            }
            _ => self,
        }
    }

    fn loading_packets_received(self) -> ClientState {
        match self {
            ClientState::WaitingForServer { timeout_after } => ClientState::WaitingForPlayerChunk {
                player_section_ready: Rc::new(Cell::new(false)),
                timeout_after,
            },
            _ => self,
        }
    }
}

impl LevelLoadTracker {
    const CLIENT_WAIT_TIMEOUT_MS: i64 = 30000;
    pub const LEVEL_LOAD_CLOSE_DELAY_MS: i64 = 500;

    pub fn new() -> LevelLoadTracker {
        LevelLoadTracker::with_close_delay(0)
    }

    pub fn with_close_delay(close_delay_ms: i64) -> LevelLoadTracker {
        LevelLoadTracker {
            client_state: None,
            close_delay_ms,
        }
    }

    /// Java takes the player and the level, which only `WaitingForPlayerChunk`'s checks read.
    pub fn start_client_load(&mut self) {
        self.client_state = Some(ClientState::WaitingForServer {
            timeout_after: Util::get_millis() + LevelLoadTracker::CLIENT_WAIT_TIMEOUT_MS,
        });
    }

    pub fn tick_client_load(&mut self) {
        if let Some(client_state) = self.client_state.take() {
            self.client_state = Some(client_state.tick());
        }
    }

    pub fn is_level_ready(&self) -> bool {
        matches!(
            self.client_state,
            Some(ClientState::ClientLevelReady { ready_at })
                if Util::get_millis() >= ready_at + self.close_delay_ms
        )
    }

    pub fn loading_packets_received(&mut self) {
        if let Some(client_state) = self.client_state.take() {
            self.client_state = Some(client_state.loading_packets_received());
        }
    }

    /// What the renderer calls once it has compiled the camera's section.
    pub fn get_player_compiled_section_callback(&self) -> Option<impl Fn() + use<>> {
        match &self.client_state {
            Some(ClientState::WaitingForPlayerChunk {
                player_section_ready,
                ..
            }) => {
                let player_section_ready = Rc::clone(player_section_ready);
                Some(move || player_section_ready.set(true))
            }
            _ => None,
        }
    }
}
