//! net/minecraft/network/chat/contents/KeybindResolver.java
//!
//! What a key binding's name shows as. Java's default, which this is, shows the name itself; the
//! client sets `KeyMapping::createNameSupplier` in `Minecraft`'s constructor, which comes with
//! the key mappings.

use crate::net::minecraft::network::chat::component::Component;

pub struct KeybindResolver;

impl KeybindResolver {
    /// `keyResolver.apply(name).get()`.
    pub fn resolve(name: &str) -> Component {
        Component::literal(name).into()
    }
}
