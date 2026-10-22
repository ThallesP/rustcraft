use std::io;

use crate::frame::{invalid, read_var_int};
use crate::packets::{Direction, State};

/// Where one connection is in the protocol. Each direction switches state on its own: a side
/// switches right after sending the packet that ends the state, which is the order the packets
/// arrive at the proxy in.
pub struct Session {
    pub serverbound: State,
    pub clientbound: State,
    pub compressed: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            serverbound: State::Handshake,
            clientbound: State::Handshake,
            compressed: false,
        }
    }
}

impl Session {
    pub fn state(&self, direction: Direction) -> State {
        match direction {
            Direction::Serverbound => self.serverbound,
            Direction::Clientbound => self.clientbound,
        }
    }

    /// Applies what a packet changes for the packets after it. Unknown packets (`name` is
    /// `None`) never change anything.
    pub fn after_packet(
        &mut self,
        direction: Direction,
        state: State,
        name: Option<&str>,
        payload: &[u8],
    ) -> io::Result<()> {
        use Direction::{Clientbound, Serverbound};
        use State::{Configuration, Handshake, Login, Play, Status};

        let Some(name) = name else { return Ok(()) };
        match (direction, state, name) {
            // The server answers nothing until the intention arrives, so both directions switch here.
            (Serverbound, Handshake, "minecraft:intention") => {
                let next = match intention(payload)? {
                    1 => Status,
                    2 | 3 => Login,
                    other => return Err(invalid(format!("unknown intention {other}"))),
                };
                self.serverbound = next;
                self.clientbound = next;
            }
            (Clientbound, Login, "minecraft:hello") => {
                return Err(io::Error::other(
                    "the server asked for encryption: captures need a server with online-mode=false",
                ));
            }
            (Clientbound, Login, "minecraft:login_compression") => {
                let (threshold, _) = read_var_int(payload)?
                    .ok_or_else(|| invalid("login_compression without a threshold"))?;
                self.compressed = threshold >= 0;
            }
            (Clientbound, Login, "minecraft:login_finished") => self.clientbound = Configuration,
            (Serverbound, Login, "minecraft:login_acknowledged") => {
                self.serverbound = Configuration
            }
            (Clientbound, Configuration, "minecraft:finish_configuration") => {
                self.clientbound = Play
            }
            (Serverbound, Configuration, "minecraft:finish_configuration") => {
                self.serverbound = Play
            }
            (Clientbound, Play, "minecraft:start_configuration") => {
                self.clientbound = Configuration
            }
            (Serverbound, Play, "minecraft:configuration_acknowledged") => {
                self.serverbound = Configuration
            }
            _ => {}
        }
        Ok(())
    }
}

// ClientIntentionPacket: protocol version, host, port, intention.
fn intention(payload: &[u8]) -> io::Result<i32> {
    let truncated = || invalid("truncated intention packet");
    let (_, protocol_length) = read_var_int(payload)?.ok_or_else(truncated)?;
    let rest = &payload[protocol_length..];
    let (host_length, prefix) = read_var_int(rest)?.ok_or_else(truncated)?;
    let port_end = prefix + host_length as usize + 2;
    let rest = rest.get(port_end..).ok_or_else(truncated)?;
    let (intention, _) = read_var_int(rest)?.ok_or_else(truncated)?;
    Ok(intention)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::write_var_int;

    fn intention_payload(intention: i32) -> Vec<u8> {
        let mut payload = Vec::new();
        write_var_int(777, &mut payload);
        write_var_int(9, &mut payload);
        payload.extend_from_slice(b"localhost");
        payload.extend_from_slice(&25565u16.to_be_bytes());
        write_var_int(intention, &mut payload);
        payload
    }

    fn apply(session: &mut Session, direction: Direction, name: &str, payload: &[u8]) {
        let state = session.state(direction);
        session
            .after_packet(direction, state, Some(name), payload)
            .unwrap();
    }

    #[test]
    fn follows_a_login_into_play_and_back_to_configuration() {
        use Direction::{Clientbound, Serverbound};

        let mut session = Session::default();
        apply(
            &mut session,
            Serverbound,
            "minecraft:intention",
            &intention_payload(2),
        );
        assert_eq!(
            (session.serverbound, session.clientbound),
            (State::Login, State::Login)
        );

        apply(
            &mut session,
            Clientbound,
            "minecraft:login_compression",
            &[0x80, 0x02],
        );
        assert!(session.compressed);

        apply(&mut session, Clientbound, "minecraft:login_finished", &[]);
        assert_eq!(
            (session.serverbound, session.clientbound),
            (State::Login, State::Configuration)
        );
        apply(
            &mut session,
            Serverbound,
            "minecraft:login_acknowledged",
            &[],
        );
        assert_eq!(session.serverbound, State::Configuration);

        apply(
            &mut session,
            Clientbound,
            "minecraft:finish_configuration",
            &[],
        );
        apply(
            &mut session,
            Serverbound,
            "minecraft:finish_configuration",
            &[],
        );
        assert_eq!(
            (session.serverbound, session.clientbound),
            (State::Play, State::Play)
        );

        apply(
            &mut session,
            Clientbound,
            "minecraft:start_configuration",
            &[],
        );
        apply(
            &mut session,
            Serverbound,
            "minecraft:configuration_acknowledged",
            &[],
        );
        assert_eq!(
            (session.serverbound, session.clientbound),
            (State::Configuration, State::Configuration)
        );
    }

    #[test]
    fn status_intention_switches_to_status() {
        let mut session = Session::default();
        apply(
            &mut session,
            Direction::Serverbound,
            "minecraft:intention",
            &intention_payload(1),
        );
        assert_eq!(
            (session.serverbound, session.clientbound),
            (State::Status, State::Status)
        );
    }

    #[test]
    fn refuses_encryption() {
        let mut session = Session {
            serverbound: State::Login,
            clientbound: State::Login,
            compressed: false,
        };
        let error = session
            .after_packet(
                Direction::Clientbound,
                State::Login,
                Some("minecraft:hello"),
                &[],
            )
            .unwrap_err();
        assert!(error.to_string().contains("online-mode=false"));
    }
}
