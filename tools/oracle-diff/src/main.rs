//! Compares what vanilla and the port produced for the same scenario: tick traces, packet
//! captures and screenshots (formats in TESTING.md). Exits 0 when everything compared matches,
//! 1 when something differs and 2 on bad input.

mod packets;
mod screenshot;
mod trace;
mod value;
mod video;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::Value;

const USAGE: &str = "usage:
  oracle-diff trace <A/trace.jsonl> <B/trace.jsonl>
  oracle-diff packets <A/capture.jsonl> <B/capture.jsonl> [--join] [--ignore name,...] [--no-default-ignore] [--ordered]
  oracle-diff image <A.png> <B.png> [--threshold N] [--heatmap out.png]
  oracle-diff dir <A> <B> [--threshold N] [--join] [--ignore name,...] [--no-default-ignore] [--ordered] [--heatmaps <dir>] [--strict]
  oracle-diff video <A> [<B>] --out out.gif [--scale N]

  --join               compare only the join phase of each connection: the serverbound packets
                       before the scenario-start marker (as the join phase always is), and the
                       clientbound packets that came before the client sent it
  --ignore             packets to skip on top of the defaults; `minecraft:` may be left out
  --no-default-ignore  don't skip the packets that differ between any two runs (keep-alives,
                       pings, session ids, timers, timestamps)
  --ordered            also compare clientbound packets in order and byte for byte; by default
                       only serverbound ones are, since the vanilla server's own output varies
                       in order between runs
  --threshold          largest channel difference a pixel may have and still match (default 0)
  --heatmap(s)         where to write images marking the differing pixels
  --strict             in dir mode, a file only one side has counts as a difference
  --out                in video mode, the GIF to write
  --scale              in video mode, shrink the frames to 1/N of their size (default 1)

dir compares trace.jsonl, capture.jsonl and screenshots/*.png in two scenario output dirs.
video plays a scenario output dir's frames (screenshots/tick-NNNN.png, a step's screenshots_every)
as a GIF in game time, or two dirs' side by side with the heatmap of where they differ.
Exit status: 0 when everything compared matches, 1 when something differs, 2 on bad input.";

#[derive(Default)]
struct Options {
    paths: Vec<PathBuf>,
    ignored: Vec<String>,
    default_ignore: bool,
    ordered: bool,
    join_only: bool,
    threshold: u8,
    heatmap: Option<PathBuf>,
    strict: bool,
    out: Option<PathBuf>,
    scale: u32,
}

fn parse_options(args: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut options = Options {
        default_ignore: true,
        scale: 1,
        ..Options::default()
    };
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--ignore" => options.ignored.extend(
                value()?
                    .split(',')
                    .filter(|name| !name.is_empty())
                    .map(packet_name),
            ),
            "--no-default-ignore" => options.default_ignore = false,
            "--ordered" => options.ordered = true,
            "--join" => options.join_only = true,
            "--threshold" => {
                options.threshold = value()?.parse().map_err(|_| "--threshold takes 0-255")?
            }
            "--heatmap" | "--heatmaps" => options.heatmap = Some(PathBuf::from(value()?)),
            "--strict" => options.strict = true,
            "--out" => options.out = Some(PathBuf::from(value()?)),
            "--scale" => {
                options.scale = value()?
                    .parse()
                    .ok()
                    .filter(|scale| *scale > 0)
                    .ok_or("--scale takes a whole number above 0")?;
            }
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            path => options.paths.push(PathBuf::from(path)),
        }
    }

    if options.default_ignore {
        options
            .ignored
            .extend(packets::DEFAULT_IGNORED.iter().map(|name| name.to_string()));
    }
    Ok(options)
}

fn packet_name(name: &str) -> String {
    if name.contains(':') {
        name.to_string()
    } else {
        format!("minecraft:{name}")
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let result = parse_options(args).and_then(|options| {
        if command == "video" {
            return write_video(&options);
        }
        let [a, b] = options.paths.as_slice() else {
            return Err(format!("{command} takes two paths"));
        };
        match command.as_str() {
            "trace" => diff_trace(a, b),
            "packets" => diff_packets(a, b, &options),
            "image" => diff_image(a, b, &options, options.heatmap.as_deref()),
            "dir" => diff_dir(a, b, &options),
            _ => Err(format!("unknown command {command:?}")),
        }
    });

    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(message) => {
            eprintln!("oracle-diff: {message}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn write_video(options: &Options) -> Result<bool, String> {
    let (a, b) = match options.paths.as_slice() {
        [a] => (a, None),
        [a, b] => (a, Some(b.as_path())),
        _ => return Err("video takes one or two paths".to_string()),
    };
    let out = options.out.as_deref().ok_or("video needs --out")?;
    let frames = video::write(a, b, out, options.scale)?;
    println!("wrote {} ({frames} frames)", out.display());
    Ok(true)
}

fn read_json_lines(path: &Path) -> Result<Vec<Value>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line)
                .map_err(|error| format!("{}:{}: {error}", path.display(), index + 1))
        })
        .collect()
}

fn diff_trace(a: &Path, b: &Path) -> Result<bool, String> {
    let diff = trace::compare(&read_json_lines(a)?, &read_json_lines(b)?);
    print!("{diff}");
    Ok(diff.is_same())
}

fn diff_packets(a: &Path, b: &Path, options: &Options) -> Result<bool, String> {
    let read = |path: &Path| -> Result<Vec<packets::Packet>, String> {
        read_json_lines(path)?
            .iter()
            .map(|line| {
                packets::Packet::parse(line).map_err(|error| format!("{}: {error}", path.display()))
            })
            .collect()
    };
    let diff = packets::compare_scoped(
        &read(a)?,
        &read(b)?,
        &options.ignored,
        options.ordered,
        options.join_only,
    );
    print!("{diff}");
    Ok(diff.is_same())
}

fn diff_image(
    a: &Path,
    b: &Path,
    options: &Options,
    heatmap: Option<&Path>,
) -> Result<bool, String> {
    let open = |path: &Path| {
        image::open(path)
            .map(|image| image.to_rgba8())
            .map_err(|error| format!("{}: {error}", path.display()))
    };
    let (image_a, image_b) = (open(a)?, open(b)?);
    let diff = screenshot::compare(&image_a, &image_b, options.threshold)?;
    println!(
        "image {}: {diff}",
        a.file_name().unwrap_or_default().to_string_lossy()
    );

    if let (Some(heatmap), false) = (heatmap, diff.is_same()) {
        screenshot::heatmap(&image_a, &image_b, options.threshold)
            .save(heatmap)
            .map_err(|error| format!("{}: {error}", heatmap.display()))?;
        println!("  heatmap: {}", heatmap.display());
    }
    Ok(diff.is_same())
}

fn diff_dir(a: &Path, b: &Path, options: &Options) -> Result<bool, String> {
    let mut same = true;
    let mut compared = 0;
    let mut one_sided = Vec::new();

    let mut compare_file = |relative: &str,
                            diff: &dyn Fn(&Path, &Path) -> Result<bool, String>|
     -> Result<(), String> {
        match (a.join(relative).exists(), b.join(relative).exists()) {
            (true, true) => {
                compared += 1;
                same &= diff(&a.join(relative), &b.join(relative))?;
            }
            (true, false) => one_sided.push(format!("only in A: {relative}")),
            (false, true) => one_sided.push(format!("only in B: {relative}")),
            (false, false) => {}
        }
        Ok(())
    };

    compare_file("trace.jsonl", &diff_trace)?;
    compare_file("capture.jsonl", &|a, b| diff_packets(a, b, options))?;

    if let Some(heatmaps) = &options.heatmap {
        fs::create_dir_all(heatmaps).map_err(|error| format!("{}: {error}", heatmaps.display()))?;
    }
    for name in screenshot_names(a)?.union(&screenshot_names(b)?) {
        let heatmap = options.heatmap.as_ref().map(|dir| dir.join(name));
        compare_file(&format!("screenshots/{name}"), &|a, b| {
            diff_image(a, b, options, heatmap.as_deref())
        })?;
    }

    for line in &one_sided {
        println!("{line}");
    }
    if compared == 0 {
        return Err(format!(
            "nothing to compare between {} and {}",
            a.display(),
            b.display()
        ));
    }
    Ok(same && !(options.strict && !one_sided.is_empty()))
}

fn screenshot_names(dir: &Path) -> Result<BTreeSet<String>, String> {
    let dir = dir.join("screenshots");
    if !dir.exists() {
        return Ok(BTreeSet::new());
    }
    let entries = fs::read_dir(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    Ok(entries
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".png"))
        .collect())
}
