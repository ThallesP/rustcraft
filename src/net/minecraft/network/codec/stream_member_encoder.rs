//! net/minecraft/network/codec/StreamMemberEncoder.java

/// Writes a `T` to a buffer, the value first: a packet's own `write` method. A functional
/// interface, so any `Fn(&T, &mut O)` is one.
pub trait StreamMemberEncoder<O, T> {
    fn encode(&self, value: &T, output: &mut O);
}

impl<O, T, F: Fn(&T, &mut O)> StreamMemberEncoder<O, T> for F {
    fn encode(&self, value: &T, output: &mut O) {
        self(value, output)
    }
}
