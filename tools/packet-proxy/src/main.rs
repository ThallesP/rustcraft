//! Sits between a Minecraft client and a vanilla server in offline mode, forwards every byte
//! unchanged and records each packet to a JSON Lines capture (format in TESTING.md).

use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use packet_proxy::frame::{FrameReader, decode_packet};
use packet_proxy::packets::{Direction, PacketTable, State};
use packet_proxy::session::Session;

const USAGE: &str = "usage: packet-proxy --packets <packets.json> --out <capture.jsonl> [--listen 127.0.0.1:25566] [--upstream 127.0.0.1:25565] [--once]

  --packets   the data generator's reports/packets.json, for packet names
  --out       capture file to write, one JSON line per packet
  --listen    address clients connect to (default 127.0.0.1:25566)
  --upstream  vanilla server to forward to (default 127.0.0.1:25565)
  --once      exit after the first connection closes";

struct Args {
    listen: String,
    upstream: String,
    packets: PathBuf,
    out: PathBuf,
    once: bool,
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut listen = "127.0.0.1:25566".to_string();
    let mut upstream = "127.0.0.1:25565".to_string();
    let mut packets = None;
    let mut out = None;
    let mut once = false;

    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--listen" => listen = value()?,
            "--upstream" => upstream = value()?,
            "--packets" => packets = Some(PathBuf::from(value()?)),
            "--out" => out = Some(PathBuf::from(value()?)),
            "--once" => once = true,
            _ => return Err(format!("unknown argument {arg}")),
        }
    }

    Ok(Args {
        listen,
        upstream,
        packets: packets.ok_or("--packets is required")?,
        out: out.ok_or("--out is required")?,
        once,
    })
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };

    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("packet-proxy: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> io::Result<()> {
    let table = Arc::new(PacketTable::load(&args.packets)?);
    let capture = Arc::new(Capture::create(&args.out)?);
    let listener = TcpListener::bind(&args.listen)?;
    eprintln!(
        "packet-proxy: {} -> {}, capturing to {}",
        args.listen,
        args.upstream,
        args.out.display()
    );

    for (connection, client) in listener.incoming().enumerate() {
        let client = client?;
        let upstream = args.upstream.clone();
        let (table, capture) = (table.clone(), capture.clone());
        let handle = thread::spawn(move || {
            if let Err(error) = proxy_connection(connection, client, &upstream, table, capture) {
                eprintln!("packet-proxy: connection {connection}: {error}");
            }
        });

        if args.once {
            handle.join().expect("connection thread panicked");
            break;
        }
    }
    Ok(())
}

fn proxy_connection(
    connection: usize,
    client: TcpStream,
    upstream: &str,
    table: Arc<PacketTable>,
    capture: Arc<Capture>,
) -> io::Result<()> {
    let server = TcpStream::connect(upstream)?;
    client.set_nodelay(true)?;
    server.set_nodelay(true)?;
    eprintln!(
        "packet-proxy: connection {connection} from {}",
        client.peer_addr()?
    );

    let session = Arc::new(Mutex::new(Session::default()));
    let pump = |direction| Pump {
        connection,
        direction,
        session: session.clone(),
        table: table.clone(),
        capture: capture.clone(),
    };

    let clientbound = pump(Direction::Clientbound);
    let (server_reader, client_writer) = (server.try_clone()?, client.try_clone()?);
    let clientbound = thread::spawn(move || clientbound.run(server_reader, client_writer));
    let serverbound = pump(Direction::Serverbound).run(client, server);

    let clientbound = clientbound.join().expect("clientbound pump panicked");
    serverbound.and(clientbound)
}

/// Copies one direction of a connection, recording each packet before forwarding the bytes that
/// complete it, so a state change is applied before the other side can answer the packet that
/// caused it.
struct Pump {
    connection: usize,
    direction: Direction,
    session: Arc<Mutex<Session>>,
    table: Arc<PacketTable>,
    capture: Arc<Capture>,
}

impl Pump {
    fn run(&self, mut from: TcpStream, mut to: TcpStream) -> io::Result<()> {
        let result = self.forward(&mut from, &mut to);
        match result {
            Ok(()) => {
                let _ = to.shutdown(Shutdown::Write);
            }
            // Closing both sockets unblocks the pump going the other way.
            Err(_) => {
                let _ = from.shutdown(Shutdown::Both);
                let _ = to.shutdown(Shutdown::Both);
            }
        }
        result
    }

    fn forward(&self, from: &mut TcpStream, to: &mut TcpStream) -> io::Result<()> {
        let mut frames = FrameReader::default();
        let mut buffer = vec![0; 64 * 1024];
        loop {
            let read = match from.read(&mut buffer) {
                Ok(0) => return Ok(()),
                Ok(read) => read,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if is_disconnect(&error) => return Ok(()),
                Err(error) => return Err(error),
            };

            frames.push(&buffer[..read]);
            while let Some(frame) = frames.next_frame()? {
                self.record(&frame)?;
            }

            match to.write_all(&buffer[..read]) {
                Err(error) if is_disconnect(&error) => return Ok(()),
                result => result?,
            }
        }
    }

    fn record(&self, frame: &[u8]) -> io::Result<()> {
        let mut session = self.session.lock().expect("session lock poisoned");
        let state = session.state(self.direction);
        let (id, payload) = decode_packet(frame, session.compressed)?;
        let name = self.table.name(state, self.direction, id);
        self.capture
            .write(self.connection, self.direction, state, id, name, &payload)?;
        session.after_packet(self.direction, state, name, &payload)
    }
}

fn is_disconnect(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::NotConnected
    )
}

struct Capture {
    output: Mutex<(BufWriter<File>, u64)>,
    started: Instant,
}

impl Capture {
    fn create(path: &PathBuf) -> io::Result<Self> {
        Ok(Self {
            output: Mutex::new((BufWriter::new(File::create(path)?), 0)),
            started: Instant::now(),
        })
    }

    fn write(
        &self,
        connection: usize,
        direction: Direction,
        state: State,
        id: i32,
        name: Option<&str>,
        payload: &[u8],
    ) -> io::Result<()> {
        let mut output = self.output.lock().expect("capture lock poisoned");
        let (writer, seq) = &mut *output;
        let name = serde_json::to_string(&name).expect("names serialize");
        writeln!(
            writer,
            r#"{{"seq":{seq},"connection":{connection},"ms":{},"direction":"{}","state":"{}","id":{id},"name":{name},"payload":"{}"}}"#,
            self.started.elapsed().as_millis(),
            direction.name(),
            state.name(),
            hex(payload),
        )?;
        *seq += 1;
        // Flushed per packet so a killed proxy still leaves a complete capture behind.
        writer.flush()
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push(DIGITS[usize::from(byte >> 4)] as char);
        hex.push(DIGITS[usize::from(byte & 0xf)] as char);
    }
    hex
}
