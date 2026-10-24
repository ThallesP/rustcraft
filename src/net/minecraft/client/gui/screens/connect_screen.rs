//! net/minecraft/client/gui/screens/ConnectScreen.java
//!
//! Not ported yet: what the screen draws (its status text, the cancel button), aborting, the
//! downloaded pack source's server control, transfers, and the connector thread: Java resolves
//! and connects on a thread of its own while the screen shows, and the port does it on the
//! game's, before the screen shows. What's here is how a connection to a server starts, and the
//! screen's tick, which ticks the connection until the level's loading screen replaces it.

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::screens::disconnected_screen::DisconnectedScreen;
use crate::net::minecraft::client::gui::screens::screen::{Screen, ScreenClass};
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::client_handshake_packet_listener_impl::ClientHandshakePacketListenerImpl;
use crate::net::minecraft::client::multiplayer::level_load_tracker::LevelLoadTracker;
use crate::net::minecraft::client::multiplayer::resolver::server_address::ServerAddress;
use crate::net::minecraft::client::multiplayer::resolver::server_name_resolver::ServerNameResolver;
use crate::net::minecraft::client::multiplayer::server_data::ServerData;
use crate::net::minecraft::client::resources::server::downloaded_pack_source::DownloadedPackSource;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::contents::translatable_contents::TranslationArgument;
use crate::net::minecraft::network::connection::Connection;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
use crate::net::minecraft::network::protocol::login::serverbound_hello_packet::ServerboundHelloPacket;

/// The screen the game shows while it connects to a server.
pub struct ConnectScreen {
    screen: Screen,
    connection: Option<Arc<Connection>>,
}

impl ConnectScreen {
    /// `UNKNOWN_HOST_MESSAGE`.
    pub fn unknown_host_message() -> Component {
        Component::translatable_with_args(
            "disconnect.genericReason",
            vec![TranslationArgument::Component(
                Component::translatable("disconnect.unknownHost").into(),
            )],
        )
        .into()
    }

    /// Shows the screen and connects to the server.
    pub fn start_connecting(
        minecraft: &Rc<Minecraft>,
        host_and_port: &ServerAddress,
        data: ServerData,
    ) {
        let mut screen = ConnectScreen {
            screen: Screen::new(
                minecraft,
                Component::translatable("connect.connecting").into(),
            ),
            connection: None,
        };
        screen.connect(minecraft, host_and_port, data);
        // A connection that failed at once has shown the disconnected screen already.
        let failed = minecraft
            .screen()
            .is_some_and(|screen| screen.borrow().as_any().is::<DisconnectedScreen>());
        if !failed {
            minecraft.set_screen(Some(Rc::new(RefCell::new(screen))));
        }
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
                &DisconnectionDetails::new(ConnectScreen::unknown_host_message()),
            );
            return;
        };

        let connection = match Connection::connect_to_server(address.as_inet_socket_address()) {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("Couldn't connect to server: {error}");
                minecraft.show_disconnected(
                    "connect.failed",
                    &DisconnectionDetails::generic_reason(&error.to_string()),
                );
                return;
            }
        };
        self.connection = Some(Arc::clone(&connection));
        minecraft
            .downloaded_pack_source
            .borrow_mut()
            .configure_for_server_control(DownloadedPackSource::convert_pack_status(
                server.get_resource_pack_status(),
            ));

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
}

impl ScreenClass for ConnectScreen {
    fn screen(&self) -> &Screen {
        &self.screen
    }

    fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    /// `tick`: the connection's, until it ends.
    fn tick(&mut self) {
        if let Some(connection) = &self.connection {
            if connection.is_connected() {
                connection.tick();
            } else {
                connection.handle_disconnection();
            }
        }
    }

    fn should_close_on_esc(&self) -> bool {
        false
    }

    /// What it draws isn't ported.
    fn extract_background(
        &mut self,
        _graphics: &mut GuiGraphicsExtractor,
        _x: i32,
        _y: i32,
        _a: f32,
    ) {
    }
}
