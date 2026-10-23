//! net/minecraft/core/HolderSet.java
//!
//! A set of a registry's elements: a list of them (`Direct`), or a tag (`Named`). A named set's
//! elements are bound from the tags the server sends (`update_tags`), which aren't ported, so a
//! named set here is its key alone. `Direct`'s `contents` set, `ListBacked` and the stream
//! methods come with their callers.

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::tags::tag_key::TagKey;

#[derive(Clone, Debug, PartialEq)]
pub enum HolderSet {
    /// `HolderSet.Direct`: the holders, in order.
    Direct(Vec<Holder>),
    /// `HolderSet.Named`: a tag of the registry.
    Named(TagKey),
}

impl HolderSet {
    /// `HolderSet.empty()`, a direct set of nothing.
    pub fn empty() -> HolderSet {
        HolderSet::Direct(Vec::new())
    }

    /// `unwrapKey()`: the tag of a named set.
    pub fn unwrap_key(&self) -> Option<&TagKey> {
        match self {
            HolderSet::Direct(_) => None,
            HolderSet::Named(key) => Some(key),
        }
    }
}
