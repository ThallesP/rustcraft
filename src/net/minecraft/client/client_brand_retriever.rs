//! net/minecraft/client/ClientBrandRetriever.java

/// The brand the client sends the server after login.
pub struct ClientBrandRetriever;

impl ClientBrandRetriever {
    pub const VANILLA_NAME: &'static str = "vanilla";

    pub fn get_client_mod_name() -> &'static str {
        "vanilla"
    }
}
