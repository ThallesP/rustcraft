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
//! and the callbacks; here it's `StatusListener`, holding them. Its handlers run on the Netty
//! thread, like Java's, which is the connection's event loop.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use crate::net::minecraft::client::gui::screens::connect_screen::ConnectScreen;
use crate::net::minecraft::client::multiplayer::resolver::resolved_server_address::ResolvedServerAddress;
use crate::net::minecraft::client::multiplayer::resolver::server_address::ServerAddress;
use crate::net::minecraft::client::multiplayer::resolver::server_name_resolver::ServerNameResolver;
use crate::net::minecraft::client::multiplayer::server_data::ServerData;
use crate::net::minecraft::network::clientbound_packet_listener::ClientboundPacketListener;
use crate::net::minecraft::network::connection::Connection;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_listener::PacketListener;
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
    connections: RefCell<Vec<Rc<Connection>>>,
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
        data: Rc<RefCell<ServerData>>,
        on_pong_response: Box<dyn Fn()>,
    ) -> io::Result<()> {
        let raw_address = ServerAddress::parse_string(Some(&data.borrow().ip));
        let Some(address) = ServerNameResolver::resolve_address(&raw_address)
            .map(|address: ResolvedServerAddress| address.as_inet_socket_address())
        else {
            ServerStatusPinger::on_ping_failed(ConnectScreen::UNKNOWN_HOST_MESSAGE, &data);
            return Ok(());
        };

        let connection = Connection::connect_to_server(address)?;
        self.connections.borrow_mut().push(Rc::clone(&connection));
        data.borrow_mut().server_status = None;
        let listener = Rc::new(RefCell::new(StatusListener {
            connection: Rc::clone(&connection),
            data,
            on_pong_response,
            success: false,
            received_ping: false,
            ping_start: 0,
        }));
        connection.initiate_serverbound_status_connection(
            raw_address.get_host(),
            raw_address.get_port(),
            listener,
        );
        connection.send::<dyn ServerStatusPacketListener>(ServerboundStatusRequestPacket::INSTANCE);
        Ok(())
    }

    fn on_ping_failed(reason: &str, data: &RefCell<ServerData>) {
        let mut data = data.borrow_mut();
        eprintln!("Can't ping {}: {reason}", data.ip);
        data.server_status = None;
    }

    pub fn tick(&self) {
        let connections: Vec<Rc<Connection>> = self.connections.borrow().clone();
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
                .retain(|other| !Rc::ptr_eq(other, &connection));
            connection.handle_disconnection();
        }
    }

    pub fn remove_all(&self) {
        let connections: Vec<Rc<Connection>> = self.connections.borrow().clone();
        for connection in connections {
            if connection.is_connected() {
                self.connections
                    .borrow_mut()
                    .retain(|other| !Rc::ptr_eq(other, &connection));
                connection.disconnect(DisconnectionDetails::new("multiplayer.status.cancelled"));
            }
        }
    }

    /// Whether a ping is still going.
    pub fn is_pinging(&self) -> bool {
        !self.connections.borrow().is_empty()
    }
}

/// The anonymous `ClientStatusPacketListener` of `pingServer`.
struct StatusListener {
    connection: Rc<Connection>,
    data: Rc<RefCell<ServerData>>,
    on_pong_response: Box<dyn Fn()>,
    success: bool,
    received_ping: bool,
    ping_start: i64,
}

impl PacketListener for StatusListener {
    fn on_disconnect(&mut self, details: &DisconnectionDetails) {
        if !self.success {
            ServerStatusPinger::on_ping_failed(details.reason(), &self.data);
        }
    }

    fn is_accepting_messages(&self) -> bool {
        self.connection.is_connected()
    }
}

impl ClientboundPacketListener for StatusListener {}

impl ClientStatusPacketListener for StatusListener {
    fn handle_status_response(&mut self, packet: &ClientboundStatusResponsePacket) {
        if self.received_ping {
            self.connection
                .disconnect(DisconnectionDetails::new("multiplayer.status.unrequested"));
        } else {
            self.received_ping = true;
            // Java fills in the MOTD, version, players and icon from the status's parts.
            self.data.borrow_mut().server_status = Some(packet.status().clone());
            self.ping_start = Util::get_millis();
            self.connection.send::<dyn ServerStatusPacketListener>(
                ServerboundPingRequestPacket::new(self.ping_start),
            );
            self.success = true;
        }
    }
}

impl ClientPongPacketListener for StatusListener {
    fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket) {
        let _ = packet;
        let then = self.ping_start;
        let now = Util::get_millis();
        self.data.borrow_mut().ping = now - then;
        self.connection
            .disconnect(DisconnectionDetails::new("multiplayer.status.finished"));
        (self.on_pong_response)();
    }
}
