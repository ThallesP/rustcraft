//! net/minecraft/server/network/EventLoopGroupHolder.java
//!
//! Java's holds a Netty event loop group of its kind of channel (NIO, epoll, kqueue, or the
//! in-memory one), whose threads run the channels registered with it. The port's connections
//! each run their own event loop (`Connection`), on a thread the group makes, named like the
//! group's (`Netty Client IO #0`). One kind of TCP channel is enough for that:
//! `remote(allowNativeTransport)` is `remote()`, and the in-memory group comes with the
//! integrated server.

use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};

static REMOTE: EventLoopGroupHolder = EventLoopGroupHolder {
    r#type: "NIO",
    name_format: "Netty Client IO",
    threads: AtomicUsize::new(0),
};

/// The threads a kind of channel's event loops run on.
pub struct EventLoopGroupHolder {
    r#type: &'static str,
    name_format: &'static str,
    threads: AtomicUsize,
}

impl EventLoopGroupHolder {
    /// The group the game's connections to servers go in.
    pub fn remote() -> &'static EventLoopGroupHolder {
        &REMOTE
    }

    pub fn r#type(&self) -> &'static str {
        self.r#type
    }

    /// Starts an event loop thread of the group, its thread factory's.
    pub fn new_thread(&self, run: impl FnOnce() + Send + 'static) -> io::Result<JoinHandle<()>> {
        let index = self.threads.fetch_add(1, Ordering::Relaxed);
        thread::Builder::new()
            .name(format!("{} #{index}", self.name_format))
            .spawn(run)
    }
}
