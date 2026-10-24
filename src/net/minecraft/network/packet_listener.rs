//! net/minecraft/network/PacketListener.java
//!
//! Not ported yet: `flow()` and `protocol()`, which only `Connection.validateListener` calls, to
//! catch a programming mistake (a listener set up for the wrong side or protocol), and
//! `fillCrashReport`, which comes with crash reports. Its subinterfaces' defaults for `flow` and
//! `protocol` come with them.
//!
//! `onPacketError`'s default rethrows the error as a crash report, which is a panic here, and its
//! `Exception` is the panic's message. `shouldHandleMessage` takes the packet as `Any`, which
//! the listeners that look at it (`ClientCommonPacketListenerImpl`) downcast, Java's `instanceof`.
//! Java's `instanceof TickablePacketListener` is `as_tickable`.

use std::any::Any;

use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::tickable_packet_listener::TickablePacketListener;

/// What a connection hands each packet it receives to: a packet's `handle` calls the listener's
/// handler for it. Each protocol has a listener of its own for each side.
pub trait PacketListener {
    fn on_disconnect(&mut self, details: &DisconnectionDetails);

    /// A packet's handler failed on the main thread.
    fn on_packet_error(&mut self, packet: &dyn Any, cause: &str) {
        let _ = packet;
        panic!("Main thread packet handler: {cause}");
    }

    /// The details of a disconnection the connection's own error caused.
    fn create_disconnection_info(
        &mut self,
        reason: Component,
        cause: &str,
    ) -> DisconnectionDetails {
        let _ = cause;
        DisconnectionDetails::new(reason)
    }

    fn is_accepting_messages(&self) -> bool;

    fn should_handle_message(&self, packet: &dyn Any) -> bool {
        let _ = packet;
        self.is_accepting_messages()
    }

    /// This listener, if it's a `TickablePacketListener`.
    fn as_tickable(&mut self) -> Option<&mut dyn TickablePacketListener> {
        None
    }
}

#[cfg(test)]
pub mod tests {
    use serde_json::Value;

    use super::PacketListener;
    use crate::golden::{Decode, Encode};
    use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;

    /// A listener of any kind that records the handlers called on it, like vanilla's cases: none
    /// before, and each call `[method, packet]` after.
    #[derive(Default)]
    pub struct Recorder {
        calls: Vec<Value>,
    }

    impl Recorder {
        pub fn record(&mut self, method: &str, packet: &impl Encode) {
            self.calls
                .push(Value::from(vec![Value::from(method), packet.encode()]));
        }
    }

    impl Decode for Recorder {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "a listener starts with no calls, got {value}"
            );
            Recorder::default()
        }
    }

    impl Encode for Recorder {
        fn encode(&self) -> Value {
            Value::from(self.calls.clone())
        }
    }

    impl PacketListener for Recorder {
        fn on_disconnect(&mut self, _details: &DisconnectionDetails) {
            panic!("the golden cases never disconnect");
        }

        fn is_accepting_messages(&self) -> bool {
            true
        }
    }
}
