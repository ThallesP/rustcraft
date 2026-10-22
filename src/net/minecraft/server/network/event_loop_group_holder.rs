//! net/minecraft/server/network/EventLoopGroupHolder.java
//!
//! Java's holds a Netty event loop group of its kind of channel (NIO, epoll, kqueue, or the
//! in-memory one), whose threads run the connections registered with it. The port's connections
//! run their own event loop (`Connection::run_event_loop`) on the game thread, so the group is
//! the connections registered with it, and the game runs them between its ticks with
//! `run_until`. One kind of TCP channel is enough for that: `remote(allowNativeTransport)` is
//! `remote()`, and the in-memory group comes with the integrated server.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Instant;

use crate::net::minecraft::network::connection::Connection;

thread_local! {
    static REMOTE: &'static EventLoopGroupHolder = Box::leak(Box::new(EventLoopGroupHolder {
        r#type: "NIO",
        connections: RefCell::new(Vec::new()),
    }));
}

/// The connections a kind of channel's event loop runs.
pub struct EventLoopGroupHolder {
    r#type: &'static str,
    connections: RefCell<Vec<Weak<Connection>>>,
}

impl EventLoopGroupHolder {
    /// The group the game's connections to servers go in.
    pub fn remote() -> &'static EventLoopGroupHolder {
        REMOTE.with(|remote| *remote)
    }

    pub fn r#type(&self) -> &'static str {
        self.r#type
    }

    pub fn register(&self, connection: &Rc<Connection>) {
        self.connections
            .borrow_mut()
            .push(Rc::downgrade(connection));
    }

    /// Runs every live connection's event loop until `deadline`, and forgets the closed ones.
    pub fn run_until(&self, deadline: Instant) {
        self.connections
            .borrow_mut()
            .retain(|connection| connection.upgrade().is_some_and(|c| c.is_connected()));
        let connections: Vec<Rc<Connection>> = self
            .connections
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect();
        match connections.as_slice() {
            [] => {
                let now = Instant::now();
                if deadline > now {
                    std::thread::sleep(deadline - now);
                }
            }
            connections => {
                for connection in connections {
                    connection.run_event_loop(deadline);
                }
            }
        }
    }
}
