//! net/minecraft/network/Connection.java
//!
//! Not ported yet: encryption (`setEncryptionKey`, `CipherDecoder` and `CipherEncoder`), which
//! online-mode servers turn on; the in-memory connections to an integrated server
//! (`connectToLocalServer`, `fromChannel`, `configureInMemoryPipeline`); the server's
//! side (`setListenerForServerboundHandshake`, sending a disconnect packet on a fault, and the
//! bundle unpacker its encoder needs); `validateListener`, which only catches a programming
//! mistake; the pending actions of a connection that isn't connected yet (`runOnceConnected`,
//! `flushQueue`: the port's connections are active once `connect` returns); and the bandwidth
//! debug monitor.
//!
//! Java's connection is the last handler of a Netty pipeline, which Netty runs on an event loop
//! thread: the frame splitter, decompression, flow control, the packet decoder and the bundle
//! packer, and on the way out the packet encoder, compression and the length prepender. Here the
//! pipeline is the connection's own, and so is its event loop thread (`run_event_loop`, which
//! `EventLoopGroupHolder` starts): it takes what the socket read, splits it into frames, decodes
//! them and hands each packet to the listener, and it runs what other threads hand it (Netty's
//! `execute`) in the order they did, writes above all. Rust's sockets block, so a second thread
//! reads the socket for it and hands it what came, where Netty's selector would wake it.
//!
//! A listener is two halves (PORTING.md, "The connection's threads"): the game thread's (an
//! `Rc`), which the connection ticks and tells about the disconnection, and the half Netty's
//! thread hands packets to (`NettyListener`), whose handlers run there when Java's do and queue
//! the packet for the game thread when Java's call `PacketUtils.ensureRunningOnSameThread`. The
//! connection itself is shared by both threads, like Java's (an `Arc`): what both touch is
//! atomic or behind a lock, and the pipeline's handlers are only run by the event loop.
//!
//! A packet the connection hands on is Java's `Packet<?>`: `send` takes the listener the packet
//! is for (`send::<dyn ServerGamePacketListener>`), and the encoder casts it to its protocol's,
//! failing like Java's `ClassCastException` for another protocol's packet. Errors the pipeline
//! throws (a packet that doesn't decode, a handler that fails) are panics, which it catches like
//! Netty's `exceptionCaught`. A reason is the translation key or the text it would show
//! (`DisconnectionDetails`).

use std::any::Any;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::io::netty::buffer::unpooled;
use crate::net::minecraft::network::compression_decoder::CompressionDecoder;
use crate::net::minecraft::network::compression_encoder::CompressionEncoder;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_bundle_packer::PacketBundlePacker;
use crate::net::minecraft::network::packet_decoder::PacketDecoder;
use crate::net::minecraft::network::packet_encoder::PacketEncoder;
use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::packet_processor::{GameThreadListener, lock, panic_message};
use crate::net::minecraft::network::protocol::handshake::client_intent::ClientIntent;
use crate::net::minecraft::network::protocol::handshake::client_intention_packet::ClientIntentionPacket;
use crate::net::minecraft::network::protocol::handshake::handshake_protocols::HandshakeProtocols;
use crate::net::minecraft::network::protocol::handshake::server_handshake_packet_listener::ServerHandshakePacketListener;
use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
use crate::net::minecraft::network::protocol::login::login_protocols::LoginProtocols;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::protocol::status::client_status_packet_listener::ClientStatusPacketListener;
use crate::net::minecraft::network::protocol::status::status_protocols::StatusProtocols;
use crate::net::minecraft::network::protocol_info::ProtocolInfo;
use crate::net::minecraft::network::varint21_frame_decoder::Varint21FrameDecoder;
use crate::net::minecraft::network::varint21_length_field_prepender::Varint21LengthFieldPrepender;
use crate::net::minecraft::server::network::event_loop_group_holder::EventLoopGroupHolder;
use crate::net::minecraft::shared_constants::SharedConstants;
use crate::net::minecraft::util::mth::Mth;

/// The half of a packet listener Netty's thread hands packets to: the listener `T` of a
/// protocol, whose handlers run where Java's do (PORTING.md, "The connection's threads").
pub trait NettyListener<T: ?Sized>: Send {
    fn listener(&mut self) -> &mut T;
}

/// A listener of the game thread, which makes the half of it Netty's thread runs. That half
/// queues the packets the game thread handles for `this`.
pub trait SplitListener<T: ?Sized> {
    fn netty_listener(&self, this: GameThreadListener<Self>) -> Box<dyn NettyListener<T>>;
}

/// What the event loop takes, in the order it came: what the socket read, and the tasks other
/// threads hand it.
enum Event {
    Read(Vec<u8>),
    /// The other side closed the connection.
    Inactive,
    ReadFailed(String),
    Task(Box<dyn FnOnce(&Connection) + Send>),
}

/// Netty's channel: the socket, and the event loop that runs its pipeline.
struct Channel {
    open: AtomicBool,
    /// The socket, for closing it.
    stream: TcpStream,
    events: Sender<Event>,
    /// The event loop's thread, which sets it as it starts, before it takes anything.
    event_loop: OnceLock<ThreadId>,
}

/// The pipeline's handlers before the decoder: `ReadTimeoutHandler`'s clock, the splitter with
/// what it has cumulated, decompression, and `FlowControlHandler`'s queue of frames split but
/// not decoded yet, while reading is off.
struct InboundPipeline {
    last_read: Instant,
    cumulation: ByteBuf,
    splitter: Varint21FrameDecoder,
    frames: VecDeque<ByteBuf>,
    decompress: Option<CompressionDecoder>,
}

/// The pipeline's handlers on the way out: the encoder, compression and the prepender, onto
/// the socket.
struct OutboundPipeline {
    stream: Option<TcpStream>,
    encoder: Option<Arc<dyn Outbound>>,
    compress: Option<CompressionEncoder>,
    prepender: Varint21LengthFieldPrepender,
}

/// The pipeline's `decoder`: a protocol's `PacketDecoder`, the `bundler` after it when the
/// protocol has bundles, and the half of the listener Netty's thread hands their packets to.
trait Inbound: Send + Sync {
    /// Decodes a frame's packet, and says whether it was terminal and what the handlers after
    /// the decoder hand on: the packet, or nothing while the bundle packer gathers a bundle, or
    /// the bundle.
    fn decode(&self, frame: &mut ByteBuf) -> (bool, Vec<Box<dyn Received>>);

    /// The listener's details of a disconnection the connection's own error caused, unless the
    /// listener is the one handling a packet.
    fn create_disconnection_info(&self, reason: &str, cause: &str) -> Option<DisconnectionDetails>;
}

/// A packet the decoder read, on its way to the listener.
trait Received {
    /// `channelRead0`: hands the packet to the listener, unless it's done with messages.
    fn handle(self: Box<Self>) -> bool;
}

type SharedNettyListener<T> = Arc<Mutex<Box<dyn NettyListener<T>>>>;

struct InboundProtocol<T: ?Sized> {
    decoder: PacketDecoder<T>,
    bundler: Option<Mutex<PacketBundlePacker<T>>>,
    listener: SharedNettyListener<T>,
}

impl<T: PacketListener + ?Sized + 'static> Inbound for InboundProtocol<T> {
    fn decode(&self, frame: &mut ByteBuf) -> (bool, Vec<Box<dyn Received>>) {
        let packet = self.decoder.decode(frame);
        let terminal = packet.is_terminal();
        let packets = match &self.bundler {
            Some(bundler) => {
                let mut out = Vec::new();
                lock(bundler).decode(packet, &mut out);
                out
            }
            None => vec![packet],
        };
        let received = packets
            .into_iter()
            .map(|packet| {
                Box::new(ReceivedPacket {
                    packet,
                    listener: Arc::clone(&self.listener),
                }) as Box<dyn Received>
            })
            .collect();
        (terminal, received)
    }

    fn create_disconnection_info(&self, reason: &str, cause: &str) -> Option<DisconnectionDetails> {
        let mut listener = self.listener.try_lock().ok()?;
        Some(listener.listener().create_disconnection_info(reason, cause))
    }
}

struct ReceivedPacket<T: ?Sized> {
    packet: Box<dyn Packet<T>>,
    listener: SharedNettyListener<T>,
}

impl<T: PacketListener + ?Sized + 'static> Received for ReceivedPacket<T> {
    fn handle(self: Box<Self>) -> bool {
        let mut listener = lock(&self.listener);
        let listener = listener.listener();
        let packet: &dyn Any = &*self.packet;
        if listener.should_handle_message(packet) {
            self.packet.handle(listener);
            true
        } else {
            false
        }
    }
}

/// The pipeline's `encoder`: a protocol's `PacketEncoder`.
trait Outbound: Send + Sync {
    /// Writes the packet, a `Box<dyn Packet<T>>` for the protocol's listener `T`, and says
    /// whether it was terminal.
    fn encode(&self, packet: Box<dyn Any + Send>, output: &mut ByteBuf) -> bool;
}

struct OutboundProtocol<T: ?Sized> {
    encoder: PacketEncoder<T>,
}

impl<T: ?Sized + 'static> Outbound for OutboundProtocol<T> {
    fn encode(&self, packet: Box<dyn Any + Send>, output: &mut ByteBuf) -> bool {
        let packet = (packet as Box<dyn Any>)
            .downcast::<Box<dyn Packet<T>>>()
            .unwrap_or_else(|_| {
                panic!(
                    "ClassCastException: a packet for another protocol than {}",
                    self.encoder.protocol_info().id().id()
                )
            });
        self.encoder.encode(&packet, output);
        packet.is_terminal()
    }
}

/// What the connection calls on the game thread's half of its listener, whatever its protocol.
trait ListenerHandle: Send + Sync {
    fn on_disconnect(&self, details: &DisconnectionDetails);

    fn tick(&self);
}

impl<T: PacketListener + ?Sized + 'static> ListenerHandle for GameThreadListener<T> {
    fn on_disconnect(&self, details: &DisconnectionDetails) {
        self.with(|listener| listener.borrow_mut().on_disconnect(details));
    }

    fn tick(&self) {
        self.with(|listener| {
            if let Some(tickable) = listener.borrow_mut().as_tickable() {
                tickable.tick();
            }
        });
    }
}

/// One side of a connection to the other: what it receives goes to its packet listener, and
/// what it sends goes through its protocol's encoder.
pub struct Connection {
    receiving: PacketFlow,
    channel: OnceLock<Channel>,
    address: OnceLock<Option<SocketAddr>>,
    inbound: Mutex<InboundPipeline>,
    decoder: Mutex<Option<Arc<dyn Inbound>>>,
    outbound: Mutex<OutboundPipeline>,
    /// The channel's `autoRead`: whether frames go on to the decoder.
    auto_read: AtomicBool,
    /// Whether the event loop is handing frames on, so a handler setting a protocol up doesn't
    /// start again from inside.
    firing: AtomicBool,
    disconnect_listener: Mutex<Option<Arc<dyn ListenerHandle>>>,
    packet_listener: Mutex<Option<Arc<dyn ListenerHandle>>>,
    disconnection_details: Mutex<Option<DisconnectionDetails>>,
    disconnection_handled: AtomicBool,
    received_packets: AtomicI32,
    sent_packets: AtomicI32,
    average_received_packets: Mutex<f32>,
    average_sent_packets: Mutex<f32>,
    tick_count: AtomicI32,
    handling_fault: AtomicBool,
    delayed_disconnect: Mutex<Option<DisconnectionDetails>>,
}

impl Connection {
    const AVERAGE_PACKETS_SMOOTHING: f32 = 0.75;
    /// `ReadTimeoutHandler(30)`'s timeout.
    const READ_TIMEOUT: Duration = Duration::from_secs(30);

    pub fn new(receiving: PacketFlow) -> Connection {
        Connection {
            receiving,
            channel: OnceLock::new(),
            address: OnceLock::new(),
            inbound: Mutex::new(InboundPipeline {
                last_read: Instant::now(),
                cumulation: unpooled::buffer(),
                splitter: Varint21FrameDecoder::new(),
                frames: VecDeque::new(),
                decompress: None,
            }),
            decoder: Mutex::new(None),
            outbound: Mutex::new(OutboundPipeline {
                stream: None,
                encoder: None,
                compress: None,
                prepender: Varint21LengthFieldPrepender,
            }),
            auto_read: AtomicBool::new(true),
            firing: AtomicBool::new(false),
            disconnect_listener: Mutex::new(None),
            packet_listener: Mutex::new(None),
            disconnection_details: Mutex::new(None),
            disconnection_handled: AtomicBool::new(false),
            received_packets: AtomicI32::new(0),
            sent_packets: AtomicI32::new(0),
            average_received_packets: Mutex::new(0.0),
            average_sent_packets: Mutex::new(0.0),
            tick_count: AtomicI32::new(0),
            handling_fault: AtomicBool::new(false),
            delayed_disconnect: Mutex::new(None),
        }
    }

    /// The channel is up: Netty's `channelActive`.
    fn channel_active(&self) {
        let address = self
            .channel
            .get()
            .and_then(|channel| channel.stream.peer_addr().ok());
        let _ = self.address.set(address);
        let delayed = lock(&self.delayed_disconnect).take();
        if let Some(details) = delayed {
            self.disconnect(details);
        }
    }

    /// The other side closed the connection: Netty's `channelInactive`, after it closed the
    /// channel.
    fn channel_inactive(&self) {
        if let Some(channel) = self.channel.get() {
            channel.open.store(false, Ordering::SeqCst);
        }
        self.disconnect(DisconnectionDetails::new("disconnect.endOfStream"));
    }

    /// Something in the pipeline failed: Netty's `exceptionCaught`, on the client's side.
    fn exception_caught(&self, cause: &str, timeout: bool) {
        let is_first_fault = !self.handling_fault.swap(true, Ordering::SeqCst);
        if self.is_connected() {
            if timeout {
                self.disconnect(DisconnectionDetails::new("disconnect.timeout"));
            } else {
                let reason = format!("Internal Exception: {cause}");
                let decoder = lock(&self.decoder).clone();
                let details = decoder
                    .and_then(|decoder| decoder.create_disconnection_info(&reason, cause))
                    .unwrap_or_else(|| DisconnectionDetails::new(reason));
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
    fn channel_read(&self, decoder: Arc<dyn Inbound>, mut frame: ByteBuf) {
        let (terminal, packets) = decoder.decode(&mut frame);
        if terminal {
            let mut current = lock(&self.decoder);
            if current
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, &decoder))
            {
                *current = None;
                drop(current);
                self.set_auto_read(false);
            }
        }

        for packet in packets {
            if self.is_connected() && packet.handle() {
                self.received_packets.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Sets the protocol the connection receives in up, with its listener: the game thread's
    /// half the connection ticks, and the half Netty's thread hands packets to, which it makes.
    /// Like Java's, it waits until the event loop has set it up.
    pub fn setup_inbound_protocol<T, L>(
        &self,
        protocol: Box<dyn ProtocolInfo<T>>,
        packet_listener: Rc<RefCell<L>>,
    ) where
        T: PacketListener + ?Sized + 'static,
        L: SplitListener<T> + PacketListener + 'static,
    {
        assert!(
            protocol.flow() == self.get_receiving(),
            "Invalid inbound protocol: {}",
            protocol.id().id()
        );

        let this = GameThreadListener::new(&packet_listener);
        *lock(&self.packet_listener) = Some(Arc::new(this.clone()));
        *lock(&self.disconnect_listener) = None;
        let listener = packet_listener.borrow().netty_listener(this);
        let bundler = protocol
            .bundler_info()
            .map(|bundler_info| Mutex::new(PacketBundlePacker::new(bundler_info)));
        let decoder: Arc<dyn Inbound> = Arc::new(InboundProtocol {
            decoder: PacketDecoder::new(protocol),
            bundler,
            listener: Arc::new(Mutex::new(listener)),
        });
        self.sync_after_configuration_change(move |connection| {
            *lock(&connection.decoder) = Some(decoder);
            connection.set_auto_read(true);
            connection.fire_frames();
        });
    }

    /// Sets the protocol the connection sends in up. Like Java's, it waits until the event loop
    /// has set it up.
    pub fn setup_outbound_protocol<T: ?Sized + 'static>(&self, protocol: Box<dyn ProtocolInfo<T>>) {
        assert!(
            protocol.flow() == self.get_sending(),
            "Invalid outbound protocol: {}",
            protocol.id().id()
        );

        let encoder: Arc<dyn Outbound> = Arc::new(OutboundProtocol {
            encoder: PacketEncoder::new(protocol),
        });
        self.sync_after_configuration_change(move |connection| {
            lock(&connection.outbound).encoder = Some(encoder);
        });
    }

    /// Runs a change to the pipeline on the event loop and waits for it, like Java's
    /// `syncAfterConfigurationChange` of the message that carries it.
    fn sync_after_configuration_change(&self, change: impl FnOnce(&Connection) + Send + 'static) {
        if !self.execute_and_wait(change) {
            eprintln!("Connection closed during protocol change");
        }
    }

    pub fn initiate_serverbound_status_connection<L>(
        &self,
        host_name: &str,
        port: i32,
        listener: Rc<RefCell<L>>,
    ) where
        L: SplitListener<dyn ClientStatusPacketListener> + PacketListener + 'static,
    {
        self.initiate_serverbound_connection(
            host_name,
            port,
            Box::new(&***StatusProtocols::SERVERBOUND),
            Box::new(&***StatusProtocols::CLIENTBOUND),
            listener,
            ClientIntent::STATUS,
        );
    }

    pub fn initiate_serverbound_play_connection<L>(
        &self,
        host_name: &str,
        port: i32,
        listener: Rc<RefCell<L>>,
    ) where
        L: SplitListener<dyn ClientLoginPacketListener> + PacketListener + 'static,
    {
        self.initiate_serverbound_connection(
            host_name,
            port,
            Box::new(&***LoginProtocols::SERVERBOUND),
            Box::new(&***LoginProtocols::CLIENTBOUND),
            listener,
            ClientIntent::LOGIN,
        );
    }

    fn initiate_serverbound_connection<S, C, L>(
        &self,
        host_name: &str,
        port: i32,
        outbound: Box<dyn ProtocolInfo<S>>,
        inbound: Box<dyn ProtocolInfo<C>>,
        listener: Rc<RefCell<L>>,
        intent: ClientIntent,
    ) where
        S: ?Sized + 'static,
        C: PacketListener + ?Sized + 'static,
        L: SplitListener<C> + PacketListener + 'static,
    {
        assert!(
            outbound.id() == inbound.id(),
            "Mismatched initial protocols"
        );

        *lock(&self.disconnect_listener) = Some(Arc::new(GameThreadListener::new(&listener)));
        self.setup_inbound_protocol(inbound, listener);
        self.send_packet::<dyn ServerHandshakePacketListener>(Box::new(
            ClientIntentionPacket::new(
                SharedConstants::get_protocol_version(),
                host_name.to_string(),
                port,
                intent,
            ),
        ));
        self.setup_outbound_protocol(outbound);
    }

    /// Sends the packet, for the listener `L` of the protocol it belongs to: at once on the
    /// event loop, and handed to it from any other thread. Dropped once the connection is closed,
    /// where Java queues it for a channel that never comes back.
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
        self.sent_packets.fetch_add(1, Ordering::Relaxed);
        let r#type = packet.r#type();
        let packet: Box<dyn Any + Send> = Box::new(packet);
        self.execute(move |connection| connection.do_send_packet(r#type, packet));
    }

    /// `writeAndFlush`, on the event loop: a packet that fails to encode goes to
    /// `exceptionCaught`.
    fn do_send_packet(&self, r#type: &PacketType, packet: Box<dyn Any + Send>) {
        let result = panic::catch_unwind(AssertUnwindSafe(|| self.write(packet)));
        if let Err(cause) = result {
            eprintln!("Error sending packet {type}");
            self.exception_caught(&panic_message(&*cause), false);
        }
    }

    /// The packet through the encoder, compression and the prepender, onto the socket. After a
    /// terminal packet the encoder is gone until the next protocol's is set up
    /// (`ProtocolSwapHandler.handleOutboundTerminalPacket`).
    fn write(&self, packet: Box<dyn Any + Send>) {
        let written = {
            let mut outbound = lock(&self.outbound);
            let Some(encoder) = outbound.encoder.clone() else {
                panic!("Pipeline has no outbound protocol configured, can't process packet");
            };
            let mut encoded = unpooled::buffer();
            if encoder.encode(packet, &mut encoded) {
                outbound.encoder = None;
            }
            if let Some(compress) = outbound.compress.as_mut() {
                let mut compressed = unpooled::buffer();
                compress.encode(&mut encoded, &mut compressed);
                encoded = compressed;
            }
            let mut frame = unpooled::buffer();
            outbound.prepender.encode(&encoded, &mut frame);
            match outbound.stream.as_mut() {
                Some(stream) => stream.write_all(&readable_bytes(&frame)),
                None => Ok(()),
            }
        };
        // Java's write future fires `exceptionCaught` on failure (`FIRE_EXCEPTION_ON_FAILURE`),
        // which logs and disconnects: a closed connection doesn't take the client down.
        if let Err(error) = written {
            eprintln!("Failed to send packet: {error}");
            self.disconnect(DisconnectionDetails::new(format!(
                "Internal Exception: {error}"
            )));
        }
    }

    pub fn tick(&self) {
        let listener = lock(&self.packet_listener).clone();
        if let Some(listener) = listener {
            listener.tick();
        }

        if !self.is_connected() && !self.disconnection_handled.load(Ordering::SeqCst) {
            self.handle_disconnection();
        }

        let tick_count = self.tick_count.fetch_add(1, Ordering::Relaxed);
        if tick_count % 20 == 0 {
            self.tick_second();
        }
    }

    fn tick_second(&self) {
        let sent = self.sent_packets.swap(0, Ordering::Relaxed);
        let received = self.received_packets.swap(0, Ordering::Relaxed);
        let mut average_sent = lock(&self.average_sent_packets);
        *average_sent = Mth::lerp_f32(
            Connection::AVERAGE_PACKETS_SMOOTHING,
            sent as f32,
            *average_sent,
        );
        let mut average_received = lock(&self.average_received_packets);
        *average_received = Mth::lerp_f32(
            Connection::AVERAGE_PACKETS_SMOOTHING,
            received as f32,
            *average_received,
        );
    }

    pub fn get_remote_address(&self) -> Option<SocketAddr> {
        self.address.get().copied().flatten()
    }

    /// Closes the connection, waiting for the event loop to, like Java's `awaitUninterruptibly`:
    /// what was sent before goes out first.
    pub fn disconnect(&self, details: DisconnectionDetails) {
        if self.channel.get().is_none() {
            *lock(&self.delayed_disconnect) = Some(details.clone());
        }

        if self.is_connected() {
            // Java sets the details once the channel is closed; here they're set as it closes,
            // so the game thread never finds the channel closed without them.
            let close = move |connection: &Connection| {
                if let Some(channel) = connection.channel.get()
                    && channel.open.load(Ordering::SeqCst)
                {
                    *lock(&connection.disconnection_details) = Some(details);
                    channel.open.store(false, Ordering::SeqCst);
                    let _ = channel.stream.shutdown(Shutdown::Both);
                }
            };
            self.execute_and_wait(close);
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
    pub fn connect_to_server(address: SocketAddr) -> std::io::Result<Arc<Connection>> {
        let connection = Arc::new(Connection::new(PacketFlow::CLIENTBOUND));
        Connection::connect(address, EventLoopGroupHolder::remote(), &connection)?;
        Ok(connection)
    }

    pub fn connect(
        address: SocketAddr,
        event_loop_group_holder: &EventLoopGroupHolder,
        connection: &Arc<Connection>,
    ) -> std::io::Result<()> {
        let stream = TcpStream::connect(address)?;
        let _ = stream.set_nodelay(true);
        {
            let mut outbound = lock(&connection.outbound);
            outbound.stream = Some(stream.try_clone()?);
            outbound.encoder = Some(Arc::new(OutboundProtocol {
                encoder: PacketEncoder::new(Box::new(&***HandshakeProtocols::SERVERBOUND)),
            }));
        }
        lock(&connection.inbound).last_read = Instant::now();

        // The channel is active before its event loop starts, which runs while it's open.
        let (events, received) = mpsc::channel();
        let reader = stream.try_clone()?;
        let _ = connection.channel.set(Channel {
            open: AtomicBool::new(true),
            stream,
            events: events.clone(),
            event_loop: OnceLock::new(),
        });
        let event_loop = {
            let connection = Arc::clone(connection);
            event_loop_group_holder.new_thread(move || connection.run_event_loop(received))?
        };
        if let Some(channel) = connection.channel.get() {
            let _ = channel.event_loop.set(event_loop.thread().id());
        }
        let name = format!("{} reader", event_loop.thread().name().unwrap_or("Netty"));
        thread::Builder::new()
            .name(name)
            .spawn(move || Connection::read_socket(reader, events))?;
        connection.channel_active();
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.channel
            .get()
            .is_some_and(|channel| channel.open.load(Ordering::SeqCst))
    }

    pub fn is_connecting(&self) -> bool {
        self.channel.get().is_none()
    }

    pub fn get_disconnection_details(&self) -> Option<DisconnectionDetails> {
        lock(&self.disconnection_details).clone()
    }

    pub fn set_read_only(&self) {
        self.set_auto_read(false);
    }

    fn set_auto_read(&self, auto_read: bool) {
        self.auto_read.store(auto_read, Ordering::SeqCst);
    }

    /// Turns compression on or off, or changes its threshold, on the event loop.
    pub fn setup_compression(&self, threshold: i32, validate_decompressed: bool) {
        self.execute_and_wait(move |connection| {
            let mut inbound = lock(&connection.inbound);
            let mut outbound = lock(&connection.outbound);
            if threshold >= 0 {
                match inbound.decompress.as_mut() {
                    Some(decompress) => decompress.set_threshold(threshold, validate_decompressed),
                    None => {
                        inbound.decompress =
                            Some(CompressionDecoder::new(threshold, validate_decompressed))
                    }
                }
                match outbound.compress.as_mut() {
                    Some(compress) => compress.set_threshold(threshold),
                    None => outbound.compress = Some(CompressionEncoder::new(threshold)),
                }
            } else {
                inbound.decompress = None;
                outbound.compress = None;
            }
        });
    }

    /// Tells the game thread's half of the listener that the connection ended, once: on the game
    /// thread, which ticks the connection.
    pub fn handle_disconnection(&self) {
        if self.channel.get().is_some() && !self.is_connected() {
            if self.disconnection_handled.swap(true, Ordering::SeqCst) {
                eprintln!("handleDisconnection() called twice");
            } else {
                let packet_listener = lock(&self.packet_listener).clone();
                let disconnect_listener =
                    packet_listener.or_else(|| lock(&self.disconnect_listener).clone());
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
        *lock(&self.average_received_packets)
    }

    pub fn get_average_sent_packets(&self) -> f32 {
        *lock(&self.average_sent_packets)
    }

    /// Whether the code running is this connection's event loop: Netty's `inEventLoop()`.
    fn in_event_loop(&self) -> bool {
        self.channel
            .get()
            .and_then(|channel| channel.event_loop.get())
            .is_some_and(|event_loop| *event_loop == thread::current().id())
    }

    /// Runs the task on the event loop: at once from the loop itself, and after what it was
    /// handed before from any other thread (Netty's `execute`). Dropped once the loop has ended.
    fn execute(&self, task: impl FnOnce(&Connection) + Send + 'static) {
        if self.in_event_loop() {
            task(self);
        } else if let Some(channel) = self.channel.get() {
            let _ = channel.events.send(Event::Task(Box::new(task)));
        }
    }

    /// `execute`, waiting for the task to have run. False when it never will: the loop has
    /// ended, the connection closed.
    fn execute_and_wait(&self, task: impl FnOnce(&Connection) + Send + 'static) -> bool {
        if self.in_event_loop() || self.channel.get().is_none() {
            task(self);
            return true;
        }
        let (done, ran) = mpsc::sync_channel(1);
        self.execute(move |connection| {
            task(connection);
            let _ = done.send(());
        });
        ran.recv().is_ok()
    }

    /// Reads the socket for the event loop until it closes or fails.
    fn read_socket(mut stream: TcpStream, events: Sender<Event>) {
        let mut bytes = vec![0u8; 65536];
        loop {
            let event = match stream.read(&mut bytes) {
                Ok(0) => Event::Inactive,
                Ok(read) => Event::Read(bytes[..read].to_vec()),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => Event::ReadFailed(error.to_string()),
            };
            let last = !matches!(event, Event::Read(_));
            if events.send(event).is_err() || last {
                return;
            }
        }
    }

    /// Netty's event loop for this connection, on its own thread, until the connection closes:
    /// what the socket read goes through the pipeline to the listener, the tasks other threads
    /// hand it run, and `ReadTimeoutHandler` closes the connection after 30 seconds of silence.
    fn run_event_loop(&self, events: Receiver<Event>) {
        if let Some(channel) = self.channel.get() {
            let _ = channel.event_loop.set(thread::current().id());
        }
        while self.is_connected() {
            let waited = lock(&self.inbound).last_read.elapsed();
            let timeout = Connection::READ_TIMEOUT
                .saturating_sub(waited)
                .max(Duration::from_millis(1));
            let event = match events.recv_timeout(timeout) {
                Ok(event) => event,
                Err(RecvTimeoutError::Timeout) => {
                    if lock(&self.inbound).last_read.elapsed() >= Connection::READ_TIMEOUT {
                        self.exception_caught("ReadTimeoutException", true);
                    }
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => return,
            };
            let result = panic::catch_unwind(AssertUnwindSafe(|| match event {
                Event::Read(bytes) => {
                    self.split(&bytes);
                    self.fire_frames();
                }
                Event::Inactive => self.channel_inactive(),
                Event::ReadFailed(cause) => self.exception_caught(&cause, false),
                Event::Task(task) => task(self),
            }));
            if let Err(cause) = result {
                self.exception_caught(&panic_message(&*cause), false);
            }
        }
    }

    /// The splitter, on what just came: Netty's cumulation and decode loop.
    fn split(&self, bytes: &[u8]) {
        let mut inbound = lock(&self.inbound);
        let inbound = &mut *inbound;
        inbound.last_read = Instant::now();
        let bytes: Vec<i8> = bytes.iter().map(|&byte| byte as i8).collect();
        inbound.cumulation.write_bytes_array(&bytes);

        let mut out = Vec::new();
        loop {
            let found = out.len();
            inbound.splitter.decode(&mut inbound.cumulation, &mut out);
            if !inbound.cumulation.is_readable() || out.len() == found {
                break;
            }
        }
        inbound.frames.extend(out);

        // What's been read goes, like Netty's `discardSomeReadBytes`.
        let readable = inbound.cumulation.readable_bytes();
        let mut rest = unpooled::buffer_initial_capacity(readable.max(256));
        rest.write_bytes_buf(&mut inbound.cumulation);
        inbound.cumulation = rest;
    }

    /// Decompresses, decodes and hands on the frames that have arrived, while there's a decoder
    /// to read them and reading is on.
    fn fire_frames(&self) {
        if self.firing.swap(true, Ordering::SeqCst) {
            return;
        }
        loop {
            if !self.is_connected() || !self.auto_read.load(Ordering::SeqCst) {
                break;
            }
            let decoder = lock(&self.decoder).clone();
            let Some(decoder) = decoder else {
                break;
            };
            let Some(frame) = lock(&self.inbound).frames.pop_front() else {
                break;
            };

            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                let mut frame = frame;
                let mut inbound = lock(&self.inbound);
                if let Some(decompress) = inbound.decompress.as_mut() {
                    let mut out = Vec::new();
                    decompress.decode(&mut frame, &mut out);
                    frame = out.pop().expect("a frame decompresses to one packet");
                }
                drop(inbound);
                self.channel_read(decoder, frame);
            }));
            if let Err(cause) = result {
                // `channelRead0`'s, when the game thread takes no more packets.
                let message = panic_message(&*cause);
                if message.starts_with("RejectedExecutionException") {
                    self.disconnect(DisconnectionDetails::new(
                        "multiplayer.disconnect.server_shutdown",
                    ));
                } else {
                    self.exception_caught(&message, false);
                }
            }
        }
        self.firing.store(false, Ordering::SeqCst);
    }
}

/// A buffer's readable bytes, for the socket.
fn readable_bytes(buffer: &ByteBuf) -> Vec<u8> {
    let length = buffer.readable_bytes();
    let mut bytes = vec![0i8; length as usize];
    buffer.get_bytes_array(buffer.reader_index(), &mut bytes, 0, length);
    bytes.into_iter().map(|byte| byte as u8).collect()
}
