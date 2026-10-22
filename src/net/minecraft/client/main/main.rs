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
//! - `--headless`, no window. There is no window yet (the render track brings it), so every run
//!   is headless for now;
//! - `--oracleScenario PATH`, which replays a scenario on the server the way the oracle client
//!   does (`OracleGameTest`), and exits 0 when it finished, like `rustcraft.oracle.scenario`.

use std::path::Path;

use crate::java::util::uuid::UUID;
use crate::net::minecraft::client::gui::screens::connect_screen::ConnectScreen;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::resolver::server_address::ServerAddress;
use crate::net::minecraft::client::multiplayer::server_data::{self, ServerData};
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::user::User;
use crate::net::minecraft::core::uuid_util::UUIDUtil;
use crate::rustcraft::oracle::client::oracle_game_test::{OracleGameTest, Outcome};
use crate::rustcraft::oracle::client::scenario::Scenario;

/// The client's entry point.
pub struct Main;

/// The options `main` takes, joptsimple's `OptionSet`: each `--name value`, or `--name` alone.
struct OptionSet {
    values: Vec<(String, Option<String>)>,
}

impl OptionSet {
    /// The options that take a value, which is the next argument.
    const WITH_ARGUMENT: [&'static str; 9] = [
        "username",
        "uuid",
        "accessToken",
        "version",
        "quickPlayMultiplayer",
        "server",
        "xuid",
        "clientId",
        "oracleScenario",
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
        if !option_set.has("headless") {
            println!("There's no window yet, so the client runs headless");
        }

        let minecraft = Minecraft::new(user, Options::new());
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
            let (driver, driver_outcome) = OracleGameTest::new(&minecraft, scenario);
            minecraft.set_test_driver(Box::new(driver));
            outcome = Some(driver_outcome);
            server_data.name = "rustcraft oracle".to_string();
        }

        ConnectScreen::start_connecting(
            &minecraft,
            &ServerAddress::parse_string(Some(&server_address)),
            server_data,
        );
        minecraft.run();

        match outcome.map(|outcome| outcome.take()) {
            Some(Some(Outcome::Finished)) => 0,
            Some(Some(Outcome::Failed(_))) => 1,
            _ => match minecraft.disconnection() {
                Some((title, details)) => {
                    eprintln!("{title}: {}", details.reason());
                    1
                }
                None => 0,
            },
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
