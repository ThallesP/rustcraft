//! oracle/client/src/main/java/rustcraft/oracle/client/Scenario.java: the input both clients
//! replay, in the format TESTING.md documents.

use std::path::Path;

use serde_json::Value;

/// A scenario, as the oracle client reads it.
#[derive(Clone, Debug)]
pub struct Scenario {
    pub name: String,
    pub world: World,
    pub player: Player,
    pub options: Options,
    /// Commands that build the scenario's course, without their slash, sent before the player
    /// is put in place.
    pub setup: Vec<String>,
    /// Steps on the title screen before the client joins (TESTING.md): ticks, screenshots, the
    /// cursor.
    pub title: Vec<Step>,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug)]
pub struct World {
    pub seed: i64,
    pub preset: String,
    pub time: i64,
    pub game_mode: String,
    /// The server's ticks a second, 0 for its own 20 and an unpaced client (`ServerTicks`).
    pub tick_rate: f32,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub width: i32,
    pub height: i32,
    pub fov: i32,
    pub gui_scale: i32,
    pub render_distance: i32,
    /// Vanilla's F1 (`Hud.toggle`): no HUD, hand or crosshair.
    pub hide_gui: bool,
    /// The accessibility option `hideSplashTexts`, for a scenario that shows the title screen,
    /// whose splash is random.
    pub hide_splash: bool,
}

/// Either a screenshot, a key pressed and released (`key`), characters typed (`type`), the
/// cursor moved to a place in the window (`cursor`), a mouse button clicked (`click`), the wheel
/// scrolled (`scroll`) or a command sent (`command`, without its slash) between two ticks, or `ticks` ticks with `keys` held and an optional fixed `look`, and
/// optionally clientbound packets to wait for after its last tick, or after its input
/// (`wait_for`, TESTING.md), and a screenshot after every `screenshots_every`th of its ticks.
#[derive(Clone, Debug, Default)]
pub struct Step {
    pub ticks: i32,
    pub keys: Vec<String>,
    pub look: Option<[f32; 2]>,
    pub screenshot: Option<String>,
    /// Clientbound packet types (`minecraft:set_health`) that must each have come since the step
    /// began before its last tick's trace line, or, for input, before the next tick; the client
    /// handles them then.
    pub wait_for: Vec<String>,
    pub key: Option<String>,
    pub r#type: Option<String>,
    pub cursor: Option<[f64; 2]>,
    pub command: Option<String>,
    /// A mouse button pressed and released where the cursor is (`MOUSE_BUTTONS`).
    pub click: Option<String>,
    /// The mouse wheel scrolled by this much where the cursor is.
    pub scroll: Option<f64>,
    /// A screenshot taken just after the focused edit box's cursor has come on (TESTING.md).
    pub blink_on: bool,
    /// A screenshot after every so many of the step's ticks, each named by the tick it comes after
    /// (`Scenario::tick_screenshot`); 0 for none.
    pub screenshots_every: i32,
}

impl Step {
    /// Whether it's input between two ticks (a key, characters, the cursor, a click, the wheel or
    /// a command).
    pub fn is_input(&self) -> bool {
        self.key.is_some()
            || self.r#type.is_some()
            || self.cursor.is_some()
            || self.click.is_some()
            || self.scroll.is_some()
            || self.command.is_some()
    }
}

impl Scenario {
    pub const KEYS: [&'static str; 9] = [
        "forward", "back", "left", "right", "jump", "sneak", "sprint", "attack", "use",
    ];

    /// The keys a `key` step presses, by their SDL scancodes (`InputConstants`' `KEY_` constants).
    pub const PRESSABLE_KEYS: [(&'static str, i32); 6] = [
        ("escape", 41),
        ("enter", 40),
        ("t", 23),
        ("slash", 56),
        ("tab", 43),
        ("backspace", 42),
    ];

    /// The buttons a `click` step presses, by their SDL numbers (`InputConstants`'
    /// `MOUSE_BUTTON_` constants).
    pub const MOUSE_BUTTONS: [(&'static str, i32); 1] = [("left", 1)];

    /// `Scenario.tickScreenshot`: the name of the screenshot taken after the scenario's tick (its
    /// trace line's `tick`).
    pub fn tick_screenshot(tick: i32) -> String {
        format!("tick-{tick:04}")
    }

    pub fn load(path: &Path) -> Result<Scenario, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let root: Value =
            serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = file_name
            .strip_suffix(".json")
            .unwrap_or(&file_name)
            .to_string();

        let field = |value: &Value, key: &str| -> Result<Value, String> {
            value
                .get(key)
                .cloned()
                .ok_or_else(|| format!("{}: missing {key}", path.display()))
        };
        let number = |value: Value| value.as_f64().ok_or("expected a number".to_string());
        let world = field(&root, "world")?;
        let player = field(&root, "player")?;
        let pos = field(&player, "pos")?;
        let options = field(&root, "options")?;
        let window = field(&options, "window")?;

        let mut setup = Vec::new();
        for command in root
            .get("setup")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            setup.push(
                command
                    .as_str()
                    .ok_or("a setup command isn't a string")?
                    .to_string(),
            );
        }

        let mut title = Vec::new();
        for step in root
            .get("title")
            .map(|title| title.as_array().ok_or("title isn't a list"))
            .transpose()?
            .into_iter()
            .flatten()
        {
            title.push(Scenario::parse_step(step)?);
        }

        let mut steps = Vec::new();
        for step in field(&root, "steps")?
            .as_array()
            .ok_or("steps isn't a list")?
        {
            steps.push(Scenario::parse_step(step)?);
        }

        Ok(Scenario {
            name,
            world: World {
                seed: field(&world, "seed")?.as_i64().ok_or("seed isn't a long")?,
                preset: field(&world, "preset")?
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                time: field(&world, "time")?.as_i64().ok_or("time isn't a long")?,
                game_mode: field(&world, "game_mode")?
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                tick_rate: world
                    .get("tick_rate")
                    .map(|value| value.as_f64().ok_or("tick_rate isn't a number"))
                    .transpose()?
                    .unwrap_or(0.0) as f32,
            },
            player: Player {
                x: number(pos[0].clone())?,
                y: number(pos[1].clone())?,
                z: number(pos[2].clone())?,
                y_rot: number(field(&player, "y_rot")?)? as f32,
                x_rot: number(field(&player, "x_rot")?)? as f32,
            },
            options: Options {
                width: number(window[0].clone())? as i32,
                height: number(window[1].clone())? as i32,
                fov: number(field(&options, "fov")?)? as i32,
                gui_scale: number(field(&options, "gui_scale")?)? as i32,
                render_distance: number(field(&options, "render_distance")?)? as i32,
                hide_gui: options
                    .get("hide_gui")
                    .map(|value| value.as_bool().ok_or("hide_gui isn't a boolean"))
                    .transpose()?
                    .unwrap_or(false),
                hide_splash: options
                    .get("hide_splash")
                    .map(|value| value.as_bool().ok_or("hide_splash isn't a boolean"))
                    .transpose()?
                    .unwrap_or(false),
            },
            setup,
            title,
            steps,
        })
    }

    fn parse_step(step: &Value) -> Result<Step, String> {
        if let Some(screenshot) = step.get("screenshot") {
            return Ok(Step {
                screenshot: Some(screenshot.as_str().unwrap_or_default().to_string()),
                blink_on: step
                    .get("blink_on")
                    .map(|value| value.as_bool().ok_or("blink_on isn't a boolean"))
                    .transpose()?
                    .unwrap_or(false),
                ..Step::default()
            });
        }
        let mut wait_for = Vec::new();
        for packet in step
            .get("wait_for")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            wait_for.push(
                packet
                    .as_str()
                    .ok_or("a wait_for packet isn't a string")?
                    .to_string(),
            );
        }

        if let Some(key) = step.get("key") {
            let key = key.as_str().unwrap_or_default();
            if !Scenario::PRESSABLE_KEYS
                .iter()
                .any(|(name, _)| *name == key)
            {
                return Err(format!("unknown key '{key}'"));
            }
            return Ok(Step {
                key: Some(key.to_string()),
                wait_for: wait_for.clone(),
                ..Step::default()
            });
        }
        if let Some(text) = step.get("type") {
            return Ok(Step {
                r#type: Some(text.as_str().ok_or("type isn't a string")?.to_string()),
                wait_for: wait_for.clone(),
                ..Step::default()
            });
        }
        if let Some(cursor) = step.get("cursor").and_then(Value::as_array) {
            return Ok(Step {
                cursor: Some([
                    cursor[0].as_f64().unwrap_or_default(),
                    cursor[1].as_f64().unwrap_or_default(),
                ]),
                wait_for: wait_for.clone(),
                ..Step::default()
            });
        }
        if let Some(button) = step.get("click") {
            let button = button.as_str().unwrap_or_default();
            if !Scenario::MOUSE_BUTTONS
                .iter()
                .any(|(name, _)| *name == button)
            {
                return Err(format!("unknown mouse button '{button}'"));
            }
            return Ok(Step {
                click: Some(button.to_string()),
                wait_for: wait_for.clone(),
                ..Step::default()
            });
        }
        if let Some(scroll) = step.get("scroll") {
            return Ok(Step {
                scroll: Some(scroll.as_f64().ok_or("scroll isn't a number")?),
                wait_for: wait_for.clone(),
                ..Step::default()
            });
        }
        if let Some(command) = step.get("command") {
            return Ok(Step {
                command: Some(
                    command
                        .as_str()
                        .ok_or("command isn't a string")?
                        .to_string(),
                ),
                wait_for: wait_for.clone(),
                ..Step::default()
            });
        }

        let mut keys = Vec::new();
        for key in step
            .get("keys")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let name = key.as_str().unwrap_or_default();
            if !Scenario::KEYS.contains(&name) {
                return Err(format!(
                    "unknown key '{name}', expected one of {:?}",
                    Scenario::KEYS
                ));
            }
            if !keys.iter().any(|key| key == name) {
                keys.push(name.to_string());
            }
        }

        let look = step.get("look").and_then(Value::as_array).map(|rotation| {
            [
                rotation[0].as_f64().unwrap_or_default() as f32,
                rotation[1].as_f64().unwrap_or_default() as f32,
            ]
        });

        let ticks = step
            .get("ticks")
            .and_then(Value::as_i64)
            .ok_or("a step needs ticks")? as i32;
        if !wait_for.is_empty() && ticks < 1 {
            return Err("a step that waits for packets needs a tick".to_string());
        }
        let screenshots_every = step
            .get("screenshots_every")
            .map(|value| value.as_i64().ok_or("screenshots_every isn't a number"))
            .transpose()?
            .unwrap_or(0) as i32;
        if screenshots_every < 0 {
            return Err("screenshots_every can't be negative".to_string());
        }
        Ok(Step {
            ticks,
            keys,
            look,
            wait_for,
            screenshots_every,
            ..Step::default()
        })
    }
}
