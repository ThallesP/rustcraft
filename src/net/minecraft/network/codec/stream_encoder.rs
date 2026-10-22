//! net/minecraft/network/codec/StreamEncoder.java

/// Writes a `T` to a buffer. A functional interface, so any `Fn(&mut O, &T)` is one.
pub trait StreamEncoder<O, T> {
    fn encode(&self, output: &mut O, value: &T);
}

impl<O, T, F: Fn(&mut O, &T)> StreamEncoder<O, T> for F {
    fn encode(&self, output: &mut O, value: &T) {
        self(output, value)
    }
}
