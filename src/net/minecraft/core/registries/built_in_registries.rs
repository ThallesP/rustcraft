//! net/minecraft/core/registries/BuiltInRegistries.java
//!
//! Not ported yet: every registry but the attributes', which come with their classes (blocks'
//! is `Block`'s own numbering, `BLOCK_STATE_REGISTRY`). A registry here is its elements' keys in
//! id order, the order Java registers them in (`Registry`).

use std::sync::LazyLock;

use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::core::registry::Registry;
use crate::net::minecraft::world::entity::ai::attributes::attributes::{self, Attributes};

static ATTRIBUTE: LazyLock<Registry> = LazyLock::new(|| {
    Registry::new(
        (**Registries::ATTRIBUTE).clone(),
        attributes::REGISTERED
            .iter()
            .map(|(_, name)| {
                Attributes::holder(name)
                    .unwrap_key()
                    .expect("a reference has its key")
                    .clone()
            })
            .collect(),
    )
});

/// The registries the client has built in, which the server doesn't send.
pub struct BuiltInRegistries;

impl BuiltInRegistries {
    pub const ATTRIBUTE: &'static LazyLock<Registry> = &ATTRIBUTE;

    /// The ones ported, which the client's registries start from (`ClientRegistryLayer.STATIC`).
    pub fn registries() -> Vec<Registry> {
        vec![Registry::clone(BuiltInRegistries::ATTRIBUTE)]
    }
}
