//! The OpenGL backend on Mesa's llvmpipe under Xvfb, the way the oracle client runs vanilla
//! (`tools/oracle_client.py`'s `headless_env`): the device it makes, and a frame read back like
//! `Screenshot.takeScreenshot`. Each run is the `rustcraft` binary under `xvfb-run -a`, so SDL's
//! and GL's process state stays out of the other tests. Without `xvfb-run` the tests say so and
//! pass, since there's no display to open a context on.

use std::path::{Path, PathBuf};
use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_rustcraft");

/// The binary under a fresh Xvfb with llvmpipe, or `None` without `xvfb-run`.
fn run(arguments: &[&str]) -> Option<String> {
    let found = Command::new("xvfb-run").arg("--help").output().is_ok();
    if !found {
        eprintln!("skipped: no xvfb-run (sudo apt install xvfb)");
        return None;
    }
    let output = Command::new("xvfb-run")
        .args(["-a", "-s", "-screen 0 1920x1080x24 -nolisten tcp", BINARY])
        .args(arguments)
        .env_remove("WAYLAND_DISPLAY")
        .env("SDL_VIDEO_DRIVER", "x11")
        .env("LIBGL_ALWAYS_SOFTWARE", "1")
        .env("GALLIUM_DRIVER", "llvmpipe")
        .env("__GLX_VENDOR_LIBRARY_NAME", "mesa")
        .output()
        .expect("runs xvfb-run");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "rustcraft {arguments:?} failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Some(stdout)
}

/// The device's lines as vanilla logs them, and the flags vanilla's shaders depend on.
#[test]
fn makes_the_device_vanilla_makes() {
    let Some(output) = run(&["--device-info"]) else {
        return;
    };
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines[1].contains("llvmpipe"), "not llvmpipe: {output}");
    // The flags come from the extensions, like GlDevice's DeviceInfo.
    let extensions = lines[2]
        .strip_prefix("Using graphics device extensions: ")
        .expect("the extensions");
    let has = |extension: &str| extensions.split(", ").any(|enabled| enabled == extension);
    let flags = format!(
        "isZZeroToOne: {}, shaderDrawParameters: {}, isExplicitDepthRequired: false",
        has("GL_ARB_clip_control"),
        has("GL_ARB_shader_draw_parameters")
    );
    assert_eq!(lines[3], flags);

    // The oracle client's log, when this machine has run it, says what vanilla made of the same
    // driver, extensions in its HashSet's order. Then the flags must be the ones the shader
    // goldens are recorded with (RenderSuites' LLVMPIPE). Elsewhere (CI) Mesa can differ.
    let log = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla")
        .join(include_str!("../MINECRAFT_VERSION").trim())
        .join("oracle/walk/client.log");
    let Ok(log) = std::fs::read_to_string(&log) else {
        eprintln!(
            "no oracle client log at {}: checked the flags against the extensions only",
            log.display()
        );
        return;
    };
    let vanilla: Vec<&str> = log
        .lines()
        .filter_map(|line| {
            line.split_once("(Minecraft) Using graphics ")
                .map(|(_, rest)| rest)
        })
        .collect();
    let port: Vec<&str> = lines[..3]
        .iter()
        .map(|line| line.strip_prefix("Using graphics ").expect("a device line"))
        .collect();
    assert_eq!(port, vanilla);
    assert_eq!(
        lines[3],
        "isZZeroToOne: true, shaderDrawParameters: true, isExplicitDepthRequired: false"
    );
}

fn read_png(path: &Path) -> (u32, u32, png::ColorType, Vec<u8>) {
    let decoder = png::Decoder::new(std::io::BufReader::new(
        std::fs::File::open(path).expect("the screenshot exists"),
    ));
    let mut reader = decoder.read_info().expect("a PNG");
    let mut pixels = vec![0; reader.output_buffer_size().expect("a size")];
    let info = reader.next_frame(&mut pixels).expect("one frame");
    pixels.truncate(info.buffer_size());
    (info.width, info.height, info.color_type, pixels)
}

fn screenshot(name: &str, arguments: &[&str]) -> Option<(u32, u32, png::ColorType, Vec<u8>)> {
    let path: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let path_text = path.to_str().expect("a UTF-8 path");
    let mut all = vec!["--screenshot", path_text];
    all.extend_from_slice(arguments);
    run(&all)?;
    Some(read_png(&path))
}

/// The frame's clear, read back like vanilla's screenshots: RGBA at the target's size, every
/// pixel the clear color with its alpha made opaque.
#[test]
fn reads_back_a_cleared_frame_like_vanillas_screenshots() {
    // GuiRenderState.clearColorOverride's default, transparent black.
    let Some((width, height, color_type, pixels)) = screenshot("default-clear.png", &[]) else {
        return;
    };
    assert_eq!(
        (width, height, color_type),
        (854, 480, png::ColorType::Rgba)
    );
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| *pixel == [0, 0, 0, 255])
    );

    // The loading overlay's brand background (LoadingOverlay.LOGO_BACKGROUND_COLOR), at a size
    // whose rows aren't a multiple of 8 pixels.
    let (width, height, _, pixels) = screenshot(
        "brand-clear.png",
        &["--size", "427x241", "--clear", "FFEF323D"],
    )
    .expect("ran before");
    assert_eq!((width, height), (427, 241));
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| *pixel == [239, 50, 61, 255])
    );
}

/// A render pass clearing GL's lower left quarter (its render area from the origin), which the
/// readback's flip puts at the image's lower left, as vanilla's screenshots show GL's frames.
#[test]
fn reads_back_frames_the_right_way_up() {
    let Some((width, height, _, pixels)) = screenshot(
        "quadrant.png",
        &["--size", "320x240", "--quadrant", "FF0000FF"],
    ) else {
        return;
    };
    assert_eq!((width, height), (320, 240));
    for y in 0..height {
        for x in 0..width {
            let start = ((x + y * width) * 4) as usize;
            let lower_left = x < width / 2 && y >= height - height / 2;
            let expected = if lower_left {
                [0, 0, 255, 255]
            } else {
                [0, 0, 0, 255]
            };
            assert_eq!(pixels[start..start + 4], expected, "pixel ({x}, {y})");
        }
    }
}
