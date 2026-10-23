//! net/minecraft/client/multiplayer/MultiPlayerGameMode.java
//!
//! Not ported yet: nearly all of it: breaking and placing blocks, using items, attacking,
//! containers, the carried item (`ensureHasSentCarriedItem`, which only sends when the selected
//! slot changes, and nothing changes it yet), and the abilities a game mode gives the player
//! (`adjustPlayer`, `setLocalMode`'s), which come with the player. What's here is what ticks the
//! connection while the player is in a level, and the game mode the server set. The game shares
//! it, so its state is in cells.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
use crate::net::minecraft::world::level::game_type::GameType;

/// The player's side of a game mode, on a server.
pub struct MultiPlayerGameMode {
    connection: Rc<RefCell<ClientPacketListener>>,
    local_player_mode: Cell<GameType>,
    previous_local_player_mode: Cell<Option<GameType>>,
}

impl MultiPlayerGameMode {
    pub fn new(connection: Rc<RefCell<ClientPacketListener>>) -> MultiPlayerGameMode {
        MultiPlayerGameMode {
            connection,
            local_player_mode: Cell::new(GameType::DEFAULT_MODE),
            previous_local_player_mode: Cell::new(None),
        }
    }

    /// `setLocalMode(GameType, GameType)`.
    pub fn set_local_mode(&self, mode: GameType, previous_mode: Option<GameType>) {
        self.local_player_mode.set(mode);
        self.previous_local_player_mode.set(previous_mode);
    }

    pub fn get_player_mode(&self) -> GameType {
        self.local_player_mode.get()
    }

    pub fn get_previous_player_mode(&self) -> Option<GameType> {
        self.previous_local_player_mode.get()
    }

    pub fn can_hurt_player(&self) -> bool {
        self.local_player_mode.get().is_survival()
    }

    pub fn has_experience(&self) -> bool {
        self.local_player_mode.get().is_survival()
    }

    pub fn tick(&self) {
        let connection = self.connection.borrow().get_connection();
        if connection.is_connected() {
            connection.tick();
        } else {
            connection.handle_disconnection();
        }
    }
}
