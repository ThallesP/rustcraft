//! net/minecraft/client/multiplayer/CommonListenerCookie.java
//!
//! Not ported yet: the telemetry manager, the chat state, the players seen and
//! the insecure chat warning, which come with their classes. The enabled features are their
//! identifiers, in the order the server sent them, until `FeatureFlagSet` comes.

use std::sync::Arc;

use crate::com::mojang::authlib::game_profile::GameProfile;
use crate::java::util::hash_map::HashMap;
use crate::net::minecraft::client::multiplayer::level_load_tracker::LevelLoadTracker;
use crate::net::minecraft::client::multiplayer::server_data::ServerData;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::server_links::ServerLinks;

/// What one client listener hands the next, when the connection moves on to another protocol.
pub struct CommonListenerCookie {
    pub level_load_tracker: LevelLoadTracker,
    pub local_game_profile: GameProfile,
    pub received_registries: Arc<RegistryAccess>,
    pub enabled_features: Vec<Identifier>,
    pub server_brand: Option<String>,
    pub server_data: Option<ServerData>,
    pub server_cookies: Vec<(Identifier, Arc<Vec<i8>>)>,
    pub custom_report_details: HashMap<String, String>,
    pub server_links: ServerLinks,
}
