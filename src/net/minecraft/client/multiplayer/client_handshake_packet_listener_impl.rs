//! net/minecraft/client/multiplayer/ClientHandshakePacketListenerImpl.java
//!
//! Not ported yet: encryption and online-mode login (`handleHello` sets up the cipher and asks
//! Mojang's session server to let the player join), which the port can't do yet, so it
//! disconnects there; the login disconnect packet, which waits for text components; the status
//! messages the connect screen shows; telemetry; and the transfer state a transferred-to
//! connection starts with. Like Java's, every handler runs on the Netty thread.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::java::util::hash_map::HashMap;
use crate::net::minecraft::client::client_brand_retriever::ClientBrandRetriever;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::client_configuration_packet_listener_impl::ClientConfigurationPacketListenerImpl;
use crate::net::minecraft::client::multiplayer::common_listener_cookie::CommonListenerCookie;
use crate::net::minecraft::client::multiplayer::level_load_tracker::LevelLoadTracker;
use crate::net::minecraft::client::multiplayer::server_data::ServerData;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::network::clientbound_packet_listener::ClientboundPacketListener;
use crate::net::minecraft::network::connection::Connection;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::protocol::common::custom::brand_payload::BrandPayload;
use crate::net::minecraft::network::protocol::common::serverbound_client_information_packet::ServerboundClientInformationPacket;
use crate::net::minecraft::network::protocol::common::serverbound_custom_payload_packet::ServerboundCustomPayloadPacket;
use crate::net::minecraft::network::protocol::configuration::configuration_protocols::ConfigurationProtocols;
use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
use crate::net::minecraft::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener;
use crate::net::minecraft::network::protocol::cookie::clientbound_cookie_request_packet::ClientboundCookieRequestPacket;
use crate::net::minecraft::network::protocol::cookie::serverbound_cookie_response_packet::ServerboundCookieResponsePacket;
use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
use crate::net::minecraft::network::protocol::login::clientbound_custom_query_packet::ClientboundCustomQueryPacket;
use crate::net::minecraft::network::protocol::login::clientbound_hello_packet::ClientboundHelloPacket;
use crate::net::minecraft::network::protocol::login::clientbound_login_compression_packet::ClientboundLoginCompressionPacket;
use crate::net::minecraft::network::protocol::login::clientbound_login_finished_packet::ClientboundLoginFinishedPacket;
use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
use crate::net::minecraft::network::protocol::login::serverbound_custom_query_answer_packet::ServerboundCustomQueryAnswerPacket;
use crate::net::minecraft::network::protocol::login::serverbound_login_acknowledged_packet::ServerboundLoginAcknowledgedPacket;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::resources::identifier::Identifier;

/// Where the login is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    CONNECTING,
    AUTHORIZING,
    ENCRYPTING,
    JOINING,
}

impl State {
    /// The states a state can be switched to from.
    fn from_states(&self) -> &'static [State] {
        match self {
            State::CONNECTING => &[],
            State::AUTHORIZING => &[State::CONNECTING],
            State::ENCRYPTING => &[State::AUTHORIZING],
            State::JOINING => &[State::ENCRYPTING, State::CONNECTING],
        }
    }
}

/// The client's listener while it logs in.
pub struct ClientHandshakePacketListenerImpl {
    minecraft: Rc<Minecraft>,
    server_data: Option<ServerData>,
    connection: Rc<Connection>,
    level_load_tracker: Option<LevelLoadTracker>,
    cookies: Vec<(Identifier, Arc<Vec<i8>>)>,
    was_transferred_to: bool,
    state: State,
}

impl ClientHandshakePacketListenerImpl {
    pub fn new(
        connection: Rc<Connection>,
        minecraft: Rc<Minecraft>,
        server_data: Option<ServerData>,
        level_load_tracker: LevelLoadTracker,
    ) -> Rc<RefCell<ClientHandshakePacketListenerImpl>> {
        Rc::new(RefCell::new(ClientHandshakePacketListenerImpl {
            minecraft,
            server_data,
            connection,
            level_load_tracker: Some(level_load_tracker),
            cookies: Vec::new(),
            was_transferred_to: false,
            state: State::CONNECTING,
        }))
    }

    /// Panics for a state the login can't get to from where it is, where Java throws
    /// `IllegalStateException`.
    fn switch_state(&mut self, to_state: State) {
        assert!(
            to_state.from_states().contains(&self.state),
            "Tried to switch to {to_state:?} from {:?}, but expected one of {:?}",
            self.state,
            to_state.from_states()
        );
        self.state = to_state;
    }

    fn send(&self, packet: impl Packet<dyn ServerLoginPacketListener>) {
        self.connection
            .send::<dyn ServerLoginPacketListener>(packet);
    }
}

impl PacketListener for ClientHandshakePacketListenerImpl {
    fn on_disconnect(&mut self, details: &DisconnectionDetails) {
        let title = if self.was_transferred_to {
            "connect.transferFailed"
        } else {
            "connect.failed"
        };
        self.minecraft.show_disconnected(title, details);
    }

    fn is_accepting_messages(&self) -> bool {
        self.connection.is_connected()
    }
}

impl ClientboundPacketListener for ClientHandshakePacketListenerImpl {}

impl ClientCookiePacketListener for ClientHandshakePacketListenerImpl {
    fn handle_request_cookie(&mut self, packet: &ClientboundCookieRequestPacket) {
        let payload = self
            .cookies
            .iter()
            .find(|(key, _)| key == packet.key())
            .map(|(_, payload)| Arc::clone(payload));
        self.send(ServerboundCookieResponsePacket::new(
            packet.key().clone(),
            payload,
        ));
    }
}

impl ClientLoginPacketListener for ClientHandshakePacketListenerImpl {
    /// Java sets up encryption here, and authenticates with Mojang's session server when the
    /// server asks to. Neither is ported, so the port can only log in to a server that doesn't
    /// encrypt (offline mode), and disconnects from one that does.
    fn handle_hello(&mut self, packet: &ClientboundHelloPacket) {
        self.switch_state(State::AUTHORIZING);
        let _ = packet;
        self.connection.disconnect(DisconnectionDetails::new(
            "disconnect.loginFailedInfo: encryption isn't ported yet, so only offline-mode \
             servers can be joined",
        ));
    }

    fn handle_login_finished(&mut self, packet: &ClientboundLoginFinishedPacket) {
        self.switch_state(State::JOINING);
        let local_game_profile = packet.game_profile().clone();
        let cookie = CommonListenerCookie {
            level_load_tracker: self
                .level_load_tracker
                .take()
                .expect("a login finishes once"),
            local_game_profile,
            received_registries: Arc::new(RegistryAccess::default()),
            enabled_features: Vec::new(),
            server_brand: None,
            server_data: self.server_data.clone(),
            server_cookies: self.cookies.clone(),
            custom_report_details: HashMap::new(),
        };
        let listener = ClientConfigurationPacketListenerImpl::new(
            Rc::clone(&self.minecraft),
            Rc::clone(&self.connection),
            cookie,
        );
        self.connection
            .setup_inbound_protocol(Box::new(&***ConfigurationProtocols::CLIENTBOUND), listener);
        self.send(ServerboundLoginAcknowledgedPacket::INSTANCE);
        self.connection
            .setup_outbound_protocol(Box::new(&***ConfigurationProtocols::SERVERBOUND));
        self.connection
            .send::<dyn ServerConfigurationPacketListener>(ServerboundCustomPayloadPacket::new(
                Arc::new(BrandPayload::new(
                    ClientBrandRetriever::get_client_mod_name().to_string(),
                )),
            ));
        let information = self.minecraft.options.borrow().build_player_information();
        self.connection
            .send::<dyn ServerConfigurationPacketListener>(
                ServerboundClientInformationPacket::new(information),
            );
    }

    fn handle_compression(&mut self, packet: &ClientboundLoginCompressionPacket) {
        if !self.connection.is_memory_connection() {
            self.connection
                .setup_compression(packet.get_compression_threshold(), false);
        }
    }

    fn handle_custom_query(&mut self, packet: &ClientboundCustomQueryPacket) {
        self.send(ServerboundCustomQueryAnswerPacket::new(
            packet.transaction_id(),
            None,
        ));
    }
}
