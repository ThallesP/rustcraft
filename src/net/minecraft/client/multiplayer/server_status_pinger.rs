//! net/minecraft/client/multiplayer/ServerStatusPinger.java
//!
//! Not ported yet: what a status response fills in that's text components or the status's parts
//! (the MOTD sanitized with `ResolutionContext`, the version, the player count and sample, the
//! icon), which `ServerStatus` holds as its JSON until text components come, so the response is
//! kept whole in `ServerData::server_status`, and with them `onPersistentDataChange`, which Java
//! calls when the icon changed; and the legacy ping (`pingLegacyServer`,
//! `LegacyServerPinger`), which Java falls back to when a server doesn't answer the status
//! protocol, for servers from before 1.7.
//!
//! The listener is an anonymous class in Java, which captures the connection, the server's data
//! and the callbacks; here it's `StatusListener`, holding them. Its handlers run on Netty's
//! thread, like Java's, in its half there (`StatusNettyHalf`, PORTING.md "The connection's
//! threads"), so the server's data is shared with it (`Arc<Mutex<ServerData>>`), and so is the
//! callback; the game thread's half hears the disconnection.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::net::minecraft::client::gui::screens::connect_screen::ConnectScreen;
use crate::net::minecraft::client::multiplayer::resolver::resolved_server_address::ResolvedServerAddress;
use crate::net::minecraft::client::multiplayer::resolver::server_address::ServerAddress;
use crate::net::minecraft::client::multiplayer::resolver::server_name_resolver::ServerNameResolver;
use crate::net::minecraft::client::multiplayer::server_data::ServerData;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::clientbound_packet_listener::ClientboundPacketListener;
use crate::net::minecraft::network::connection::{Connection, NettyListener, SplitListener};
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::packet_processor::{GameThreadListener, lock};
use crate::net::minecraft::network::protocol::ping::client_pong_packet_listener::ClientPongPacketListener;
use crate::net::minecraft::network::protocol::ping::clientbound_pong_response_packet::ClientboundPongResponsePacket;
use crate::net::minecraft::network::protocol::ping::serverbound_ping_request_packet::ServerboundPingRequestPacket;
use crate::net::minecraft::network::protocol::status::client_status_packet_listener::ClientStatusPacketListener;
use crate::net::minecraft::network::protocol::status::clientbound_status_response_packet::ClientboundStatusResponsePacket;
use crate::net::minecraft::network::protocol::status::server_status_packet_listener::ServerStatusPacketListener;
use crate::net::minecraft::network::protocol::status::serverbound_status_request_packet::ServerboundStatusRequestPacket;
use crate::net::minecraft::util::util::Util;

/// Pings the servers of the server list: asks each for its status, then for a pong, timing it.
#[derive(Default)]
pub struct ServerStatusPinger {
    connections: RefCell<Vec<Arc<Connection>>>,
}

impl ServerStatusPinger {
    /// `CANT_CONNECT_MESSAGE`'s translation key.
    pub const CANT_CONNECT_MESSAGE: &'static str = "multiplayer.status.cannot_connect";

    pub fn new() -> ServerStatusPinger {
        ServerStatusPinger::default()
    }

    /// Starts pinging the server `data` names. `on_pong_response` runs once the pong came, after
    /// the ping's time is in `data`. Fails like Java's when the connection can't be opened.
    pub fn ping_server(
        &self,
        data: Arc<Mutex<ServerData>>,
        on_pong_response: Box<dyn Fn() + Send>,
    ) -> io::Result<()> {
        let raw_address = ServerAddress::parse_string(Some(&lock(&data).ip));
        let Some(address) = ServerNameResolver::resolve_address(&raw_address)
            .map(|address: ResolvedServerAddress| address.as_inet_socket_address())
        else {
            ServerStatusPinger::on_ping_failed(&ConnectScreen::unknown_host_message(), &data);
            return Ok(());
        };

        let connection = Connection::connect_to_server(address)?;
        self.connections.borrow_mut().push(Arc::clone(&connection));
        lock(&data).server_status = None;
        let listener = Rc::new(RefCell::new(StatusListener {
            connection: Arc::clone(&connection),
            data,
            on_pong_response: Mutex::new(Some(on_pong_response)),
            success: Arc::new(AtomicBool::new(false)),
        }));
        connection.initiate_serverbound_status_connection(
            raw_address.get_host(),
            raw_address.get_port(),
            listener,
        );
        connection.send::<dyn ServerStatusPacketListener>(ServerboundStatusRequestPacket::INSTANCE);
        Ok(())
    }

    fn on_ping_failed(reason: &Component, data: &Mutex<ServerData>) {
        let mut data = lock(data);
        eprintln!("Can't ping {}: {}", data.ip, reason.get_string());
        data.server_status = None;
    }

    pub fn tick(&self) {
        let connections: Vec<Arc<Connection>> = self.connections.borrow().clone();
        let mut closed = Vec::new();
        for connection in connections {
            if connection.is_connected() {
                connection.tick();
            } else {
                closed.push(connection);
            }
        }
        for connection in closed {
            self.connections
                .borrow_mut()
                .retain(|other| !Arc::ptr_eq(other, &connection));
            connection.handle_disconnection();
        }
    }

    pub fn remove_all(&self) {
        let connections: Vec<Arc<Connection>> = self.connections.borrow().clone();
        for connection in connections {
            if connection.is_connected() {
                self.connections
                    .borrow_mut()
                    .retain(|other| !Arc::ptr_eq(other, &connection));
                connection.disconnect(DisconnectionDetails::translatable(
                    "multiplayer.status.cancelled",
                ));
            }
        }
    }

    /// Whether a ping is still going.
    pub fn is_pinging(&self) -> bool {
        !self.connections.borrow().is_empty()
    }
}

/// The anonymous `ClientStatusPacketListener` of `pingServer`: the game thread's half.
struct StatusListener {
    connection: Arc<Connection>,
    data: Arc<Mutex<ServerData>>,
    /// Handed to Netty's half when it's made.
    on_pong_response: Mutex<Option<Box<dyn Fn() + Send>>>,
    success: Arc<AtomicBool>,
}

impl PacketListener for StatusListener {
    fn on_disconnect(&mut self, details: &DisconnectionDetails) {
        if !self.success.load(Ordering::SeqCst) {
            ServerStatusPinger::on_ping_failed(details.reason(), &self.data);
        }
    }

    fn is_accepting_messages(&self) -> bool {
        self.connection.is_connected()
    }
}

impl SplitListener<dyn ClientStatusPacketListener> for StatusListener {
    fn netty_listener(
        &self,
        this: GameThreadListener<StatusListener>,
    ) -> Box<dyn NettyListener<dyn ClientStatusPacketListener>> {
        let _ = this;
        Box::new(StatusNettyHalf {
            connection: Arc::clone(&self.connection),
            data: Arc::clone(&self.data),
            on_pong_response: lock(&self.on_pong_response)
                .take()
                .expect("a ping's listener is set up once"),
            success: Arc::clone(&self.success),
            received_ping: false,
            ping_start: 0,
        })
    }
}

/// What Netty's thread runs of the listener: all of its handlers, like Java's.
struct StatusNettyHalf {
    connection: Arc<Connection>,
    data: Arc<Mutex<ServerData>>,
    on_pong_response: Box<dyn Fn() + Send>,
    success: Arc<AtomicBool>,
    received_ping: bool,
    ping_start: i64,
}

impl NettyListener<dyn ClientStatusPacketListener> for StatusNettyHalf {
    fn listener(&mut self) -> &mut (dyn ClientStatusPacketListener + 'static) {
        self
    }
}

impl PacketListener for StatusNettyHalf {
    /// The game thread's half hears it, which the pinger ticks.
    fn on_disconnect(&mut self, details: &DisconnectionDetails) {
        let _ = details;
    }

    fn is_accepting_messages(&self) -> bool {
        self.connection.is_connected()
    }
}

impl ClientboundPacketListener for StatusNettyHalf {}

impl ClientStatusPacketListener for StatusNettyHalf {
    fn handle_status_response(&mut self, packet: &ClientboundStatusResponsePacket) {
        if self.received_ping {
            self.connection
                .disconnect(DisconnectionDetails::translatable(
                    "multiplayer.status.unrequested",
                ));
        } else {
            self.received_ping = true;
            // Java fills in the MOTD, version, players and icon from the status's parts.
            lock(&self.data).server_status = Some(packet.status().clone());
            self.ping_start = Util::get_millis();
            self.connection.send::<dyn ServerStatusPacketListener>(
                ServerboundPingRequestPacket::new(self.ping_start),
            );
            self.success.store(true, Ordering::SeqCst);
        }
    }
}

impl ClientPongPacketListener for StatusNettyHalf {
    fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket) {
        let _ = packet;
        let then = self.ping_start;
        let now = Util::get_millis();
        lock(&self.data).ping = now - then;
        self.connection
            .disconnect(DisconnectionDetails::translatable(
                "multiplayer.status.finished",
            ));
        (self.on_pong_response)();
    }
}
