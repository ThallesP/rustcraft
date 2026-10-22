//! net/minecraft/SharedConstants.java
//!
//! Not ported yet: everything but the protocol version and the current version's id.
//! `getCurrentVersion()` is the version the game reads from its jar's `version.json`
//! (`DetectedVersion`): its `id` is `MINECRAFT_VERSION`'s, and its `protocol_version` the same
//! 777 as `getProtocolVersion()`. The port has no jar to read it from, so they're constants here,
//! which the golden cases of their callers check (`KnownPack.vanilla`).

/// The game's constants.
pub struct SharedConstants;

impl SharedConstants {
    pub const RELEASE_NETWORK_PROTOCOL_VERSION: i32 = 777;
    pub const SNAPSHOT_NETWORK_PROTOCOL_VERSION: i32 = 338;

    pub fn get_protocol_version() -> i32 {
        777
    }

    /// `getCurrentVersion().id()`.
    pub fn get_current_version_id() -> &'static str {
        "26.3"
    }
}
