//! net/minecraft/network/PacketProcessor.java
//!
//! Java's queue is shared with the Netty thread, which adds to it while the game thread takes
//! from it. Here the connection runs Netty's part on the game thread too (`Connection`), so the
//! queue is a `RefCell`, and `is_same_thread` tells the two apart by whether the connection is
//! running its pipeline: a handler called from there is on "the Netty thread", like Java's, and
//! one the game calls from `process_queued_packets` is on the game's. The `Exception` a queued
//! handler throws is a panic, caught and handed to the listener's `on_packet_error` like Java's.

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;

use crate::net::minecraft::network::connection;
use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;

/// The packets whose handlers asked to run on the game thread, in the order they asked.
pub struct PacketProcessor {
    packets_to_be_handled: RefCell<VecDeque<ListenerAndPacket>>,
    closed: Cell<bool>,
}

impl Default for PacketProcessor {
    fn default() -> PacketProcessor {
        PacketProcessor::new()
    }
}

impl PacketProcessor {
    /// Java's takes the thread it runs on, which is the one that makes it (`Minecraft`'s).
    pub fn new() -> PacketProcessor {
        PacketProcessor {
            packets_to_be_handled: RefCell::new(VecDeque::new()),
            closed: Cell::new(false),
        }
    }

    pub fn is_same_thread(&self) -> bool {
        !connection::in_event_loop()
    }

    /// Queues the packet for its listener. Panics once closed, where Java throws
    /// `RejectedExecutionException`.
    pub fn schedule_if_possible<T, P>(&self, listener: Rc<RefCell<T>>, packet: P)
    where
        T: PacketListener + ?Sized + 'static,
        P: Packet<T> + 'static,
    {
        assert!(
            !self.closed.get(),
            "RejectedExecutionException: Server already shutting down"
        );

        self.packets_to_be_handled
            .borrow_mut()
            .push_back(ListenerAndPacket {
                handle: Box::new(move || ListenerAndPacket::handle(&listener, &packet)),
            });
    }

    pub fn process_queued_packets(&self) {
        if !self.closed.get() {
            loop {
                // Taken out first, since a handler can queue another.
                let next = self.packets_to_be_handled.borrow_mut().pop_front();
                let Some(next) = next else {
                    break;
                };
                (next.handle)();
            }
        }
    }

    pub fn close(&self) {
        self.closed.set(true);
    }
}

/// `PacketProcessor.ListenerAndPacket`: a packet, and the listener to hand it to.
struct ListenerAndPacket {
    handle: Box<dyn FnOnce()>,
}

impl ListenerAndPacket {
    fn handle<T, P>(listener: &Rc<RefCell<T>>, packet: &P)
    where
        T: PacketListener + ?Sized,
        P: Packet<T>,
    {
        if listener.borrow().should_handle_message(packet as &dyn Any) {
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                packet.handle(&mut *listener.borrow_mut());
            }));
            if let Err(cause) = result {
                listener
                    .borrow_mut()
                    .on_packet_error(packet as &dyn Any, &panic_message(&cause));
            }
        } else {
            eprintln!("Ignoring packet due to disconnection: {packet:?}");
        }
    }
}

/// What a caught panic said, for the logs.
pub fn panic_message(cause: &(dyn Any + Send)) -> String {
    if let Some(message) = cause.downcast_ref::<&str>() {
        message.to_string()
    } else if let Some(message) = cause.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown error".to_string()
    }
}
