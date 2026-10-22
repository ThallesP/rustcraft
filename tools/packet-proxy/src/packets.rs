use std::collections::HashMap;
use std::io;
use std::path::Path;

use serde_json::Value;

use crate::frame::invalid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

impl State {
    const ALL: [State; 5] = [
        State::Handshake,
        State::Status,
        State::Login,
        State::Configuration,
        State::Play,
    ];

    /// The name `packets.json` and the capture format use.
    pub fn name(self) -> &'static str {
        match self {
            State::Handshake => "handshake",
            State::Status => "status",
            State::Login => "login",
            State::Configuration => "configuration",
            State::Play => "play",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Serverbound,
    Clientbound,
}

impl Direction {
    pub fn name(self) -> &'static str {
        match self {
            Direction::Serverbound => "serverbound",
            Direction::Clientbound => "clientbound",
        }
    }
}

/// Packet ids and names per state and direction, from the data generator's `packets.json`.
pub struct PacketTable {
    names: HashMap<(State, Direction, i32), String>,
    ids: HashMap<(State, Direction, String), i32>,
}

impl PacketTable {
    pub fn load(path: &Path) -> io::Result<Self> {
        let json: Value = serde_json::from_str(&std::fs::read_to_string(path)?)
            .map_err(|error| invalid(error.to_string()))?;
        Self::from_json(&json).map_err(|message| invalid(format!("{}: {message}", path.display())))
    }

    pub fn from_json(json: &Value) -> Result<Self, String> {
        let mut names = HashMap::new();
        let mut ids = HashMap::new();
        for state in State::ALL {
            for direction in [Direction::Serverbound, Direction::Clientbound] {
                let Some(packets) = json
                    .get(state.name())
                    .and_then(|packets| packets.get(direction.name()))
                else {
                    continue;
                };
                let packets = packets.as_object().ok_or(format!(
                    "{}.{} isn't an object",
                    state.name(),
                    direction.name()
                ))?;
                for (name, packet) in packets {
                    let id = packet["protocol_id"]
                        .as_i64()
                        .ok_or(format!("{name} has no protocol_id"))?
                        as i32;
                    names.insert((state, direction, id), name.clone());
                    ids.insert((state, direction, name.clone()), id);
                }
            }
        }

        if names.is_empty() {
            return Err("no packets found".to_string());
        }
        Ok(Self { names, ids })
    }

    pub fn name(&self, state: State, direction: Direction, id: i32) -> Option<&str> {
        self.names.get(&(state, direction, id)).map(String::as_str)
    }

    pub fn id(&self, state: State, direction: Direction, name: &str) -> Option<i32> {
        self.ids.get(&(state, direction, name.to_string())).copied()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn looks_up_both_ways() {
        let table = PacketTable::from_json(&json!({
            "handshake": { "serverbound": { "minecraft:intention": { "protocol_id": 0 } } },
            "play": { "clientbound": { "minecraft:keep_alive": { "protocol_id": 45 } } },
        }))
        .unwrap();

        assert_eq!(
            table.name(State::Play, Direction::Clientbound, 45),
            Some("minecraft:keep_alive")
        );
        assert_eq!(table.name(State::Play, Direction::Serverbound, 45), None);
        assert_eq!(
            table.id(
                State::Handshake,
                Direction::Serverbound,
                "minecraft:intention"
            ),
            Some(0)
        );
    }
}
