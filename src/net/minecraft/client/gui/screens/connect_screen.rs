//! net/minecraft/client/gui/screens/ConnectScreen.java
//!
//! Not ported yet: the screen itself (its status text, the cancel button), aborting, the
//! downloaded pack source's server control, transfers, and the connector thread: Java resolves
//! and connects on a thread of its own while the screen shows, and the port does it on the
//! game's, before the first tick, since there's nothing to draw meanwhile. What's left is how a
//! connection to a server starts, and the screen's tick, which ticks the connection until the
//! level's loading screen replaces it.

use std::rc::Rc;
use std::sync::Arc;

use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::client_handshake_packet_listener_impl::ClientHandshakePacketListenerImpl;
use crate::net::minecraft::client::multiplayer::level_load_tracker::LevelLoadTracker;
use crate::net::minecraft::client::multiplayer::resolver::server_address::ServerAddress;
use crate::net::minecraft::client::multiplayer::resolver::server_name_resolver::ServerNameResolver;
use crate::net::minecraft::client::multiplayer::server_data::ServerData;
use crate::net::minecraft::network::connection::Connection;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
use crate::net::minecraft::network::protocol::login::serverbound_hello_packet::ServerboundHelloPacket;

/// The screen the game shows while it connects to a server.
#[derive(Clone)]
pub struct ConnectScreen {
    connection: Option<Arc<Connection>>,
}

impl ConnectScreen {
    pub const UNKNOWN_HOST_MESSAGE: &'static str = "disconnect.genericReason: Unknown host";

    /// Shows the screen and connects to the server.
    pub fn start_connecting(
        minecraft: &Rc<Minecraft>,
        host_and_port: &ServerAddress,
        data: ServerData,
    ) {
        let mut screen = ConnectScreen { connection: None };
        screen.connect(minecraft, host_and_port, data);
        minecraft.set_connect_screen(screen);
    }

    fn connect(
        &mut self,
        minecraft: &Rc<Minecraft>,
        host_and_port: &ServerAddress,
        server: ServerData,
    ) {
        println!(
            "Connecting to {}, {}",
            host_and_port.get_host(),
            host_and_port.get_port()
        );
        let Some(address) = ServerNameResolver::resolve_address(host_and_port) else {
            minecraft.show_disconnected(
                "connect.failed",
                &DisconnectionDetails::new(ConnectScreen::UNKNOWN_HOST_MESSAGE),
            );
            return;
        };

        let connection = match Connection::connect_to_server(address.as_inet_socket_address()) {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("Couldn't connect to server: {error}");
                minecraft.show_disconnected(
                    "connect.failed",
                    &DisconnectionDetails::new(format!("disconnect.genericReason: {error}")),
                );
                return;
            }
        };
        self.connection = Some(Arc::clone(&connection));

        let listener = ClientHandshakePacketListenerImpl::new(
            Arc::clone(&connection),
            Rc::clone(minecraft),
            Some(server),
            LevelLoadTracker::new(),
        );
        connection.initiate_serverbound_play_connection(
            &address.get_host_name(),
            address.get_port(),
            listener,
        );
        let user = minecraft.get_user();
        connection.send::<dyn ServerLoginPacketListener>(ServerboundHelloPacket::new(
            user.get_name().to_string(),
            user.get_profile_id(),
        ));
    }

    pub fn tick(&self) {
        if let Some(connection) = &self.connection {
            if connection.is_connected() {
                connection.tick();
            } else {
                connection.handle_disconnection();
            }
        }
    }
}
