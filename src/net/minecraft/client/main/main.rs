//! net/minecraft/client/main/Main.java
//!
//! Not ported yet: most options (the window's size and fullscreen, the game, assets and resource
//! pack directories, the proxy, demo mode, the graphics backend, profiling, quick play into a
//! singleplayer world or a realm, and the quick play log), bootstrapping the game's registries,
//! native libraries and the crash reports. `--accessToken` and `--version`, which Java requires,
//! are optional: the port only logs in offline so far.
//!
//! Options the port adds, which vanilla doesn't have:
//! - `--server HOST:PORT`, the same as `--quickPlayMultiplayer HOST:PORT`;
//! - `--headless`, no window: the client renders only to take a scenario's screenshots, offscreen.
//!   Without it, the client opens a window (`--window` says so too) and renders a frame of the
//!   level each time round its loop, like vanilla;
//! - `--oracleScenario PATH`, which replays a scenario on the server the way the oracle client
//!   does (`OracleGameTest`), and exits 0 when it finished, like `rustcraft.oracle.scenario`,
//!   writing its trace into `--oracleOut DIR` (`target/oracle` by default), like
//!   `rustcraft.oracle.out`;
//! - `--ping HOST:PORT`, which pings the server the way the server list does
//!   (`ServerStatusPinger`), prints its status's JSON and the round trip, and exits 0 when the
//!   pong came, without joining;
//! - `--stall SECONDS`, which stalls the game thread that long once the level is ready, and exits
//!   0 when the connection held (`StallTest`).

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::com::mojang::blaze3d::platform::window::Window;
use crate::com::mojang::blaze3d::systems::render_system::RenderSystem;
use crate::com::mojang::renderpearl::api::device::gpu_backend::GpuBackend;
use crate::com::mojang::renderpearl::api::device::gpu_debug_options::GpuDebugOptions;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::backend::opengl::gl_backend::GlBackend;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::client::gui::screens::connect_screen::ConnectScreen;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::resolver::server_address::ServerAddress;
use crate::net::minecraft::client::multiplayer::server_data::{self, ServerData};
use crate::net::minecraft::client::multiplayer::server_status_pinger::ServerStatusPinger;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::renderer::game_renderer::GameRenderer;
use crate::net::minecraft::client::user::User;
use crate::net::minecraft::core::uuid_util::UUIDUtil;
use crate::net::minecraft::network::packet_processor::lock;
use crate::net::minecraft::network::protocol::unported::Unported;
use crate::rustcraft::oracle::client::oracle_game_test::{OracleGameTest, Outcome};
use crate::rustcraft::oracle::client::scenario::Scenario;
use crate::rustcraft::stall_test::StallTest;

/// The client's entry point.
pub struct Main;

/// The options `main` takes, joptsimple's `OptionSet`: each `--name value`, or `--name` alone.
struct OptionSet {
    values: Vec<(String, Option<String>)>,
}

impl OptionSet {
    /// The options that take a value, which is the next argument.
    const WITH_ARGUMENT: [&'static str; 12] = [
        "username",
        "uuid",
        "accessToken",
        "version",
        "quickPlayMultiplayer",
        "server",
        "xuid",
        "clientId",
        "oracleScenario",
        "oracleOut",
        "ping",
        "stall",
    ];

    fn parse(args: &[String]) -> Result<OptionSet, String> {
        let mut values = Vec::new();
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            let Some(name) = arg.strip_prefix("--") else {
                // joptsimple's non-options, which Java logs and ignores.
                eprintln!("Completely ignored arguments: [{arg}]");
                continue;
            };
            let (name, value) = match name.split_once('=') {
                Some((name, value)) => (name.to_string(), Some(value.to_string())),
                None if OptionSet::WITH_ARGUMENT.contains(&name) => {
                    let value = args
                        .next()
                        .ok_or(format!("Option {name} requires an argument"))?;
                    (name.to_string(), Some(value.clone()))
                }
                None => (name.to_string(), None),
            };
            values.push((name, value));
        }
        Ok(OptionSet { values })
    }

    fn has(&self, name: &str) -> bool {
        self.values.iter().any(|(option, _)| option == name)
    }

    fn value_of(&self, name: &str) -> Option<&str> {
        self.values
            .iter()
            .rev()
            .find(|(option, _)| option == name)
            .and_then(|(_, value)| value.as_deref())
    }
}

impl Main {
    /// Runs the client with the command line's arguments, and returns the exit status.
    pub fn main(args: &[String]) -> i32 {
        let option_set = match OptionSet::parse(args) {
            Ok(option_set) => option_set,
            Err(message) => {
                eprintln!("{message}");
                return -5;
            }
        };

        let username = option_set
            .value_of("username")
            .map(str::to_string)
            .unwrap_or_else(|| {
                let millis = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|time| time.as_millis())
                    .unwrap_or_default();
                format!("Player{}", millis % 1000)
            });
        let uuid = option_set
            .value_of("uuid")
            .and_then(Main::parse_undashed_uuid)
            .unwrap_or_else(|| UUIDUtil::create_offline_player_uuid(&username));
        let user = User::new(
            username,
            uuid,
            option_set
                .value_of("accessToken")
                .unwrap_or("0")
                .to_string(),
            option_set
                .value_of("xuid")
                .filter(|xuid| !xuid.is_empty())
                .map(str::to_string),
            option_set
                .value_of("clientId")
                .filter(|client_id| !client_id.is_empty())
                .map(str::to_string),
        );

        if let Some(address) = option_set.value_of("ping") {
            return Main::ping(address);
        }

        let Some(server_address) = option_set
            .value_of("quickPlayMultiplayer")
            .or(option_set.value_of("server"))
            .map(str::to_string)
        else {
            eprintln!(
                "Nothing to do: the port has no title screen yet, so it needs a server to join \
                 (--server HOST:PORT or --quickPlayMultiplayer HOST:PORT)"
            );
            return 2;
        };
        let windowed = option_set.has("window") || !option_set.has("headless");

        let minecraft = Minecraft::new(user, Options::new());
        let mut window_size = (854, 480);
        let mut needs_renderer = windowed;
        let mut outcome = None;
        let mut server_data = ServerData::new(
            "Minecraft Server".to_string(),
            server_address.clone(),
            server_data::Type::OTHER,
        );
        if let Some(path) = option_set.value_of("oracleScenario") {
            let scenario = match Scenario::load(Path::new(path)) {
                Ok(scenario) => scenario,
                Err(message) => {
                    eprintln!("{message}");
                    return 2;
                }
            };
            let out_dir = Path::new(option_set.value_of("oracleOut").unwrap_or("target/oracle"));
            window_size = (scenario.options.width, scenario.options.height);
            needs_renderer |= OracleGameTest::has_screenshots(&scenario);
            let (driver, driver_outcome) = OracleGameTest::new(&minecraft, scenario, out_dir);
            minecraft.set_test_driver(Box::new(driver));
            outcome = Some(driver_outcome);
            server_data.name = "rustcraft oracle".to_string();
        } else if let Some(seconds) = option_set.value_of("stall") {
            let Ok(seconds) = seconds.parse::<f64>() else {
                eprintln!("--stall takes a number of seconds, not {seconds}");
                return 2;
            };
            let (driver, driver_outcome) = StallTest::new(Duration::from_secs_f64(seconds));
            minecraft.set_test_driver(Box::new(driver));
            outcome = Some(driver_outcome);
        }

        if needs_renderer {
            // A client that can't render still joins: its screenshots are what's missing.
            if let Err(error) = Main::init_renderer(&minecraft, windowed, window_size) {
                eprintln!("Can't render: {error}");
            }
        }

        ConnectScreen::start_connecting(
            &minecraft,
            &ServerAddress::parse_string(Some(&server_address)),
            server_data,
        );
        minecraft.run();
        // What the client skipped, for the soak's report (`Unported`).
        for (r#type, count) in Unported::counts() {
            println!("Unported packet {type}: {count}");
        }

        match outcome.map(|outcome| outcome.take()) {
            Some(Some(Outcome::Finished)) => 0,
            Some(Some(Outcome::Failed(_))) => 1,
            _ => match minecraft.disconnection() {
                Some((title, details)) => {
                    eprintln!("{title}: {}", details.reason_text());
                    1
                }
                None => 0,
            },
        }
    }

    /// `Minecraft`'s window and renderer: SDL's video and the GL library, the device (which lives
    /// as long as the game, like Java's `RenderSystem` device), the window if there's to be one,
    /// and the game renderer with the vanilla pack's atlas and models.
    fn init_renderer(
        minecraft: &Minecraft,
        windowed: bool,
        (width, height): (i32, i32),
    ) -> Result<(), String> {
        RenderSystem::init_backend_system();
        let mut backend = GlBackend::new();
        backend.load_library().map_err(|error| error.to_string())?;
        let device: &'static dyn GpuDevice = Box::leak(
            backend
                .create_device(&GpuDebugOptions::default())
                .map_err(|error| error.to_string())?,
        );
        let window = windowed.then(|| {
            let window = Window::new(&backend, width, height, "Minecraft* 26.3 (rustcraft)");
            let surface = device.create_surface(window.handle());
            (window, surface)
        });
        let (width, height) = window.as_ref().map_or((width, height), |(window, _)| {
            (window.get_width(), window.get_height())
        });
        minecraft.init_renderer(GameRenderer::new(device, width, height), window);
        Ok(())
    }

    /// `--ping`: what the server list does for one server, ticking the pinger until the pong
    /// comes, the connection ends, or 30 seconds (Netty's read timeout) pass.
    fn ping(address: &str) -> i32 {
        let data = Arc::new(Mutex::new(ServerData::new(
            "Minecraft Server".to_string(),
            address.to_string(),
            server_data::Type::OTHER,
        )));
        let pinger = ServerStatusPinger::new();
        let ponged = Arc::new(AtomicBool::new(false));
        let on_pong = Arc::clone(&ponged);
        if let Err(error) = pinger.ping_server(
            Arc::clone(&data),
            Box::new(move || on_pong.store(true, Ordering::SeqCst)),
        ) {
            eprintln!("Can't ping {address}: {error}");
            return 1;
        }

        let deadline = Instant::now() + Duration::from_secs(30);
        while pinger.is_pinging() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
            pinger.tick();
        }
        pinger.remove_all();

        let data = lock(&data);
        match (&data.server_status, ponged.load(Ordering::SeqCst)) {
            (Some(status), true) => {
                println!("{}", status.json());
                println!("ping: {} ms", data.ping);
                0
            }
            _ => 1,
        }
    }

    /// `UndashedUuid.fromStringLenient`: a UUID with or without its dashes.
    fn parse_undashed_uuid(text: &str) -> Option<UUID> {
        let hex: String = text.chars().filter(|&c| c != '-').collect();
        if hex.len() != 32 {
            return None;
        }
        let most = u64::from_str_radix(&hex[..16], 16).ok()? as i64;
        let least = u64::from_str_radix(&hex[16..], 16).ok()? as i64;
        Some(UUID::new(most, least))
    }
}
