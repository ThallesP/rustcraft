//! net/minecraft/network/protocol/BundleDelimiterPacket.java
//!
//! The abstract class only makes its subclasses' `handle` final, so it's that `handle`, which
//! each delimiter's `Packet::handle` calls.

/// A delimiter's `handle`: never called, since the bundle packer takes every delimiter out of the
/// stream before the listener (`PacketBundlePacker`).
pub fn handle() -> ! {
    panic!("AssertionError: This packet should be handled by pipeline");
}
