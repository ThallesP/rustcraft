//! net/minecraft/network/PacketListener.java
//!
//! Not ported yet: its methods (`flow`, `protocol`, `onDisconnect`, `isAcceptingMessages`, the
//! error and crash report ones), which come with `Connection`, the one that calls them. Its
//! subinterfaces' defaults for `flow` and `protocol` come with them.

/// What a connection hands each packet it receives to: a packet's `handle` calls the listener's
/// handler for it. Each protocol has a listener of its own for each side.
pub trait PacketListener {}

#[cfg(test)]
pub mod tests {
    use serde_json::Value;

    use super::PacketListener;
    use crate::golden::{Decode, Encode};

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

    impl PacketListener for Recorder {}
}
