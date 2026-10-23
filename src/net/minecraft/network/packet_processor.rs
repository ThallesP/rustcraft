//! net/minecraft/network/PacketProcessor.java
//!
//! Java's queue is shared with the Netty thread, which adds to it while the game thread takes
//! from it. Here the connection runs Netty's part on the game thread too (`Connection`), so the
//! queue is a `RefCell`, and `is_same_thread` tells the two apart by whether the connection is
//! running its pipeline: a handler called from there is on "the Netty thread", like Java's, and
//! one the game calls from `process_queued_packets` is on the game's. The `Exception` a queued
//! handler throws is a panic, caught and handed to the listener's `on_packet_error` like Java's.
//!
//! It also counts the packets it queues by type (`arrivals`), which Java's doesn't: that's the
//! oracle client's `PacketProcessorMixin`, which a scenario step's `wait_for` waits on in both
//! clients (TESTING.md).

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;

use crate::net::minecraft::network::connection;
use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;

/// The packets whose handlers asked to run on the game thread, in the order they asked.
pub struct PacketProcessor {
    packets_to_be_handled: RefCell<VecDeque<ListenerAndPacket>>,
    closed: Cell<bool>,
    /// How many packets of each type it has queued, by the type's id.
    arrivals: RefCell<HashMap<String, i32>>,
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
            arrivals: RefCell::new(HashMap::new()),
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
        *self
            .arrivals
            .borrow_mut()
            .entry(packet.r#type().id().to_string())
            .or_insert(0) += 1;

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

    /// How many packets of the type (`minecraft:set_health`) it has queued so far: the oracle
    /// client's count, for `wait_for`.
    pub fn arrivals(&self, packet_type: &str) -> i32 {
        self.arrivals
            .borrow()
            .get(packet_type)
            .copied()
            .unwrap_or(0)
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
