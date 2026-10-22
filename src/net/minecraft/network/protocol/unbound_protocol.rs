//! net/minecraft/network/protocol/UnboundProtocol.java

use std::sync::LazyLock;

use crate::net::minecraft::network::protocol::simple_unbound_protocol::ContextWrapper;
use crate::net::minecraft::network::protocol_info::{DetailsProvider, ProtocolInfo};

/// A template held in a static field and made on first use: Java's
/// `static final UnboundProtocol<T, B, C>`.
pub type StaticUnboundProtocol<T, B, C> = LazyLock<Box<dyn UnboundProtocol<T, B, C>>>;

/// A protocol's packets, before the buffer its codecs read and the context its codec modifiers
/// take are known.
pub trait UnboundProtocol<T: ?Sized, B, C>: DetailsProvider + Send + Sync {
    fn bind(&self, context_wrapper: ContextWrapper<B>, context: C) -> Box<dyn ProtocolInfo<T>>;
}
