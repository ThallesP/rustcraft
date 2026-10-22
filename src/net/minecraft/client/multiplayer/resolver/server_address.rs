//! net/minecraft/client/multiplayer/resolver/ServerAddress.java
//!
//! Java's holds a Guava `HostAndPort`, whose parsing (`fromString`) is ported here: a host and
//! an optional port after a colon, or an IPv6 address in brackets, or one without brackets and no
//! port. `getHost` runs the host through `IDN.toASCII`, which leaves an ASCII host as it is; a
//! host with other characters isn't converted yet, and is sent as it is.

use std::fmt;

/// A server's host and port, as the player typed them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerAddress {
    host: String,
    port: i32,
    has_brackets: bool,
}

impl ServerAddress {
    const DEFAULT_PORT: i32 = 25565;

    pub fn new(host: &str, port: i32) -> ServerAddress {
        ServerAddress {
            host: host.to_string(),
            port,
            has_brackets: false,
        }
    }

    /// `INVALID`.
    fn invalid() -> ServerAddress {
        ServerAddress::new("server.invalid", ServerAddress::DEFAULT_PORT)
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn get_port(&self) -> i32 {
        self.port
    }

    /// The host and port, the default port without one, and `server.invalid` for what doesn't
    /// parse or has no host.
    pub fn parse_string(input: Option<&str>) -> ServerAddress {
        let Some(input) = input else {
            return ServerAddress::invalid();
        };

        match ServerAddress::host_and_port(input) {
            Some((host, _, _)) if host.is_empty() => ServerAddress::invalid(),
            Some((host, port, has_brackets)) => ServerAddress {
                host,
                port: port.unwrap_or(ServerAddress::DEFAULT_PORT),
                has_brackets,
            },
            None => {
                eprintln!("Failed to parse URL {input}");
                ServerAddress::invalid()
            }
        }
    }

    pub fn is_valid_address(input: &str) -> bool {
        ServerAddress::host_and_port(input).is_some_and(|(host, _, _)| !host.is_empty())
    }

    pub fn parse_port(str: &str) -> i32 {
        str.trim().parse().unwrap_or(ServerAddress::DEFAULT_PORT)
    }

    /// `HostAndPort.fromString`: the host, the port if there is one, and whether the host was in
    /// brackets, or `None` where Guava throws `IllegalArgumentException`.
    fn host_and_port(input: &str) -> Option<(String, Option<i32>, bool)> {
        let (host, port, has_brackets) = if let Some(rest) = input.strip_prefix('[') {
            let close = rest.find(']')?;
            let host = &rest[..close];
            let after = &rest[close + 1..];
            if after.is_empty() {
                (host, None, true)
            } else {
                (host, Some(after.strip_prefix(':')?), true)
            }
        } else {
            match input.find(':') {
                Some(colon) if !input[colon + 1..].contains(':') => {
                    (&input[..colon], Some(&input[colon + 1..]), false)
                }
                _ => (input, None, false),
            }
        };

        let port = match port {
            None => None,
            Some(port) => {
                if port.starts_with('+') || !port.is_ascii() {
                    return None;
                }
                let port: i32 = port.parse().ok()?;
                if !(0..=65535).contains(&port) {
                    return None;
                }
                Some(port)
            }
        };
        Some((host.to_string(), port, has_brackets))
    }
}

/// `HostAndPort.toString`: `host:port`, with an IPv6 host in brackets.
impl fmt::Display for ServerAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_brackets || self.host.contains(':') {
            write!(f, "[{}]:{}", self.host, self.port)
        } else {
            write!(f, "{}:{}", self.host, self.port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ServerAddress;

    #[test]
    fn parses_like_guava() {
        let parsed = |input| {
            let address = ServerAddress::parse_string(Some(input));
            (address.get_host().to_string(), address.get_port())
        };
        assert_eq!(parsed("127.0.0.1:25566"), ("127.0.0.1".to_string(), 25566));
        assert_eq!(parsed("localhost"), ("localhost".to_string(), 25565));
        assert_eq!(parsed("[::1]:7"), ("::1".to_string(), 7));
        assert_eq!(parsed("::1"), ("::1".to_string(), 25565));
        assert_eq!(parsed(":25565"), ("server.invalid".to_string(), 25565));
        assert_eq!(parsed("host:+1"), ("server.invalid".to_string(), 25565));
        assert_eq!(parsed("host:65536"), ("server.invalid".to_string(), 25565));
    }
}
