//! net/minecraft/network/protocol/PacketUtils.java
//!
//! Not ported yet: the overload taking a `ServerLevel`, and the crash report helpers
//! (`makeReportedException`, `fillCrashReport`), which come with crash reports.
//!
//! Java's handler passes `this`. A Rust listener is two halves (PORTING.md, "The connection's
//! threads"): the game thread's passes its weak reference to itself, and Netty's half the name
//! it has for the game thread's (`ListenerRef`), which is what the queue holds on to.

use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::packet_processor::{ListenerRef, PacketProcessor};
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::server::running_on_different_thread_exception::RunningOnDifferentThreadException;

pub struct PacketUtils;

impl PacketUtils {
    /// Off the game thread, queues a copy of the packet for the listener on it and returns
    /// `RunningOnDifferentThreadException`, so the handler stops there. On the game thread, lets
    /// the handler go on.
    pub fn ensure_running_on_same_thread<T, P>(
        packet: &P,
        listener: &impl ListenerRef<T>,
        packet_processor: &PacketProcessor,
    ) -> Result<(), RunningOnDifferentThreadException>
    where
        T: PacketListener + ?Sized + 'static,
        P: Packet<T> + Clone + 'static,
    {
        if !packet_processor.is_same_thread() {
            packet_processor.schedule_if_possible(listener.game_thread_listener(), packet.clone());
            return Err(RunningOnDifferentThreadException::RUNNING_ON_DIFFERENT_THREAD);
        }

        Ok(())
    }
}
