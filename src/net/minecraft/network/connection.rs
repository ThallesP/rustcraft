//! net/minecraft/network/Connection.java
//!
//! Not ported yet: encryption (`setEncryptionKey`, `CipherDecoder` and `CipherEncoder`), which
//! online-mode servers turn on; the in-memory connections to an integrated server
//! (`connectToLocalServer`, `fromChannel`, `configureInMemoryPipeline`); the status connection
//! (`initiateServerboundStatusConnection`), which waits for the status protocol; the server's
//! side (`setListenerForServerboundHandshake`, sending a disconnect packet on a fault); the
//! bundle packer the game protocol's `bundlerInfo` adds after the decoder, so a bundle's packets
//! are handled one by one as they come; `validateListener`, which only catches a programming
//! mistake; and the bandwidth debug monitor.
//!
//! Java's connection is the last handler of a Netty pipeline, which Netty runs on an event loop
//! thread: the frame splitter, decompression, flow control, the packet decoder, and on the way
//! out the packet encoder, compression and the length prepender. Here the pipeline is the
//! connection's own, and so is the event loop, `run_event_loop`, which the game calls between
//! its ticks (`EventLoopGroupHolder`): it reads what the socket has, splits it into frames, and
//! decodes and hands each packet to the listener, the way Netty does, until the time it's given.
//! Handlers that Java runs on the event loop run there, and the ones that call
//! `PacketUtils.ensureRunningOnSameThread` queue their packet for the game thread, which takes
//! them at the start of `Minecraft.runTick` like Java's. Writes go straight to the socket, like
//! `writeAndFlush`.
//!
//! A packet the connection hands on is Java's `Packet<?>`: `send` takes the listener the packet
//! is for (`send::<dyn ServerGamePacketListener>`), and the encoder casts it to its protocol's,
//! failing like Java's `ClassCastException` for another protocol's packet. Errors the pipeline
//! throws (a packet that doesn't decode, a handler that fails) are panics, which it catches like
//! Netty's `exceptionCaught`. Components aren't ported, so a reason is the translation key or the
//! text it would show (`DisconnectionDetails`).

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::io::netty::buffer::unpooled;
use crate::net::minecraft::network::compression_decoder::CompressionDecoder;
use crate::net::minecraft::network::compression_encoder::CompressionEncoder;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_decoder::PacketDecoder;
use crate::net::minecraft::network::packet_encoder::PacketEncoder;
use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::packet_processor::panic_message;
use crate::net::minecraft::network::protocol::handshake::client_intent::ClientIntent;
use crate::net::minecraft::network::protocol::handshake::client_intention_packet::ClientIntentionPacket;
use crate::net::minecraft::network::protocol::handshake::handshake_protocols::HandshakeProtocols;
use crate::net::minecraft::network::protocol::handshake::server_handshake_packet_listener::ServerHandshakePacketListener;
use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
use crate::net::minecraft::network::protocol::login::login_protocols::LoginProtocols;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol_info::ProtocolInfo;
use crate::net::minecraft::network::varint21_frame_decoder::Varint21FrameDecoder;
use crate::net::minecraft::network::varint21_length_field_prepender::Varint21LengthFieldPrepender;
use crate::net::minecraft::server::network::event_loop_group_holder::EventLoopGroupHolder;
use crate::net::minecraft::shared_constants::SharedConstants;
use crate::net::minecraft::util::mth::Mth;

thread_local! {
    static IN_EVENT_LOOP: Cell<bool> = const { Cell::new(false) };
}

/// Whether the code running is the connection's pipeline, Java's Netty thread
/// (`PacketProcessor.isSameThread`).
pub fn in_event_loop() -> bool {
    IN_EVENT_LOOP.with(Cell::get)
}

/// Marks the code running as the event loop's until it's dropped, even when it panics.
struct EventLoopScope {
    outer: bool,
}

impl EventLoopScope {
    fn enter() -> EventLoopScope {
        EventLoopScope {
            outer: IN_EVENT_LOOP.with(|flag| flag.replace(true)),
        }
    }
}

impl Drop for EventLoopScope {
    fn drop(&mut self) {
        IN_EVENT_LOOP.with(|flag| flag.set(self.outer));
    }
}

/// The socket and the handlers around the decoder and encoder: Netty's channel and pipeline.
struct Channel {
    stream: TcpStream,
    open: bool,
    auto_read: bool,
    /// `ReadTimeoutHandler`'s clock.
    last_read: Instant,
    /// `ByteToMessageDecoder`'s cumulation, which the splitter takes frames out of.
    cumulation: ByteBuf,
    splitter: Varint21FrameDecoder,
    /// `FlowControlHandler`'s queue: frames split but not decoded yet, while reading is off.
    frames: VecDeque<ByteBuf>,
    decompress: Option<CompressionDecoder>,
    compress: Option<CompressionEncoder>,
    prepender: Varint21LengthFieldPrepender,
}

/// The pipeline's `decoder`: a protocol's `PacketDecoder`, and the listener it was set up for.
trait Inbound {
    fn decode(&self, frame: &mut ByteBuf) -> Box<dyn Received>;
}

/// A packet the decoder read, on its way to the listener.
trait Received {
    fn is_terminal(&self) -> bool;

    /// `channelRead0`: hands the packet to the listener, unless it's done with messages.
    fn handle(self: Box<Self>) -> bool;
}

struct InboundProtocol<T: ?Sized> {
    decoder: PacketDecoder<T>,
    listener: Rc<RefCell<T>>,
}

impl<T: PacketListener + ?Sized + 'static> Inbound for InboundProtocol<T> {
    fn decode(&self, frame: &mut ByteBuf) -> Box<dyn Received> {
        Box::new(ReceivedPacket {
            packet: self.decoder.decode(frame),
            listener: Rc::clone(&self.listener),
        })
    }
}

struct ReceivedPacket<T: ?Sized> {
    packet: Box<dyn Packet<T>>,
    listener: Rc<RefCell<T>>,
}

impl<T: PacketListener + ?Sized + 'static> Received for ReceivedPacket<T> {
    fn is_terminal(&self) -> bool {
        self.packet.is_terminal()
    }

    fn handle(self: Box<Self>) -> bool {
        let packet: &dyn Any = &*self.packet;
        if self.listener.borrow().should_handle_message(packet) {
            self.packet.handle(&mut *self.listener.borrow_mut());
            true
        } else {
            false
        }
    }
}

/// The pipeline's `encoder`: a protocol's `PacketEncoder`.
trait Outbound {
    /// Writes the packet, a `Box<dyn Packet<T>>` for the protocol's listener `T`, and says
    /// whether it was terminal.
    fn encode(&self, packet: Box<dyn Any>, output: &mut ByteBuf) -> bool;
}

struct OutboundProtocol<T: ?Sized> {
    encoder: PacketEncoder<T>,
}

impl<T: ?Sized + 'static> Outbound for OutboundProtocol<T> {
    fn encode(&self, packet: Box<dyn Any>, output: &mut ByteBuf) -> bool {
        let packet = packet.downcast::<Box<dyn Packet<T>>>().unwrap_or_else(|_| {
            panic!(
                "ClassCastException: a packet for another protocol than {}",
                self.encoder.protocol_info().id().id()
            )
        });
        self.encoder.encode(&packet, output);
        packet.is_terminal()
    }
}

/// What the connection calls on its listener, whatever its protocol.
trait ListenerHandle {
    fn on_disconnect(&self, details: &DisconnectionDetails);

    fn create_disconnection_info(&self, reason: &str, cause: &str) -> DisconnectionDetails;

    fn tick(&self);
}

struct Listener<T: ?Sized>(Rc<RefCell<T>>);

impl<T: PacketListener + ?Sized> ListenerHandle for Listener<T> {
    fn on_disconnect(&self, details: &DisconnectionDetails) {
        self.0.borrow_mut().on_disconnect(details);
    }

    fn create_disconnection_info(&self, reason: &str, cause: &str) -> DisconnectionDetails {
        self.0.borrow_mut().create_disconnection_info(reason, cause)
    }

    fn tick(&self) {
        if let Some(tickable) = self.0.borrow_mut().as_tickable() {
            tickable.tick();
        }
    }
}

/// One side of a connection to the other: what it receives goes to its packet listener, and
/// what it sends goes through its protocol's encoder.
pub struct Connection {
    receiving: PacketFlow,
    channel: RefCell<Option<Channel>>,
    address: Cell<Option<SocketAddr>>,
    decoder: RefCell<Option<Rc<dyn Inbound>>>,
    encoder: RefCell<Option<Rc<dyn Outbound>>>,
    disconnect_listener: RefCell<Option<Rc<dyn ListenerHandle>>>,
    packet_listener: RefCell<Option<Rc<dyn ListenerHandle>>>,
    disconnection_details: RefCell<Option<DisconnectionDetails>>,
    disconnection_handled: Cell<bool>,
    received_packets: Cell<i32>,
    sent_packets: Cell<i32>,
    average_received_packets: Cell<f32>,
    average_sent_packets: Cell<f32>,
    tick_count: Cell<i32>,
    handling_fault: Cell<bool>,
    delayed_disconnect: RefCell<Option<DisconnectionDetails>>,
}

impl Connection {
    const AVERAGE_PACKETS_SMOOTHING: f32 = 0.75;
    /// `ReadTimeoutHandler(30)`'s timeout.
    const READ_TIMEOUT: Duration = Duration::from_secs(30);

    pub fn new(receiving: PacketFlow) -> Connection {
        Connection {
            receiving,
            channel: RefCell::new(None),
            address: Cell::new(None),
            decoder: RefCell::new(None),
            encoder: RefCell::new(None),
            disconnect_listener: RefCell::new(None),
            packet_listener: RefCell::new(None),
            disconnection_details: RefCell::new(None),
            disconnection_handled: Cell::new(false),
            received_packets: Cell::new(0),
            sent_packets: Cell::new(0),
            average_received_packets: Cell::new(0.0),
            average_sent_packets: Cell::new(0.0),
            tick_count: Cell::new(0),
            handling_fault: Cell::new(false),
            delayed_disconnect: RefCell::new(None),
        }
    }

    /// The channel is up: Netty's `channelActive`.
    fn channel_active(&self, stream: TcpStream) {
        self.address.set(stream.peer_addr().ok());
        *self.channel.borrow_mut() = Some(Channel {
            stream,
            open: true,
            auto_read: true,
            last_read: Instant::now(),
            cumulation: unpooled::buffer(),
            splitter: Varint21FrameDecoder::new(),
            frames: VecDeque::new(),
            decompress: None,
            compress: None,
            prepender: Varint21LengthFieldPrepender,
        });
        let delayed = self.delayed_disconnect.borrow_mut().take();
        if let Some(details) = delayed {
            self.disconnect(details);
        }
    }

    /// The other side closed the connection: Netty's `channelInactive`.
    fn channel_inactive(&self) {
        if let Some(channel) = self.channel.borrow_mut().as_mut() {
            channel.open = false;
        }
        self.disconnect(DisconnectionDetails::new("disconnect.endOfStream"));
    }

    /// Something in the pipeline failed: Netty's `exceptionCaught`, on the client's side.
    fn exception_caught(&self, cause: &str, timeout: bool) {
        let is_first_fault = !self.handling_fault.get();
        self.handling_fault.set(true);
        if self.is_connected() {
            if timeout {
                self.disconnect(DisconnectionDetails::new("disconnect.timeout"));
            } else {
                let reason = format!("Internal Exception: {cause}");
                let listener = self.packet_listener.borrow().clone();
                let details = match listener {
                    Some(listener) => listener.create_disconnection_info(&reason, cause),
                    None => DisconnectionDetails::new(reason),
                };
                eprintln!("Connection error: {cause}");
                self.disconnect(details);
                if is_first_fault {
                    self.set_read_only();
                }
            }
        }
    }

    /// Hands a frame's packet to the listener: `PacketDecoder.decode`, then `channelRead0`. A
    /// terminal packet takes the decoder out before it's handled, so the packets after it wait
    /// for the next protocol (`ProtocolSwapHandler.handleInboundTerminalPacket`).
    fn channel_read(&self, decoder: Rc<dyn Inbound>, mut frame: ByteBuf) {
        let packet = decoder.decode(&mut frame);
        if packet.is_terminal() {
            let mut current = self.decoder.borrow_mut();
            if current
                .as_ref()
                .is_some_and(|current| Rc::ptr_eq(current, &decoder))
            {
                *current = None;
                drop(current);
                self.set_auto_read(false);
            }
        }

        if self.is_connected() && packet.handle() {
            self.received_packets.set(self.received_packets.get() + 1);
        }
    }

    pub fn setup_inbound_protocol<T: PacketListener + ?Sized + 'static>(
        &self,
        protocol: Box<dyn ProtocolInfo<T>>,
        packet_listener: Rc<RefCell<T>>,
    ) {
        assert!(
            protocol.flow() == self.get_receiving(),
            "Invalid inbound protocol: {}",
            protocol.id().id()
        );

        *self.packet_listener.borrow_mut() = Some(Rc::new(Listener(Rc::clone(&packet_listener))));
        *self.disconnect_listener.borrow_mut() = None;
        *self.decoder.borrow_mut() = Some(Rc::new(InboundProtocol {
            decoder: PacketDecoder::new(protocol),
            listener: packet_listener,
        }));
        self.set_auto_read(true);
    }

    pub fn setup_outbound_protocol<T: ?Sized + 'static>(&self, protocol: Box<dyn ProtocolInfo<T>>) {
        assert!(
            protocol.flow() == self.get_sending(),
            "Invalid outbound protocol: {}",
            protocol.id().id()
        );

        *self.encoder.borrow_mut() = Some(Rc::new(OutboundProtocol {
            encoder: PacketEncoder::new(protocol),
        }));
    }

    pub fn initiate_serverbound_play_connection(
        &self,
        host_name: &str,
        port: i32,
        listener: Rc<RefCell<dyn ClientLoginPacketListener>>,
    ) {
        self.initiate_serverbound_connection(
            host_name,
            port,
            Box::new(&***LoginProtocols::SERVERBOUND),
            Box::new(&***LoginProtocols::CLIENTBOUND),
            listener,
            ClientIntent::LOGIN,
        );
    }

    fn initiate_serverbound_connection<
        S: ?Sized + 'static,
        C: PacketListener + ?Sized + 'static,
    >(
        &self,
        host_name: &str,
        port: i32,
        outbound: Box<dyn ProtocolInfo<S>>,
        inbound: Box<dyn ProtocolInfo<C>>,
        listener: Rc<RefCell<C>>,
        intent: ClientIntent,
    ) {
        assert!(
            outbound.id() == inbound.id(),
            "Mismatched initial protocols"
        );

        *self.disconnect_listener.borrow_mut() = Some(Rc::new(Listener(Rc::clone(&listener))));
        self.setup_inbound_protocol(inbound, listener);
        self.send::<dyn ServerHandshakePacketListener>(ClientIntentionPacket::new(
            SharedConstants::get_protocol_version(),
            host_name.to_string(),
            port,
            intent,
        ));
        self.setup_outbound_protocol(outbound);
    }

    /// Sends the packet, for the listener `L` of the protocol it belongs to, at once. Dropped
    /// once the connection is closed, where Java queues it for a channel that never comes back.
    pub fn send<L: ?Sized + 'static>(&self, packet: impl Packet<L>) {
        self.send_boxed(Box::new(packet) as Box<dyn Packet<L>>);
    }

    /// `send`, for a packet already boxed as one of the protocol's.
    pub fn send_boxed<L: ?Sized + 'static>(&self, packet: Box<dyn Packet<L>>) {
        if self.is_connected() {
            self.send_packet(packet);
        }
    }

    fn send_packet<L: ?Sized + 'static>(&self, packet: Box<dyn Packet<L>>) {
        self.sent_packets.set(self.sent_packets.get() + 1);
        let r#type = packet.r#type();
        let result =
            panic::catch_unwind(AssertUnwindSafe(|| self.do_send_packet(Box::new(packet))));
        if let Err(cause) = result {
            eprintln!("Error sending packet {type}");
            self.exception_caught(&panic_message(&*cause), false);
        }
    }

    /// The packet through the encoder, compression and the prepender, onto the socket. After a
    /// terminal packet the encoder is gone until the next protocol's is set up
    /// (`ProtocolSwapHandler.handleOutboundTerminalPacket`).
    fn do_send_packet(&self, packet: Box<dyn Any>) {
        let encoder = self.encoder.borrow().clone();
        let Some(encoder) = encoder else {
            panic!("Pipeline has no outbound protocol configured, can't process packet");
        };
        let mut encoded = unpooled::buffer();
        let terminal = encoder.encode(packet, &mut encoded);
        if terminal {
            let mut current = self.encoder.borrow_mut();
            if current
                .as_ref()
                .is_some_and(|current| Rc::ptr_eq(current, &encoder))
            {
                *current = None;
            }
        }

        let written = {
            let mut channel = self.channel.borrow_mut();
            let Some(channel) = channel.as_mut() else {
                return;
            };
            if let Some(compress) = channel.compress.as_mut() {
                let mut compressed = unpooled::buffer();
                compress.encode(&mut encoded, &mut compressed);
                encoded = compressed;
            }
            let mut frame = unpooled::buffer();
            channel.prepender.encode(&encoded, &mut frame);
            channel.stream.write_all(&readable_bytes(&frame))
        };
        if let Err(error) = written {
            self.close_channel();
            panic!("{error}");
        }
    }

    pub fn tick(&self) {
        let listener = self.packet_listener.borrow().clone();
        if let Some(listener) = listener {
            listener.tick();
        }

        if !self.is_connected() && !self.disconnection_handled.get() {
            self.handle_disconnection();
        }

        if self.tick_count.get() % 20 == 0 {
            self.tick_second();
        }
        self.tick_count.set(self.tick_count.get().wrapping_add(1));
    }

    fn tick_second(&self) {
        self.average_sent_packets.set(Mth::lerp_f32(
            Connection::AVERAGE_PACKETS_SMOOTHING,
            self.sent_packets.get() as f32,
            self.average_sent_packets.get(),
        ));
        self.average_received_packets.set(Mth::lerp_f32(
            Connection::AVERAGE_PACKETS_SMOOTHING,
            self.received_packets.get() as f32,
            self.average_received_packets.get(),
        ));
        self.sent_packets.set(0);
        self.received_packets.set(0);
    }

    pub fn get_remote_address(&self) -> Option<SocketAddr> {
        self.address.get()
    }

    pub fn disconnect(&self, details: DisconnectionDetails) {
        if self.channel.borrow().is_none() {
            *self.delayed_disconnect.borrow_mut() = Some(details.clone());
        }

        if self.is_connected() {
            self.close_channel();
            *self.disconnection_details.borrow_mut() = Some(details);
        }
    }

    fn close_channel(&self) {
        if let Some(channel) = self.channel.borrow_mut().as_mut() {
            if channel.open {
                let _ = channel.stream.shutdown(Shutdown::Both);
            }
            channel.open = false;
        }
    }

    pub fn is_memory_connection(&self) -> bool {
        false
    }

    pub fn get_receiving(&self) -> PacketFlow {
        self.receiving
    }

    pub fn get_sending(&self) -> PacketFlow {
        self.receiving.get_opposite()
    }

    /// Opens a TCP connection to a server and registers it with the event loop group, like
    /// `connect`'s bootstrap: no Nagle, a read timeout, and the frame handlers, with the
    /// handshake's encoder ready and no decoder yet.
    pub fn connect_to_server(address: SocketAddr) -> std::io::Result<Rc<Connection>> {
        let connection = Rc::new(Connection::new(PacketFlow::CLIENTBOUND));
        Connection::connect(address, EventLoopGroupHolder::remote(), &connection)?;
        Ok(connection)
    }

    pub fn connect(
        address: SocketAddr,
        event_loop_group_holder: &EventLoopGroupHolder,
        connection: &Rc<Connection>,
    ) -> std::io::Result<()> {
        let stream = TcpStream::connect(address)?;
        let _ = stream.set_nodelay(true);
        connection.setup_outbound_protocol(Box::new(&***HandshakeProtocols::SERVERBOUND));
        connection.channel_active(stream);
        event_loop_group_holder.register(connection);
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.channel
            .borrow()
            .as_ref()
            .is_some_and(|channel| channel.open)
    }

    pub fn is_connecting(&self) -> bool {
        self.channel.borrow().is_none()
    }

    pub fn get_disconnection_details(&self) -> Option<DisconnectionDetails> {
        self.disconnection_details.borrow().clone()
    }

    pub fn set_read_only(&self) {
        self.set_auto_read(false);
    }

    fn set_auto_read(&self, auto_read: bool) {
        if let Some(channel) = self.channel.borrow_mut().as_mut() {
            channel.auto_read = auto_read;
        }
    }

    pub fn setup_compression(&self, threshold: i32, validate_decompressed: bool) {
        let mut channel = self.channel.borrow_mut();
        let Some(channel) = channel.as_mut() else {
            return;
        };
        if threshold >= 0 {
            match channel.decompress.as_mut() {
                Some(decompress) => decompress.set_threshold(threshold, validate_decompressed),
                None => {
                    channel.decompress =
                        Some(CompressionDecoder::new(threshold, validate_decompressed))
                }
            }
            match channel.compress.as_mut() {
                Some(compress) => compress.set_threshold(threshold),
                None => channel.compress = Some(CompressionEncoder::new(threshold)),
            }
        } else {
            channel.decompress = None;
            channel.compress = None;
        }
    }

    pub fn handle_disconnection(&self) {
        if self.channel.borrow().is_some() && !self.is_connected() {
            if self.disconnection_handled.get() {
                eprintln!("handleDisconnection() called twice");
            } else {
                self.disconnection_handled.set(true);
                let packet_listener = self.packet_listener.borrow().clone();
                let disconnect_listener =
                    packet_listener.or_else(|| self.disconnect_listener.borrow().clone());
                if let Some(disconnect_listener) = disconnect_listener {
                    let details = self.get_disconnection_details().unwrap_or_else(|| {
                        DisconnectionDetails::new("multiplayer.disconnect.generic")
                    });
                    disconnect_listener.on_disconnect(&details);
                }
            }
        }
    }

    pub fn get_average_received_packets(&self) -> f32 {
        self.average_received_packets.get()
    }

    pub fn get_average_sent_packets(&self) -> f32 {
        self.average_sent_packets.get()
    }

    /// Netty's event loop for this connection, until `deadline`: decodes and hands on every
    /// packet whose bytes have arrived, waiting on the socket for more in between.
    pub fn run_event_loop(&self, deadline: Instant) {
        let _scope = EventLoopScope::enter();
        loop {
            self.fire_frames();
            if !self.is_connected() {
                return;
            }

            let now = Instant::now();
            if now >= deadline {
                return;
            }
            let reading = self
                .channel
                .borrow()
                .as_ref()
                .is_some_and(|channel| channel.auto_read);
            if !reading {
                // Reading is off until the game thread sets the next protocol up.
                std::thread::sleep(deadline - now);
                return;
            }
            if !self.read_socket(deadline - now) {
                return;
            }
        }
    }

    /// Waits up to `timeout` for bytes and splits what came into frames. False when nothing
    /// came, or the channel closed or failed.
    fn read_socket(&self, timeout: Duration) -> bool {
        let mut bytes = [0u8; 65536];
        let result = {
            let mut channel = self.channel.borrow_mut();
            let Some(channel) = channel.as_mut() else {
                return false;
            };
            let timeout = timeout.max(Duration::from_millis(1));
            let _ = channel.stream.set_read_timeout(Some(timeout));
            channel.stream.read(&mut bytes)
        };

        match result {
            Ok(0) => {
                self.channel_inactive();
                false
            }
            Ok(read) => {
                let split = panic::catch_unwind(AssertUnwindSafe(|| self.split(&bytes[..read])));
                if let Err(cause) = split {
                    self.exception_caught(&panic_message(&*cause), false);
                    return false;
                }
                true
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                let timed_out =
                    self.channel.borrow().as_ref().is_some_and(|channel| {
                        channel.last_read.elapsed() >= Connection::READ_TIMEOUT
                    });
                if timed_out {
                    self.exception_caught("ReadTimeoutException", true);
                }
                false
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => true,
            Err(error) => {
                self.exception_caught(&error.to_string(), false);
                false
            }
        }
    }

    /// The splitter, on what just came: Netty's cumulation and decode loop.
    fn split(&self, bytes: &[u8]) {
        let mut channel = self.channel.borrow_mut();
        let Some(channel) = channel.as_mut() else {
            return;
        };
        channel.last_read = Instant::now();
        let bytes: Vec<i8> = bytes.iter().map(|&byte| byte as i8).collect();
        channel.cumulation.write_bytes_array(&bytes);

        let mut out = Vec::new();
        loop {
            let found = out.len();
            channel.splitter.decode(&mut channel.cumulation, &mut out);
            if !channel.cumulation.is_readable() || out.len() == found {
                break;
            }
        }
        channel.frames.extend(out);

        // What's been read goes, like Netty's `discardSomeReadBytes`.
        let readable = channel.cumulation.readable_bytes();
        let mut rest = unpooled::buffer_initial_capacity(readable.max(256));
        rest.write_bytes_buf(&mut channel.cumulation);
        channel.cumulation = rest;
    }

    /// Decompresses, decodes and hands on the frames that have arrived, while there's a decoder
    /// to read them and reading is on.
    fn fire_frames(&self) {
        loop {
            if !self.is_connected() {
                return;
            }
            let decoder = self.decoder.borrow().clone();
            let Some(decoder) = decoder else {
                return;
            };
            let frame = {
                let mut channel = self.channel.borrow_mut();
                let Some(channel) = channel.as_mut() else {
                    return;
                };
                if !channel.auto_read {
                    return;
                }
                let Some(frame) = channel.frames.pop_front() else {
                    return;
                };
                (frame, channel.decompress.is_some())
            };

            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                let (mut frame, compressed) = frame;
                if compressed {
                    let mut out = Vec::new();
                    if let Some(decompress) = self
                        .channel
                        .borrow_mut()
                        .as_mut()
                        .and_then(|channel| channel.decompress.as_mut())
                    {
                        decompress.decode(&mut frame, &mut out);
                    }
                    frame = out.pop().expect("a frame decompresses to one packet");
                }
                self.channel_read(decoder, frame);
            }));
            if let Err(cause) = result {
                self.exception_caught(&panic_message(&*cause), false);
            }
        }
    }
}

/// A buffer's readable bytes, for the socket.
fn readable_bytes(buffer: &ByteBuf) -> Vec<u8> {
    let length = buffer.readable_bytes();
    let mut bytes = vec![0i8; length as usize];
    buffer.get_bytes_array(buffer.reader_index(), &mut bytes, 0, length);
    bytes.into_iter().map(|byte| byte as u8).collect()
}
