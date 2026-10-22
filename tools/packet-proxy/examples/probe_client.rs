//! A minimal client for testing the proxy against a vanilla server in offline mode: a status ping,
//! or a login that answers configuration like the vanilla client, then stays in play answering
//! keep-alives and teleports.
//!
//!     cargo run -p packet-proxy --example probe_client -- --packets <packets.json> [--address 127.0.0.1:25566] [--status] [--name rustcraft] [--stay 5]

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use packet_proxy::frame::{
    FrameReader, decode_packet, encode_frame, invalid, read_var_int, write_var_int,
};
use packet_proxy::packets::{Direction, PacketTable, State};

// Matches resources/version.json of the pinned version.
const PROTOCOL_VERSION: i32 = 777;
const INTENTION_STATUS: i32 = 1;
const INTENTION_LOGIN: i32 = 2;

struct Args {
    address: String,
    packets: PathBuf,
    status: bool,
    name: String,
    stay: Duration,
}

fn parse_args() -> Result<Args, String> {
    let mut args = std::env::args().skip(1);
    let mut address = "127.0.0.1:25566".to_string();
    let mut packets = None;
    let mut status = false;
    let mut name = "rustcraft".to_string();
    let mut stay = Duration::from_secs(5);

    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--address" => address = value()?,
            "--packets" => packets = Some(PathBuf::from(value()?)),
            "--status" => status = true,
            "--name" => name = value()?,
            "--stay" => {
                stay =
                    Duration::from_secs_f64(value()?.parse().map_err(|_| "--stay takes seconds")?)
            }
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    Ok(Args {
        address,
        packets: packets.ok_or("--packets is required")?,
        status,
        name,
        stay,
    })
}

fn main() -> io::Result<()> {
    let args = parse_args().map_err(io::Error::other)?;
    let table = PacketTable::load(&args.packets)?;
    let mut connection = Connection::open(&args.address, table)?;

    if args.status {
        status(&mut connection)
    } else {
        login(&mut connection, &args.name)?;
        configure(&mut connection)?;
        play(&mut connection, args.stay)
    }
}

fn status(connection: &mut Connection) -> io::Result<()> {
    connection.send_intention(INTENTION_STATUS)?;
    connection.state = State::Status;
    connection.send("minecraft:status_request", &[])?;
    let (_, response) = connection.expect("minecraft:status_response")?;
    println!("status: {}", read_string(&response)?);

    // The vanilla client pings with its clock, so captures of two runs differ here.
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before 1970")
        .as_millis() as i64;
    connection.send("minecraft:ping_request", &millis.to_be_bytes())?;
    connection.expect("minecraft:pong_response")?;
    println!("pong");
    Ok(())
}

fn login(connection: &mut Connection, name: &str) -> io::Result<()> {
    connection.send_intention(INTENTION_LOGIN)?;
    connection.state = State::Login;

    // Offline servers derive the player's UUID from the name and ignore this one.
    let mut hello = Vec::new();
    write_string(name, &mut hello);
    hello.extend_from_slice(&[0; 16]);
    connection.send("minecraft:hello", &hello)?;

    loop {
        let (packet, payload) = connection.receive()?;
        match packet.as_str() {
            "minecraft:login_compression" => {
                let (threshold, _) =
                    read_var_int(&payload)?.ok_or_else(|| invalid("empty login_compression"))?;
                connection.compression = usize::try_from(threshold).ok();
            }
            "minecraft:login_finished" => break,
            "minecraft:login_disconnect" => {
                return Err(io::Error::other(format!(
                    "disconnected: {}",
                    String::from_utf8_lossy(&payload)
                )));
            }
            "minecraft:hello" => {
                return Err(io::Error::other(
                    "the server wants encryption: run it with online-mode=false",
                ));
            }
            other => return Err(io::Error::other(format!("unexpected {other} during login"))),
        }
    }

    connection.send("minecraft:login_acknowledged", &[])?;
    connection.state = State::Configuration;
    println!("logged in as {name}");
    Ok(())
}

fn configure(connection: &mut Connection) -> io::Result<()> {
    connection.send("minecraft:client_information", &client_information())?;

    loop {
        let (packet, payload) = connection.receive()?;
        match packet.as_str() {
            // Echoing the server's packs back says we know them all, like a vanilla client does.
            "minecraft:select_known_packs" => {
                connection.send("minecraft:select_known_packs", &payload)?
            }
            "minecraft:keep_alive" => connection.send("minecraft:keep_alive", &payload)?,
            "minecraft:ping" => connection.send("minecraft:pong", &payload)?,
            "minecraft:code_of_conduct" => {
                connection.send("minecraft:accept_code_of_conduct", &[])?
            }
            "minecraft:disconnect" => {
                return Err(io::Error::other("disconnected during configuration"));
            }
            "minecraft:finish_configuration" => break,
            _ => {}
        }
    }

    connection.send("minecraft:finish_configuration", &[])?;
    connection.state = State::Play;
    println!("configured");
    Ok(())
}

fn play(connection: &mut Connection, stay: Duration) -> io::Result<()> {
    let deadline = Instant::now() + stay;
    let mut received = 0;
    while Instant::now() < deadline {
        let Some((packet, payload)) = connection.receive_until(deadline)? else {
            break;
        };
        received += 1;
        match packet.as_str() {
            "minecraft:keep_alive" => connection.send("minecraft:keep_alive", &payload)?,
            "minecraft:ping" => connection.send("minecraft:pong", &payload)?,
            "minecraft:player_position" => {
                connection.send(
                    "minecraft:accept_teleportation",
                    &accept_teleportation(&payload)?,
                )?;
                println!("accepted teleport");
            }
            "minecraft:chunk_batch_finished" => {
                connection.send("minecraft:chunk_batch_received", &64.0f32.to_be_bytes())?
            }
            "minecraft:disconnect" => return Err(io::Error::other("disconnected during play")),
            _ => {}
        }
    }
    println!("left play after {received} packets");
    Ok(())
}

// player_position is a teleport id, position, delta movement (three doubles each), y_rot, x_rot and
// relative flags. The answer repeats the id, position and rotation the client ended up at; the
// probe treats every teleport as absolute, which the first one always is.
fn accept_teleportation(player_position: &[u8]) -> io::Result<Vec<u8>> {
    let (_, id_length) =
        read_var_int(player_position)?.ok_or_else(|| invalid("empty player_position"))?;
    let truncated = || invalid("truncated player_position");
    let position = player_position
        .get(id_length..id_length + 24)
        .ok_or_else(truncated)?;
    let rotation = player_position
        .get(id_length + 48..id_length + 56)
        .ok_or_else(truncated)?;
    Ok([&player_position[..id_length], position, rotation].concat())
}

// ClientInformation: language, view distance, chat visibility, chat colors, skin parts, main hand,
// text filtering, allows listing, particles.
fn client_information() -> Vec<u8> {
    let mut payload = Vec::new();
    write_string("en_us", &mut payload);
    payload.push(4);
    write_var_int(0, &mut payload);
    payload.push(1);
    payload.push(0x7f);
    write_var_int(1, &mut payload);
    payload.push(0);
    payload.push(1);
    write_var_int(0, &mut payload);
    payload
}

fn write_string(value: &str, output: &mut Vec<u8>) {
    write_var_int(value.len() as i32, output);
    output.extend_from_slice(value.as_bytes());
}

fn read_string(payload: &[u8]) -> io::Result<String> {
    let (length, prefix) = read_var_int(payload)?.ok_or_else(|| invalid("truncated string"))?;
    let bytes = payload
        .get(prefix..prefix + length as usize)
        .ok_or_else(|| invalid("truncated string"))?;
    String::from_utf8(bytes.to_vec()).map_err(|error| invalid(error.to_string()))
}

struct Connection {
    stream: TcpStream,
    frames: FrameReader,
    table: PacketTable,
    state: State,
    compression: Option<usize>,
}

impl Connection {
    fn open(address: &str, table: PacketTable) -> io::Result<Self> {
        let stream = TcpStream::connect(address)?;
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            frames: FrameReader::default(),
            table,
            state: State::Handshake,
            compression: None,
        })
    }

    fn send_intention(&mut self, intention: i32) -> io::Result<()> {
        let (host, port) = self
            .stream
            .peer_addr()
            .map(|address| (address.ip().to_string(), address.port()))?;
        let mut payload = Vec::new();
        write_var_int(PROTOCOL_VERSION, &mut payload);
        write_string(&host, &mut payload);
        payload.extend_from_slice(&port.to_be_bytes());
        write_var_int(intention, &mut payload);
        self.send("minecraft:intention", &payload)
    }

    fn send(&mut self, name: &str, payload: &[u8]) -> io::Result<()> {
        let id = self
            .table
            .id(self.state, Direction::Serverbound, name)
            .ok_or_else(|| invalid(format!("no serverbound {name} in {}", self.state.name())))?;
        self.stream
            .write_all(&encode_frame(id, payload, self.compression))
    }

    fn expect(&mut self, name: &str) -> io::Result<(String, Vec<u8>)> {
        let (packet, payload) = self.receive()?;
        if packet != name {
            return Err(io::Error::other(format!("expected {name}, got {packet}")));
        }
        Ok((packet, payload))
    }

    fn receive(&mut self) -> io::Result<(String, Vec<u8>)> {
        self.stream
            .set_read_timeout(Some(Duration::from_secs(30)))?;
        self.next_packet()?
            .ok_or_else(|| io::Error::other("timed out waiting for the server"))
    }

    fn receive_until(&mut self, deadline: Instant) -> io::Result<Option<(String, Vec<u8>)>> {
        let left = deadline
            .saturating_duration_since(Instant::now())
            .max(Duration::from_millis(1));
        self.stream.set_read_timeout(Some(left))?;
        self.next_packet()
    }

    fn next_packet(&mut self) -> io::Result<Option<(String, Vec<u8>)>> {
        let mut buffer = [0; 64 * 1024];
        loop {
            if let Some(frame) = self.frames.next_frame()? {
                let (id, payload) = decode_packet(&frame, self.compression.is_some())?;
                let name = self
                    .table
                    .name(self.state, Direction::Clientbound, id)
                    .unwrap_or("unknown")
                    .to_string();
                return Ok(Some((name, payload)));
            }

            match self.stream.read(&mut buffer) {
                Ok(0) => return Err(io::Error::other("the server closed the connection")),
                Ok(read) => self.frames.push(&buffer[..read]),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    return Ok(None);
                }
                Err(error) => return Err(error),
            }
        }
    }
}
