//! net/minecraft/client/multiplayer/ClientCommonPacketListenerImpl.java
//!
//! Not ported yet: the resource packs (`handleResourcePackPush`, which waits for its packet, and
//! the downloaded pack source that `handleResourcePackPop` pops from), the post effects the player
//! gets (`handlePostEffects`, which comes with the renderer), dialogs (`handleShowDialog`,
//! `clearDialog`), server links, transfers (`handleTransfer`, which disconnects and connects to
//! the other server: the port disconnects), the disconnection report written about a packet
//! error, telemetry, and the disconnected screen, which the headless client is
//! (`Minecraft::disconnect`).
//!
//! Java's abstract class is a struct each client listener holds (the configuration one and the
//! game one), and derefs to (PORTING.md). Its listener interface's handlers are the concrete
//! listeners' own, which `client_common_handlers!` writes to forward here with a reference to the
//! listener itself, the `this` that `PacketUtils.ensureRunningOnSameThread` queues. It's generic
//! in the server's listener `S` of the protocol it sends in, which a packet it sends is for.
//!
//! Its half on Netty's thread (`NettyHalf`, PORTING.md "The connection's threads") is what the
//! concrete listeners' halves hold, and `client_common_netty_handlers!` writes their handlers the
//! same way: `handleKeepAlive`, `handleDisconnect` and a discarded payload's `handleCustomPayload`
//! run there, like Java's, and the others queue their packet for the game thread. What both
//! halves touch (the deferred packets, the transfer flag) is `Shared`.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::java::util::hash_map::HashMap;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::common_listener_cookie::CommonListenerCookie;
use crate::net::minecraft::client::multiplayer::server_data::ServerData;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::connection::Connection;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_processor::{GameThreadListener, PacketProcessor, lock};
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::clientbound_clear_dialog_packet::ClientboundClearDialogPacket;
use crate::net::minecraft::network::protocol::common::clientbound_custom_payload_packet::ClientboundCustomPayloadPacket;
use crate::net::minecraft::network::protocol::common::clientbound_custom_report_details_packet::ClientboundCustomReportDetailsPacket;
use crate::net::minecraft::network::protocol::common::clientbound_disconnect_packet::ClientboundDisconnectPacket;
use crate::net::minecraft::network::protocol::common::clientbound_keep_alive_packet::ClientboundKeepAlivePacket;
use crate::net::minecraft::network::protocol::common::clientbound_ping_packet::ClientboundPingPacket;
use crate::net::minecraft::network::protocol::common::clientbound_post_effects_packet::ClientboundPostEffectsPacket;
use crate::net::minecraft::network::protocol::common::clientbound_resource_pack_pop_packet::ClientboundResourcePackPopPacket;
use crate::net::minecraft::network::protocol::common::clientbound_store_cookie_packet::ClientboundStoreCookiePacket;
use crate::net::minecraft::network::protocol::common::clientbound_transfer_packet::ClientboundTransferPacket;
use crate::net::minecraft::network::protocol::common::custom::brand_payload::BrandPayload;
use crate::net::minecraft::network::protocol::common::custom::custom_packet_payload::CustomPacketPayload;
use crate::net::minecraft::network::protocol::common::custom::discarded_payload::DiscardedPayload;
use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use crate::net::minecraft::network::protocol::common::serverbound_keep_alive_packet::ServerboundKeepAlivePacket;
use crate::net::minecraft::network::protocol::common::serverbound_pong_packet::ServerboundPongPacket;
use crate::net::minecraft::network::protocol::cookie::clientbound_cookie_request_packet::ClientboundCookieRequestPacket;
use crate::net::minecraft::network::protocol::cookie::serverbound_cookie_response_packet::ServerboundCookieResponsePacket;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_utils::PacketUtils;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::util::Util;

/// A packet the client sends once a condition holds, or drops when it expires first.
struct DeferredPacket<S: ?Sized> {
    packet: Box<dyn Packet<S>>,
    send_condition: Box<dyn Fn() -> bool + Send>,
    expiration_time: i64,
}

/// What the client's listeners after login share.
pub struct ClientCommonPacketListenerImpl<S: ?Sized> {
    pub minecraft: Rc<Minecraft>,
    pub connection: Arc<Connection>,
    pub server_data: Option<ServerData>,
    pub server_brand: Option<String>,
    shared: Arc<Shared<S>>,
    pub server_cookies: Vec<(Identifier, Arc<Vec<i8>>)>,
    pub custom_report_details: HashMap<String, String>,
}

/// The fields both of the listener's halves touch, and the handlers Java runs on Netty's thread,
/// which either half can run: Netty's, or the game thread's for a packet of a bundle.
struct Shared<S: ?Sized> {
    connection: Arc<Connection>,
    is_transferring: AtomicBool,
    deferred_packets: Mutex<Vec<DeferredPacket<S>>>,
}

impl<S: ServerCommonPacketListener + ?Sized + 'static> Shared<S> {
    /// Java's also lets a transferring client take the cookie and transfer packets it's sent
    /// after it stopped accepting messages.
    fn should_handle_message(&self, packet: &dyn std::any::Any) -> bool {
        self.connection.is_connected()
            || self.is_transferring.load(Ordering::SeqCst)
                && (packet.is::<ClientboundStoreCookiePacket>()
                    || packet.is::<ClientboundTransferPacket>())
    }

    /// The answer goes out at once, on the thread handling it.
    fn handle_keep_alive(&self, packet: &ClientboundKeepAlivePacket) {
        self.send_when(
            ServerboundKeepAlivePacket::new(packet.get_id()),
            Box::new(ClientCommonPacketListenerImpl::<S>::not_frozen_at_poll_events),
            60000,
        );
    }

    /// The server's reason, which the connection disconnects with.
    fn handle_disconnect(&self, packet: &ClientboundDisconnectPacket) {
        self.connection
            .disconnect(DisconnectionDetails::new(packet.reason().get_string()));
    }

    fn send_when(
        &self,
        packet: impl Packet<S>,
        condition: Box<dyn Fn() -> bool + Send>,
        expire_after_millis: i64,
    ) {
        if condition() {
            self.connection.send::<S>(packet);
        } else {
            lock(&self.deferred_packets).push(DeferredPacket {
                packet: Box::new(packet),
                send_condition: condition,
                expiration_time: Util::get_millis() + expire_after_millis,
            });
        }
    }
}

impl<S: ServerCommonPacketListener + ?Sized + 'static> ClientCommonPacketListenerImpl<S> {
    const GENERIC_DISCONNECT_MESSAGE: &'static str = "disconnect.lost";

    /// The keep-alive's condition: the game isn't frozen at polling the window's events, which
    /// a client with no window never is (`RenderSystem.isFrozenAtPollEvents`).
    fn not_frozen_at_poll_events() -> bool {
        true
    }

    pub fn new(
        minecraft: Rc<Minecraft>,
        connection: Arc<Connection>,
        cookie: &CommonListenerCookie,
    ) -> ClientCommonPacketListenerImpl<S> {
        ClientCommonPacketListenerImpl {
            minecraft,
            connection: Arc::clone(&connection),
            server_data: cookie.server_data.clone(),
            server_brand: cookie.server_brand.clone(),
            shared: Arc::new(Shared {
                connection,
                is_transferring: AtomicBool::new(false),
                deferred_packets: Mutex::new(Vec::new()),
            }),
            server_cookies: cookie.server_cookies.clone(),
            custom_report_details: cookie.custom_report_details.clone(),
        }
    }

    /// The half of the listener Netty's thread runs.
    pub fn netty_half(&self) -> NettyHalf<S> {
        NettyHalf {
            connection: Arc::clone(&self.connection),
            packet_processor: Arc::clone(self.minecraft.packet_processor()),
            shared: Arc::clone(&self.shared),
        }
    }

    pub fn is_transferring(&self) -> bool {
        self.shared.is_transferring.load(Ordering::SeqCst)
    }

    pub fn on_packet_error(&mut self, packet: &str, cause: &str) {
        eprintln!("Failed to handle packet {packet}, disconnecting: {cause}");
        self.connection
            .disconnect(DisconnectionDetails::new("disconnect.packetError"));
    }

    pub fn should_handle_message(&self, packet: &dyn std::any::Any) -> bool {
        self.shared.should_handle_message(packet)
    }

    /// On the Netty thread, like Java's (`NettyHalf`): only a bundle's reaches the game thread.
    pub fn handle_keep_alive(&mut self, packet: &ClientboundKeepAlivePacket) {
        self.shared.handle_keep_alive(packet);
    }

    pub fn handle_ping<T>(&mut self, this: &Weak<RefCell<T>>, packet: &ClientboundPingPacket)
    where
        T: ClientCommonPacketListener + 'static,
    {
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            this,
            self.minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.send(ServerboundPongPacket::new(packet.get_id()));
    }

    pub fn handle_custom_payload<T>(
        &mut self,
        this: &Weak<RefCell<T>>,
        packet: &ClientboundCustomPayloadPacket,
    ) where
        T: ClientCommonPacketListener + 'static,
    {
        let payload = packet.payload();
        let payload: &dyn std::any::Any = &*payload;
        if !payload.is::<DiscardedPayload>() {
            if PacketUtils::ensure_running_on_same_thread(
                packet,
                this,
                self.minecraft.packet_processor(),
            )
            .is_err()
            {
                return;
            }
            match payload.downcast_ref::<BrandPayload>() {
                Some(brand) => self.server_brand = Some(brand.brand().to_string()),
                None => {
                    let payload = packet.payload();
                    self.handle_unknown_custom_payload(&*payload);
                }
            }
        }
    }

    /// Both listeners' `handleCustomPayload(CustomPacketPayload)` for a payload they don't know,
    /// which is every one so far: the game protocol's debug payloads aren't ported yet.
    fn handle_unknown_custom_payload(&self, payload: &dyn CustomPacketPayload) {
        eprintln!("Unknown custom packet payload: {}", payload.r#type().id());
    }

    pub fn handle_resource_pack_pop<T>(
        &mut self,
        this: &Weak<RefCell<T>>,
        packet: &ClientboundResourcePackPopPacket,
    ) where
        T: ClientCommonPacketListener + 'static,
    {
        let _ = PacketUtils::ensure_running_on_same_thread(
            packet,
            this,
            self.minecraft.packet_processor(),
        );
    }

    pub fn handle_post_effects<T>(
        &mut self,
        this: &Weak<RefCell<T>>,
        packet: &ClientboundPostEffectsPacket,
    ) where
        T: ClientCommonPacketListener + 'static,
    {
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            this,
            self.minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        assert!(
            self.minecraft.player.borrow().is_some(),
            "Cannot set player post effects before the player is created."
        );
    }

    pub fn handle_request_cookie<T>(
        &mut self,
        this: &Weak<RefCell<T>>,
        packet: &ClientboundCookieRequestPacket,
    ) where
        T: ClientCommonPacketListener + 'static,
    {
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            this,
            self.minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let payload = self
            .server_cookies
            .iter()
            .find(|(key, _)| key == packet.key())
            .map(|(_, payload)| Arc::clone(payload));
        self.connection
            .send::<S>(ServerboundCookieResponsePacket::new(
                packet.key().clone(),
                payload,
            ));
    }

    pub fn handle_store_cookie<T>(
        &mut self,
        this: &Weak<RefCell<T>>,
        packet: &ClientboundStoreCookiePacket,
    ) where
        T: ClientCommonPacketListener + 'static,
    {
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            this,
            self.minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let payload = Arc::new(packet.payload().to_vec());
        match self
            .server_cookies
            .iter_mut()
            .find(|(key, _)| key == packet.key())
        {
            Some((_, stored)) => *stored = payload,
            None => self.server_cookies.push((packet.key().clone(), payload)),
        }
    }

    pub fn handle_custom_report_details<T>(
        &mut self,
        this: &Weak<RefCell<T>>,
        packet: &ClientboundCustomReportDetailsPacket,
    ) where
        T: ClientCommonPacketListener + 'static,
    {
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            this,
            self.minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.custom_report_details = packet.details().clone();
    }

    pub fn handle_clear_dialog<T>(
        &mut self,
        this: &Weak<RefCell<T>>,
        packet: &ClientboundClearDialogPacket,
    ) where
        T: ClientCommonPacketListener + 'static,
    {
        let _ = PacketUtils::ensure_running_on_same_thread(
            packet,
            this,
            self.minecraft.packet_processor(),
        );
    }

    /// Java disconnects and connects to the server the packet names; the port only disconnects.
    pub fn handle_transfer<T>(
        &mut self,
        this: &Weak<RefCell<T>>,
        packet: &ClientboundTransferPacket,
    ) where
        T: ClientCommonPacketListener + 'static,
    {
        self.shared.is_transferring.store(true, Ordering::SeqCst);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            this,
            self.minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        assert!(
            self.server_data.is_some(),
            "Cannot transfer to server from singleplayer"
        );
        eprintln!(
            "Transfers aren't ported yet: not connecting to {}:{}",
            packet.host(),
            packet.port()
        );
        self.connection
            .disconnect(DisconnectionDetails::new("disconnect.transfer"));
        self.connection.set_read_only();
        self.connection.handle_disconnection();
    }

    /// On the Netty thread, like Java's (`NettyHalf`): only a bundle's reaches the game thread.
    pub fn handle_disconnect(&mut self, packet: &ClientboundDisconnectPacket) {
        self.shared.handle_disconnect(packet);
    }

    pub fn send_deferred_packets(&mut self) {
        let mut deferred_packets = lock(&self.shared.deferred_packets);
        let mut index = 0;
        while index < deferred_packets.len() {
            let deferred_packet = &deferred_packets[index];
            if (deferred_packet.send_condition)() {
                let deferred_packet = deferred_packets.remove(index);
                self.connection.send_boxed(deferred_packet.packet);
            } else if deferred_packet.expiration_time <= Util::get_millis() {
                deferred_packets.remove(index);
            } else {
                index += 1;
            }
        }
    }

    pub fn send(&self, packet: impl Packet<S>) {
        self.connection.send::<S>(packet);
    }

    pub fn on_disconnect(&mut self, details: &DisconnectionDetails) {
        self.minecraft.disconnect(details, self.is_transferring());
        eprintln!("Client disconnected with reason: {}", details.reason());
    }

    pub fn server_brand(&self) -> Option<&str> {
        self.server_brand.as_deref()
    }

    /// The title of the disconnected screen.
    pub fn generic_disconnect_message() -> &'static str {
        ClientCommonPacketListenerImpl::<S>::GENERIC_DISCONNECT_MESSAGE
    }
}

/// What Netty's thread runs of the listener: the handlers Java runs there, and for the others,
/// up to `ensureRunningOnSameThread`, which queues the packet for the game thread's half.
pub struct NettyHalf<S: ?Sized> {
    pub connection: Arc<Connection>,
    pub packet_processor: Arc<PacketProcessor>,
    shared: Arc<Shared<S>>,
}

impl<S: ServerCommonPacketListener + ?Sized + 'static> NettyHalf<S> {
    pub fn should_handle_message(&self, packet: &dyn std::any::Any) -> bool {
        self.shared.should_handle_message(packet)
    }

    pub fn handle_keep_alive(&self, packet: &ClientboundKeepAlivePacket) {
        self.shared.handle_keep_alive(packet);
    }

    pub fn handle_disconnect(&self, packet: &ClientboundDisconnectPacket) {
        self.shared.handle_disconnect(packet);
    }

    /// A discarded payload goes no further, on Netty's thread; any other on the game thread.
    pub fn handle_custom_payload<T>(
        &self,
        this: &GameThreadListener<T>,
        packet: &ClientboundCustomPayloadPacket,
    ) where
        T: ClientCommonPacketListener + 'static,
    {
        let payload = packet.payload();
        let payload: &dyn std::any::Any = &*payload;
        if !payload.is::<DiscardedPayload>() {
            self.on_game_thread(this, packet);
        }
    }

    /// The transfer flag is set at once, which lets the packets of a transfer through; the rest
    /// on the game thread.
    pub fn handle_transfer<T>(
        &self,
        this: &GameThreadListener<T>,
        packet: &ClientboundTransferPacket,
    ) where
        T: ClientCommonPacketListener + 'static,
    {
        self.shared.is_transferring.store(true, Ordering::SeqCst);
        self.on_game_thread(this, packet);
    }

    /// A handler that starts with `ensureRunningOnSameThread`, which queues its packet for the
    /// game thread's half.
    pub fn on_game_thread<T, P>(&self, this: &GameThreadListener<T>, packet: &P)
    where
        T: ClientCommonPacketListener + 'static,
        P: Packet<T> + Clone + 'static,
    {
        let _ = PacketUtils::ensure_running_on_same_thread(packet, this, &self.packet_processor);
    }
}

/// Implements the cookie and common listeners' handlers on a client listener that holds a
/// `ClientCommonPacketListenerImpl` in `common` and a weak reference to itself in `this`.
macro_rules! client_common_handlers {
    ($listener:ty) => {
        impl $crate::net::minecraft::network::clientbound_packet_listener::ClientboundPacketListener
            for $listener
        {
        }

        impl $crate::net::minecraft::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener
            for $listener
        {
            fn handle_request_cookie(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::cookie::clientbound_cookie_request_packet::ClientboundCookieRequestPacket,
            ) {
                let this = self.this.clone();
                self.common.handle_request_cookie(&this, packet);
            }
        }

        impl $crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener
            for $listener
        {
            fn handle_keep_alive(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_keep_alive_packet::ClientboundKeepAlivePacket,
            ) {
                self.common.handle_keep_alive(packet);
            }

            fn handle_ping(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_ping_packet::ClientboundPingPacket,
            ) {
                let this = self.this.clone();
                self.common.handle_ping(&this, packet);
            }

            fn handle_custom_payload(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_custom_payload_packet::ClientboundCustomPayloadPacket,
            ) {
                let this = self.this.clone();
                self.common.handle_custom_payload(&this, packet);
            }

            fn handle_resource_pack_pop(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_resource_pack_pop_packet::ClientboundResourcePackPopPacket,
            ) {
                let this = self.this.clone();
                self.common.handle_resource_pack_pop(&this, packet);
            }

            fn handle_post_effects(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_post_effects_packet::ClientboundPostEffectsPacket,
            ) {
                let this = self.this.clone();
                self.common.handle_post_effects(&this, packet);
            }

            fn handle_store_cookie(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_store_cookie_packet::ClientboundStoreCookiePacket,
            ) {
                let this = self.this.clone();
                self.common.handle_store_cookie(&this, packet);
            }

            fn handle_transfer(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_transfer_packet::ClientboundTransferPacket,
            ) {
                let this = self.this.clone();
                self.common.handle_transfer(&this, packet);
            }

            fn handle_disconnect(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_disconnect_packet::ClientboundDisconnectPacket,
            ) {
                self.common.handle_disconnect(packet);
            }

            fn handle_custom_report_details(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_custom_report_details_packet::ClientboundCustomReportDetailsPacket,
            ) {
                let this = self.this.clone();
                self.common.handle_custom_report_details(&this, packet);
            }

            fn handle_clear_dialog(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_clear_dialog_packet::ClientboundClearDialogPacket,
            ) {
                let this = self.this.clone();
                self.common.handle_clear_dialog(&this, packet);
            }
        }
    };
}

pub(crate) use client_common_handlers;

/// Implements the cookie and common listeners' handlers on a client listener's half on Netty's
/// thread, which holds this class's `NettyHalf` in `common` and the name of the game thread's
/// half in `this`: the ones Java runs on Netty's thread run there, and the others queue their
/// packet for the game thread's half.
macro_rules! client_common_netty_handlers {
    ($listener:ty) => {
        impl $crate::net::minecraft::network::clientbound_packet_listener::ClientboundPacketListener
            for $listener
        {
        }

        impl $crate::net::minecraft::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener
            for $listener
        {
            fn handle_request_cookie(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::cookie::clientbound_cookie_request_packet::ClientboundCookieRequestPacket,
            ) {
                self.common.on_game_thread(&self.this, packet);
            }
        }

        impl $crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener
            for $listener
        {
            fn handle_keep_alive(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_keep_alive_packet::ClientboundKeepAlivePacket,
            ) {
                self.common.handle_keep_alive(packet);
            }

            fn handle_ping(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_ping_packet::ClientboundPingPacket,
            ) {
                self.common.on_game_thread(&self.this, packet);
            }

            fn handle_custom_payload(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_custom_payload_packet::ClientboundCustomPayloadPacket,
            ) {
                self.common.handle_custom_payload(&self.this, packet);
            }

            fn handle_resource_pack_pop(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_resource_pack_pop_packet::ClientboundResourcePackPopPacket,
            ) {
                self.common.on_game_thread(&self.this, packet);
            }

            fn handle_post_effects(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_post_effects_packet::ClientboundPostEffectsPacket,
            ) {
                self.common.on_game_thread(&self.this, packet);
            }

            fn handle_store_cookie(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_store_cookie_packet::ClientboundStoreCookiePacket,
            ) {
                self.common.on_game_thread(&self.this, packet);
            }

            fn handle_transfer(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_transfer_packet::ClientboundTransferPacket,
            ) {
                self.common.handle_transfer(&self.this, packet);
            }

            fn handle_disconnect(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_disconnect_packet::ClientboundDisconnectPacket,
            ) {
                self.common.handle_disconnect(packet);
            }

            fn handle_custom_report_details(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_custom_report_details_packet::ClientboundCustomReportDetailsPacket,
            ) {
                self.common.on_game_thread(&self.this, packet);
            }

            fn handle_clear_dialog(
                &mut self,
                packet: &$crate::net::minecraft::network::protocol::common::clientbound_clear_dialog_packet::ClientboundClearDialogPacket,
            ) {
                self.common.on_game_thread(&self.this, packet);
            }
        }
    };
}

pub(crate) use client_common_netty_handlers;
