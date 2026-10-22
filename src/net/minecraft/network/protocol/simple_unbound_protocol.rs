//! net/minecraft/network/protocol/SimpleUnboundProtocol.java

use std::sync::{Arc, LazyLock};

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::protocol_info::{DetailsProvider, ProtocolInfo};

/// Java's `Function<ByteBuf, B>` that wraps the connection's buffer in the one a protocol's
/// codecs read (`FriendlyByteBuf::new`). Each packet's codec holds it.
pub type ContextWrapper<B> = Arc<dyn Fn(ByteBuf) -> B + Send + Sync>;

/// A template held in a static field and made on first use: Java's
/// `static final SimpleUnboundProtocol<T, B>`.
pub type StaticSimpleUnboundProtocol<T, B> = LazyLock<Box<dyn SimpleUnboundProtocol<T, B>>>;

/// A protocol's packets, before the buffer its codecs read is known.
pub trait SimpleUnboundProtocol<T: ?Sized, B>: DetailsProvider + Send + Sync {
    fn bind(&self, context_wrapper: ContextWrapper<B>) -> Box<dyn ProtocolInfo<T>>;
}
