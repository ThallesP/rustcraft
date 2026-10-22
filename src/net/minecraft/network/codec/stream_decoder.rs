//! net/minecraft/network/codec/StreamDecoder.java

/// Reads a `T` from a buffer. A functional interface, so any `Fn(&mut I) -> T` is one, like
/// Java's lambdas and method references (`FriendlyByteBuf::read_identifier`).
pub trait StreamDecoder<I, T> {
    fn decode(&self, input: &mut I) -> T;
}

impl<I, T, F: Fn(&mut I) -> T> StreamDecoder<I, T> for F {
    fn decode(&self, input: &mut I) -> T {
        self(input)
    }
}
