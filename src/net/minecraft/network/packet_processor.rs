//! net/minecraft/network/PacketProcessor.java
//!
//! Java's queue is shared with the Netty thread, which adds to it while the game thread takes
//! from it, and so is this one (a `Mutex`). `is_same_thread` asks whether the code running is on
//! the thread that made the processor, the game's, like Java's. The `Exception` a queued handler
//! throws is a panic, caught and handed to the listener's `on_packet_error` like Java's.
//!
//! Java's queue holds each packet's listener itself, which any thread can reach. A Rust listener
//! the game thread handles packets with is the game thread's (an `Rc`), so other threads name it
//! with a `GameThreadListener`, which the game thread looks up when it handles the packet.
//!
//! It also counts the packets it queues by type (`arrivals`), and keeps when the last of each came
//! (`last_arrival`), which Java's doesn't: that's the oracle client's `PacketProcessorMixin` and
//! `PacketArrivals`, which a scenario step's `wait_for` waits on in both clients, and a paced
//! scenario takes the server's ticks from (TESTING.md).

use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::panic::{self, AssertUnwindSafe};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, ThreadId};
use std::time::Instant;

use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::protocol::packet::Packet;

/// The packets whose handlers asked to run on the game thread, in the order they asked.
pub struct PacketProcessor {
    packets_to_be_handled: Mutex<VecDeque<ListenerAndPacket>>,
    running_thread: ThreadId,
    closed: AtomicBool,
    /// How many packets of each type it has queued, by the type's id.
    arrivals: Mutex<HashMap<String, i32>>,
    /// When it queued the last of each type.
    last_arrivals: Mutex<HashMap<String, Instant>>,
}

impl PacketProcessor {
    /// Java's takes the thread it runs on, which is the one that makes it (`Minecraft`'s).
    pub fn new() -> PacketProcessor {
        PacketProcessor {
            packets_to_be_handled: Mutex::new(VecDeque::new()),
            running_thread: thread::current().id(),
            closed: AtomicBool::new(false),
            arrivals: Mutex::new(HashMap::new()),
            last_arrivals: Mutex::new(HashMap::new()),
        }
    }

    pub fn is_same_thread(&self) -> bool {
        thread::current().id() == self.running_thread
    }

    /// Queues the packet for its listener. Panics once closed, where Java throws
    /// `RejectedExecutionException`.
    pub fn schedule_if_possible<T, P>(&self, listener: GameThreadListener<T>, packet: P)
    where
        T: PacketListener + ?Sized + 'static,
        P: Packet<T> + 'static,
    {
        assert!(
            !self.closed.load(Ordering::Relaxed),
            "RejectedExecutionException: Server already shutting down"
        );
        let packet_type = packet.r#type().id().to_string();
        lock(&self.last_arrivals).insert(packet_type.clone(), Instant::now());
        *lock(&self.arrivals).entry(packet_type).or_insert(0) += 1;

        lock(&self.packets_to_be_handled).push_back(ListenerAndPacket {
            handle: Box::new(move || {
                let handled =
                    listener.with(|listener| ListenerAndPacket::handle(listener, &packet));
                if handled.is_none() {
                    eprintln!("Ignoring packet for a listener that's gone: {packet:?}");
                }
            }),
        });
    }

    pub fn process_queued_packets(&self) {
        if !self.closed.load(Ordering::Relaxed) {
            loop {
                // Taken out first, since a handler can queue another.
                let next = lock(&self.packets_to_be_handled).pop_front();
                let Some(next) = next else {
                    break;
                };
                (next.handle)();
            }
        }
        game_thread_listener::forget_unnamed();
    }

    /// How many packets of the type (`minecraft:set_health`) it has queued so far: the oracle
    /// client's count, for `wait_for`.
    pub fn arrivals(&self, packet_type: &str) -> i32 {
        lock(&self.arrivals).get(packet_type).copied().unwrap_or(0)
    }

    /// When it queued the last packet of the type, if one has come: `PacketArrivals.lastNanos`.
    pub fn last_arrival(&self, packet_type: &str) -> Option<Instant> {
        lock(&self.last_arrivals).get(packet_type).copied()
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);
    }
}

impl Default for PacketProcessor {
    fn default() -> PacketProcessor {
        PacketProcessor::new()
    }
}

/// `PacketProcessor.ListenerAndPacket`: a packet, and the listener to hand it to.
struct ListenerAndPacket {
    handle: Box<dyn FnOnce() + Send>,
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

/// A listener the game thread owns, named so that any thread can hand it a packet: what Java's
/// reference to the listener is to the Netty thread. Only the game thread reaches the listener
/// (`with`), which lives at least as long as a name for it does.
pub struct GameThreadListener<T: ?Sized> {
    name: Arc<u64>,
    listener: PhantomData<fn() -> Box<T>>,
}

impl<T: ?Sized> Clone for GameThreadListener<T> {
    fn clone(&self) -> GameThreadListener<T> {
        GameThreadListener {
            name: Arc::clone(&self.name),
            listener: PhantomData,
        }
    }
}

impl<T: ?Sized + 'static> GameThreadListener<T> {
    /// Names the listener, on the game thread.
    pub fn new(listener: &Rc<RefCell<T>>) -> GameThreadListener<T> {
        game_thread_listener::forget_unnamed();
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let name = Arc::new(NEXT.fetch_add(1, Ordering::Relaxed));
        game_thread_listener::LISTENERS.with(|listeners| {
            listeners.borrow_mut().insert(
                *name,
                (Arc::downgrade(&name), Box::new(Rc::clone(listener))),
            );
        });
        GameThreadListener {
            name,
            listener: PhantomData,
        }
    }

    /// Calls `f` with the listener, on the game thread; `None` anywhere else.
    pub fn with<R>(&self, f: impl FnOnce(&Rc<RefCell<T>>) -> R) -> Option<R> {
        let listener = game_thread_listener::LISTENERS.with(|listeners| {
            listeners
                .borrow()
                .get(&*self.name)
                .and_then(|(_, listener)| listener.downcast_ref::<Rc<RefCell<T>>>().cloned())
        })?;
        Some(f(&listener))
    }
}

/// What `PacketUtils.ensureRunningOnSameThread` queues a packet for: Java's listener, which a
/// handler passes as `this`. A listener on the game thread passes its weak reference to itself,
/// and the half Netty's thread runs (`NettyListener`) the name it has for its game thread's.
pub trait ListenerRef<T: ?Sized> {
    fn game_thread_listener(&self) -> GameThreadListener<T>;
}

impl<T: ?Sized + 'static> ListenerRef<T> for GameThreadListener<T> {
    fn game_thread_listener(&self) -> GameThreadListener<T> {
        self.clone()
    }
}

impl<T: ?Sized + 'static> ListenerRef<T> for Weak<RefCell<T>> {
    fn game_thread_listener(&self) -> GameThreadListener<T> {
        GameThreadListener::new(
            &self
                .upgrade()
                .expect("a listener handling a packet is alive"),
        )
    }
}

mod game_thread_listener {
    use std::any::Any;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::Weak;

    /// A name's weak reference to itself, and the listener it names, an `Rc<RefCell<T>>`.
    type Named = (Weak<u64>, Box<dyn Any>);

    thread_local! {
        /// The listeners named so far on this thread, the game's.
        pub static LISTENERS: RefCell<HashMap<u64, Named>> = RefCell::new(HashMap::new());
    }

    /// Lets go of the listeners nothing names any more, on the game thread, which their `Rc`s
    /// are.
    pub fn forget_unnamed() {
        let forgotten: Vec<Box<dyn Any>> = LISTENERS.with(|listeners| {
            let mut listeners = listeners.borrow_mut();
            let unnamed: Vec<u64> = listeners
                .iter()
                .filter(|(_, (name, _))| name.strong_count() == 0)
                .map(|(name, _)| *name)
                .collect();
            unnamed
                .into_iter()
                .filter_map(|name| listeners.remove(&name))
                .map(|(_, listener)| listener)
                .collect()
        });
        // Dropped once the registry isn't borrowed: a listener's drop can name another.
        drop(forgotten);
    }
}

/// A mutex's guard, even when a thread panicked holding it: what the connection's threads share
/// is left as the panic left it, like Java's objects after an exception.
pub fn lock<T: ?Sized>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
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
