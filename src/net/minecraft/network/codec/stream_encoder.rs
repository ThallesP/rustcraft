//! net/minecraft/network/codec/StreamEncoder.java

/// Writes a `T` to a buffer. A functional interface, so any `Fn(&mut O, &T)` is one. `T` can be
/// a trait object, like Java's interface types (`CustomQueryAnswerPayload`).
pub trait StreamEncoder<O, T: ?Sized> {
    fn encode(&self, output: &mut O, value: &T);
}

impl<O, T: ?Sized, F: Fn(&mut O, &T)> StreamEncoder<O, T> for F {
    fn encode(&self, output: &mut O, value: &T) {
        self(output, value)
    }
}
