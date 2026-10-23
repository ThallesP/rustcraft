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
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug)]
pub struct World {
    pub seed: i64,
    pub preset: String,
    pub time: i64,
    pub game_mode: String,
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
}

/// Either a screenshot, or `ticks` ticks with `keys` held and an optional fixed `look`.
#[derive(Clone, Debug)]
pub struct Step {
    pub ticks: i32,
    pub keys: Vec<String>,
    pub look: Option<[f32; 2]>,
    pub screenshot: Option<String>,
}

impl Scenario {
    pub const KEYS: [&'static str; 9] = [
        "forward", "back", "left", "right", "jump", "sneak", "sprint", "attack", "use",
    ];

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
            },
            setup,
            steps,
        })
    }

    fn parse_step(step: &Value) -> Result<Step, String> {
        if let Some(screenshot) = step.get("screenshot") {
            return Ok(Step {
                ticks: 0,
                keys: Vec::new(),
                look: None,
                screenshot: Some(screenshot.as_str().unwrap_or_default().to_string()),
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

        Ok(Step {
            ticks: step
                .get("ticks")
                .and_then(Value::as_i64)
                .ok_or("a step needs ticks")? as i32,
            keys,
            look,
            screenshot: None,
        })
    }
}
