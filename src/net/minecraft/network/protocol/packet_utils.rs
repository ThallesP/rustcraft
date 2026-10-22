//! net/minecraft/network/protocol/PacketUtils.java
//!
//! Not ported yet: the overload taking a `ServerLevel`, and the crash report helpers
//! (`makeReportedException`, `fillCrashReport`), which come with crash reports.
//!
//! Java's handler passes `this`; a Rust handler has `&mut self`, so the listeners keep a weak
//! reference to themselves for it (`this`), which is what the queue holds on to.

use std::cell::RefCell;
use std::rc::Weak;

use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::packet_processor::PacketProcessor;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::server::running_on_different_thread_exception::RunningOnDifferentThreadException;

pub struct PacketUtils;

impl PacketUtils {
    /// Off the game thread, queues a copy of the packet for the listener on it and returns
    /// `RunningOnDifferentThreadException`, so the handler stops there. On the game thread, lets
    /// the handler go on.
    pub fn ensure_running_on_same_thread<T, P>(
        packet: &P,
        listener: &Weak<RefCell<T>>,
        packet_processor: &PacketProcessor,
    ) -> Result<(), RunningOnDifferentThreadException>
    where
        T: PacketListener + ?Sized + 'static,
        P: Packet<T> + Clone + 'static,
    {
        if !packet_processor.is_same_thread() {
            let listener = listener
                .upgrade()
                .expect("a listener handling a packet is alive");
            packet_processor.schedule_if_possible(listener, packet.clone());
            return Err(RunningOnDifferentThreadException::RUNNING_ON_DIFFERENT_THREAD);
        }

        Ok(())
    }
}
