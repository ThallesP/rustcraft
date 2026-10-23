//! oracle/client/src/main/java/rustcraft/oracle/client/TraceWriter.java: writes `trace.jsonl`,
//! a line per tick with the player's state after the tick (TESTING.md), floats as their raw bits.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::world::entity::entity::EntityClass;

/// The trace's file, written line by line.
pub struct TraceWriter {
    writer: BufWriter<File>,
}

impl TraceWriter {
    pub fn new(file: &Path) -> std::io::Result<TraceWriter> {
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(TraceWriter {
            writer: BufWriter::new(File::create(file)?),
        })
    }

    pub fn write(&mut self, line: &str) -> std::io::Result<()> {
        writeln!(self.writer, "{line}")
    }

    pub fn close(mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

/// The line for the player after `tick` ran.
pub fn line(tick: i32, player: &LocalPlayer) -> String {
    let delta = player.get_delta_movement();
    format!(
        "{{\"tick\":{tick},\"pos\":[{},{},{}],\"delta\":[{},{},{}],\"y_rot\":{},\"x_rot\":{},\"on_ground\":{},\"horizontal_collision\":{},\"vertical_collision\":{},\"sprinting\":{},\"crouching\":{},\"pose\":\"{}\"}}",
        bits_f64(player.get_x()),
        bits_f64(player.get_y()),
        bits_f64(player.get_z()),
        bits_f64(delta.x),
        bits_f64(delta.y),
        bits_f64(delta.z),
        bits_f32(player.get_y_rot()),
        bits_f32(player.get_x_rot()),
        player.on_ground(),
        player.horizontal_collision,
        player.vertical_collision,
        player.is_sprinting(),
        player.is_crouching(),
        player.get_pose().name(),
    )
}

fn bits_f64(value: f64) -> String {
    format!("\"{:016x}\"", value.to_bits())
}

fn bits_f32(value: f32) -> String {
    format!("\"{:08x}\"", value.to_bits())
}
