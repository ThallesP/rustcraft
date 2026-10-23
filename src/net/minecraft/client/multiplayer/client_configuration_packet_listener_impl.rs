//! net/minecraft/client/multiplayer/ClientConfigurationPacketListenerImpl.java
//!
//! Not ported yet: the code of conduct's screen (`handleCodeOfConduct` asks the player, who
//! hasn't accepted any code, so a headless client never answers), the registries' values and
//! tags, which `RegistryDataCollector` leaves out, the chat state, the dialogs, resources from the
//! known packs, and an integrated server's registries.
//!
//! Its half on Netty's thread (`NettyHalf`, PORTING.md "The connection's threads") runs
//! `handleEnabledFeatures` and `handleResetChat`, like Java's, with the common ones, and queues
//! the other packets for the game thread's.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};

use crate::com::mojang::authlib::game_profile::GameProfile;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::client_common_packet_listener_impl::{
    self, ClientCommonPacketListenerImpl, client_common_handlers, client_common_netty_handlers,
};
use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
use crate::net::minecraft::client::multiplayer::common_listener_cookie::CommonListenerCookie;
use crate::net::minecraft::client::multiplayer::known_packs_manager::KnownPacksManager;
use crate::net::minecraft::client::multiplayer::level_load_tracker::LevelLoadTracker;
use crate::net::minecraft::client::multiplayer::registry_data_collector::RegistryDataCollector;
use crate::net::minecraft::network::connection::{Connection, NettyListener, SplitListener};
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::packet_processor::{GameThreadListener, lock};
use crate::net::minecraft::network::protocol::configuration::client_configuration_packet_listener::ClientConfigurationPacketListener;
use crate::net::minecraft::network::protocol::configuration::clientbound_code_of_conduct_packet::ClientboundCodeOfConductPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_finish_configuration_packet::ClientboundFinishConfigurationPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_registry_data_packet::ClientboundRegistryDataPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_reset_chat_packet::ClientboundResetChatPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_select_known_packs::ClientboundSelectKnownPacks;
use crate::net::minecraft::network::protocol::configuration::clientbound_update_enabled_features_packet::ClientboundUpdateEnabledFeaturesPacket;
use crate::net::minecraft::network::protocol::configuration::server_configuration_packet_listener::ServerConfigurationPacketListener;
use crate::net::minecraft::network::protocol::configuration::serverbound_accept_code_of_conduct_packet::ServerboundAcceptCodeOfConductPacket;
use crate::net::minecraft::network::protocol::configuration::serverbound_finish_configuration_packet::ServerboundFinishConfigurationPacket;
use crate::net::minecraft::network::protocol::configuration::serverbound_select_known_packs::ServerboundSelectKnownPacks;
use crate::net::minecraft::network::protocol::game::game_protocols::{Context, GameProtocols};
use crate::net::minecraft::network::protocol::packet_utils::PacketUtils;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::network::tickable_packet_listener::TickablePacketListener;
use crate::net::minecraft::resources::identifier::Identifier;

/// The client's listener while the server configures it after login.
pub struct ClientConfigurationPacketListenerImpl {
    this: Weak<RefCell<ClientConfigurationPacketListenerImpl>>,
    common: ClientCommonPacketListenerImpl<dyn ServerConfigurationPacketListener>,
    level_load_tracker: Option<LevelLoadTracker>,
    local_game_profile: GameProfile,
    shared: Arc<Shared>,
    registry_data_collector: RegistryDataCollector,
    known_packs: Option<KnownPacksManager>,
    seen_code_of_conduct: bool,
}

impl ClientConfigurationPacketListenerImpl {
    pub const DISCONNECTED_MESSAGE: &'static str = "multiplayer.disconnect.code_of_conduct";

    pub fn new(
        minecraft: Rc<Minecraft>,
        connection: Arc<Connection>,
        cookie: CommonListenerCookie,
    ) -> Rc<RefCell<ClientConfigurationPacketListenerImpl>> {
        let common = ClientCommonPacketListenerImpl::new(minecraft, connection, &cookie);
        Rc::new_cyclic(|this| {
            RefCell::new(ClientConfigurationPacketListenerImpl {
                this: this.clone(),
                common,
                level_load_tracker: Some(cookie.level_load_tracker),
                local_game_profile: cookie.local_game_profile,
                shared: Arc::new(Shared {
                    enabled_features: Mutex::new(cookie.enabled_features),
                }),
                registry_data_collector: RegistryDataCollector::new(),
                known_packs: None,
                seen_code_of_conduct: false,
            })
        })
    }

    fn send(
        &self,
        packet: impl crate::net::minecraft::network::protocol::packet::Packet<
            dyn ServerConfigurationPacketListener,
        >,
    ) {
        self.common.send(packet);
    }
}

/// The fields both of the listener's halves touch, and the handlers Java runs on Netty's thread.
struct Shared {
    /// The flags are their identifiers until `FeatureFlags` comes.
    enabled_features: Mutex<Vec<Identifier>>,
}

impl Shared {
    fn handle_enabled_features(&self, packet: &ClientboundUpdateEnabledFeaturesPacket) {
        *lock(&self.enabled_features) = packet.features().iter().cloned().collect();
    }

    /// The chat isn't ported yet, so there's no state to reset.
    fn handle_reset_chat(&self, packet: &ClientboundResetChatPacket) {
        let _ = packet;
    }
}

/// The context the client binds the game's serverbound protocol to: Java's anonymous one, which
/// lets every modified codec through, since the server checks them.
struct ClientContext;

impl Context for ClientContext {
    fn has_infinite_materials(&self) -> bool {
        true
    }

    fn can_use_command_blocks(&self) -> bool {
        true
    }
}

impl PacketListener for ClientConfigurationPacketListenerImpl {
    fn on_disconnect(&mut self, details: &DisconnectionDetails) {
        self.common.on_disconnect(details);
    }

    fn on_packet_error(&mut self, packet: &dyn std::any::Any, cause: &str) {
        let _ = packet;
        self.common.on_packet_error("in configuration", cause);
    }

    fn is_accepting_messages(&self) -> bool {
        self.common.connection.is_connected()
    }

    fn should_handle_message(&self, packet: &dyn std::any::Any) -> bool {
        self.common.should_handle_message(packet)
    }

    fn as_tickable(&mut self) -> Option<&mut dyn TickablePacketListener> {
        Some(self)
    }
}

impl TickablePacketListener for ClientConfigurationPacketListenerImpl {
    fn tick(&mut self) {
        self.common.send_deferred_packets();
    }
}

client_common_handlers!(ClientConfigurationPacketListenerImpl);

impl ClientConfigurationPacketListener for ClientConfigurationPacketListenerImpl {
    fn handle_code_of_conduct(&mut self, packet: &ClientboundCodeOfConductPacket) {
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            self.common.minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        assert!(
            !self.seen_code_of_conduct,
            "Server sent duplicate Code of Conduct"
        );

        self.seen_code_of_conduct = true;
        let code_of_conduct = packet.code_of_conduct();
        if self
            .common
            .server_data
            .as_ref()
            .is_some_and(|server_data| server_data.has_accepted_code_of_conduct(code_of_conduct))
        {
            self.send(ServerboundAcceptCodeOfConductPacket::INSTANCE);
        } else {
            eprintln!(
                "The server has a code of conduct, which a client with no screens can't show to \
                 accept:\n{code_of_conduct}"
            );
        }
    }

    fn handle_configuration_finished(&mut self, packet: &ClientboundFinishConfigurationPacket) {
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            self.common.minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let registries = Arc::new(self.registry_data_collector.collect_game_registries());
        let cookie = CommonListenerCookie {
            level_load_tracker: self
                .level_load_tracker
                .take()
                .expect("configuration finishes once"),
            local_game_profile: self.local_game_profile.clone(),
            received_registries: Arc::clone(&registries),
            enabled_features: lock(&self.shared.enabled_features).clone(),
            server_brand: self.common.server_brand.clone(),
            server_data: self.common.server_data.clone(),
            server_cookies: self.common.server_cookies.clone(),
            custom_report_details: self.common.custom_report_details.clone(),
        };
        let connection = Arc::clone(&self.common.connection);
        connection.setup_inbound_protocol(
            GameProtocols::CLIENTBOUND_TEMPLATE
                .bind(RegistryFriendlyByteBuf::decorator(Arc::clone(&registries))),
            ClientPacketListener::new(
                Rc::clone(&self.common.minecraft),
                Arc::clone(&connection),
                cookie,
            ),
        );
        connection.send::<dyn ServerConfigurationPacketListener>(
            ServerboundFinishConfigurationPacket::INSTANCE,
        );
        connection.setup_outbound_protocol(GameProtocols::SERVERBOUND_TEMPLATE.bind(
            RegistryFriendlyByteBuf::decorator(registries),
            Box::new(ClientContext),
        ));
    }

    fn handle_registry_data(&mut self, packet: &ClientboundRegistryDataPacket) {
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            self.common.minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.registry_data_collector
            .append_contents(packet.registry(), packet.entries());
    }

    /// On the Netty thread, like Java's (`NettyHalf`): only a bundle's reaches the game thread.
    fn handle_enabled_features(&mut self, packet: &ClientboundUpdateEnabledFeaturesPacket) {
        self.shared.handle_enabled_features(packet);
    }

    fn handle_select_known_packs(&mut self, packet: &ClientboundSelectKnownPacks) {
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            self.common.minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let selected = self
            .known_packs
            .get_or_insert_with(KnownPacksManager::new)
            .try_selecting_packs(packet.known_packs());
        self.send(ServerboundSelectKnownPacks::new(selected));
    }

    /// On the Netty thread, like Java's (`NettyHalf`): only a bundle's reaches the game thread.
    fn handle_reset_chat(&mut self, packet: &ClientboundResetChatPacket) {
        self.shared.handle_reset_chat(packet);
    }
}

impl SplitListener<dyn ClientConfigurationPacketListener>
    for ClientConfigurationPacketListenerImpl
{
    fn netty_listener(
        &self,
        this: GameThreadListener<ClientConfigurationPacketListenerImpl>,
    ) -> Box<dyn NettyListener<dyn ClientConfigurationPacketListener>> {
        Box::new(NettyHalf {
            this,
            common: self.common.netty_half(),
            shared: Arc::clone(&self.shared),
        })
    }
}

/// What Netty's thread runs of the listener: the handlers Java runs there, and for the others,
/// up to `ensureRunningOnSameThread`, which queues the packet for the game thread's half, `this`.
pub struct NettyHalf {
    this: GameThreadListener<ClientConfigurationPacketListenerImpl>,
    common: client_common_packet_listener_impl::NettyHalf<dyn ServerConfigurationPacketListener>,
    shared: Arc<Shared>,
}

impl NettyListener<dyn ClientConfigurationPacketListener> for NettyHalf {
    fn listener(&mut self) -> &mut (dyn ClientConfigurationPacketListener + 'static) {
        self
    }
}

impl PacketListener for NettyHalf {
    /// The game thread's half hears it, which the connection ticks.
    fn on_disconnect(&mut self, details: &DisconnectionDetails) {
        let _ = details;
    }

    fn is_accepting_messages(&self) -> bool {
        self.common.connection.is_connected()
    }

    fn should_handle_message(&self, packet: &dyn std::any::Any) -> bool {
        self.common.should_handle_message(packet)
    }
}

client_common_netty_handlers!(NettyHalf);

impl ClientConfigurationPacketListener for NettyHalf {
    fn handle_code_of_conduct(&mut self, packet: &ClientboundCodeOfConductPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_configuration_finished(&mut self, packet: &ClientboundFinishConfigurationPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_registry_data(&mut self, packet: &ClientboundRegistryDataPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_enabled_features(&mut self, packet: &ClientboundUpdateEnabledFeaturesPacket) {
        self.shared.handle_enabled_features(packet);
    }

    fn handle_select_known_packs(&mut self, packet: &ClientboundSelectKnownPacks) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_reset_chat(&mut self, packet: &ClientboundResetChatPacket) {
        self.shared.handle_reset_chat(packet);
    }
}
