//! net/minecraft/network/RegistryFriendlyByteBuf.java
//!
//! Java's extends `FriendlyByteBuf`; this one holds the `FriendlyByteBuf` it extends and derefs
//! to it (PORTING.md), and is `AsMut` of it and of the `ByteBuf` under it, so their codecs work
//! on it. The registries are shared, like Java's reference to them.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::simple_unbound_protocol::ContextWrapper;

/// A buffer whose codecs can read and write registries' elements by id: the game protocol's.
#[derive(Clone, Debug)]
pub struct RegistryFriendlyByteBuf {
    source: FriendlyByteBuf,
    registry_access: Arc<RegistryAccess>,
}

impl RegistryFriendlyByteBuf {
    pub fn new(source: ByteBuf, registry_access: Arc<RegistryAccess>) -> RegistryFriendlyByteBuf {
        RegistryFriendlyByteBuf {
            source: FriendlyByteBuf::new(source),
            registry_access,
        }
    }

    pub fn registry_access(&self) -> &RegistryAccess {
        &self.registry_access
    }

    /// Wraps a connection's buffer with the registries, for the game protocol's codecs.
    pub fn decorator(
        registry_access: Arc<RegistryAccess>,
    ) -> ContextWrapper<RegistryFriendlyByteBuf> {
        Arc::new(move |buf| RegistryFriendlyByteBuf::new(buf, Arc::clone(&registry_access)))
    }
}

impl Deref for RegistryFriendlyByteBuf {
    type Target = FriendlyByteBuf;

    fn deref(&self) -> &FriendlyByteBuf {
        &self.source
    }
}

impl DerefMut for RegistryFriendlyByteBuf {
    fn deref_mut(&mut self) -> &mut FriendlyByteBuf {
        &mut self.source
    }
}

impl AsMut<ByteBuf> for RegistryFriendlyByteBuf {
    fn as_mut(&mut self) -> &mut ByteBuf {
        self.source.as_mut()
    }
}

impl AsMut<FriendlyByteBuf> for RegistryFriendlyByteBuf {
    fn as_mut(&mut self) -> &mut FriendlyByteBuf {
        &mut self.source
    }
}

impl AsMut<RegistryFriendlyByteBuf> for RegistryFriendlyByteBuf {
    fn as_mut(&mut self) -> &mut RegistryFriendlyByteBuf {
        self
    }
}

/// `unwrap()`, for `StreamCodec::map_stream`.
impl From<RegistryFriendlyByteBuf> for ByteBuf {
    fn from(buffer: RegistryFriendlyByteBuf) -> ByteBuf {
        buffer.source.into()
    }
}
