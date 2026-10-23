//! net/minecraft/network/protocol/status/ServerStatus.java
//!
//! A stand-in until text components come. Java's record holds the status's parts (the
//! description, a `Component`; the players with a sample of their profiles; the version; the
//! favicon; whether the server enforces secure chat), and its `CODEC` reads them from the JSON
//! the server sends, with DataFixerUpper's `RecordCodecBuilder` and `ComponentSerialization`'s
//! codec over Gson's tree. None of those is ported yet (PROGRESS.md has the plan), so this holds
//! the JSON's text as the server sent it, which the status response's codec reads and writes as
//! it is. Java also refuses text that isn't JSON, or a status whose parts don't parse, which this
//! doesn't.

/// What a server says about itself in the server list: its status's JSON.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerStatus {
    json: String,
}

impl ServerStatus {
    pub fn new(json: String) -> ServerStatus {
        ServerStatus { json }
    }

    /// The JSON the server sent, which Java's `CODEC` would parse into the record's parts.
    pub fn json(&self) -> &str {
        &self.json
    }
}
