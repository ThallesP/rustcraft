//! The OpenGL backend on Mesa's llvmpipe under Xvfb, the way the oracle client runs vanilla
//! (`tools/oracle_client.py`'s `headless_env`): the device it makes, and a frame read back like
//! `Screenshot.takeScreenshot`. Each run is the `rustcraft` binary under `xvfb-run -a`, so SDL's
//! and GL's process state stays out of the other tests. Without `xvfb-run` the tests say so and
//! pass, since there's no display to open a context on.

use md5::{Digest, Md5};
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

/// Every pipeline `ShaderManager.reload` compiles at startup, the required ones and the
/// optional ones, compiles and links on llvmpipe from the GLSL vanilla hands the driver.
#[test]
fn compiles_every_pipeline_vanilla_compiles() {
    let Some(output) = run(&["--compile-pipelines"]) else {
        return;
    };
    assert_eq!(output.trim(), "compiled 192 of 192 pipelines");
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

/// GUI fills drawn through `pipeline/gui`, whose pixels are known exactly: each fill's corners
/// are whole pixels at the GUI scale, so every pixel's center is inside a fill or outside it, never
/// on an edge, and its opaque color blends (`TRANSLUCENT`) to itself, since alpha 1 keeps none of
/// what was there. A fill with alpha 0 is discarded by the fragment shader and leaves nothing.
/// Later fills cover earlier ones, as `GuiRenderer` draws them in order.
#[test]
fn draws_gui_fills_exactly() {
    // In GUI pixels at scale 2, as GuiGraphicsExtractor.fill takes them: the second fill's corners
    // come the other way round, which fill swaps, and the last is transparent over everything.
    let fills = [
        (10, 20, 60, 45, 0xFFFF0000u32),
        (80, 80, 30, 30, 0xFF00FF00),
        (100, 5, 150, 15, 0xFF2040C0),
        (0, 0, 160, 120, 0x00FFFFFF),
    ];
    let fill_arguments: Vec<String> = fills
        .iter()
        .map(|(x0, y0, x1, y1, color)| format!("{x0},{y0},{x1},{y1},{color:08X}"))
        .collect();
    let mut arguments = vec!["--size", "320x240", "--gui-scale", "2"];
    for fill in &fill_arguments {
        arguments.push("--fill");
        arguments.push(fill);
    }
    let Some((width, height, _, pixels)) = screenshot("gui-fills.png", &arguments) else {
        return;
    };
    assert_eq!((width, height), (320, 240));
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let mut expected = [0, 0, 0, 255];
            for &(x0, y0, x1, y1, color) in &fills {
                let inside = (x0.min(x1) * 2..x0.max(x1) * 2).contains(&x)
                    && (y0.min(y1) * 2..y0.max(y1) * 2).contains(&y);
                if inside && color >> 24 != 0 {
                    expected = [(color >> 16) as u8, (color >> 8) as u8, color as u8, 255];
                }
            }
            let start = ((x + y * width as i32) * 4) as usize;
            assert_eq!(pixels[start..start + 4], expected, "pixel ({x}, {y})");
        }
    }
}

/// The block atlas drawn on the GPU like vanilla's `TextureAtlas.upload`, read back like
/// `dumpContents`, against the pixels its draws are known to give. The layout and every sprite's
/// mip levels are vanilla's (the `SpriteLoader` goldens); each sprite is drawn by
/// `pipeline/animate_sprite_blit` over its padded place, and with nearest filtering and clamped
/// edges the pixel `k` pixels into the place at mip level `L` is the sprite's texel `k - p_L`
/// (`p_L` the padding at that level) clamped into the sprite, of its first frame. Every sprite's
/// size and padding are multiples of `2^L`, so each pixel's texture coordinate lands on a texel's
/// center, far from any edge. llvmpipe zeroes a new texture, so where no sprite is drawn the atlas
/// is transparent black.
#[test]
fn draws_the_block_atlas_from_vanillas_sprites() {
    use rustcraft::net::minecraft::client::renderer::texture::sprite_loader::SpriteLoader;
    use rustcraft::net::minecraft::resources::identifier::Identifier;
    use rustcraft::net::minecraft::server::packs::pack_resources::PackResources;
    use rustcraft::net::minecraft::server::packs::pack_type::PackType;
    use rustcraft::net::minecraft::server::packs::resources::multi_pack_resource_manager::MultiPackResourceManager;
    use rustcraft::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;
    use std::rc::Rc;

    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("atlas");
    let _ = std::fs::remove_dir_all(&dir);
    let Some(output) = run(&["--dump-atlas", dir.to_str().expect("a UTF-8 path")]) else {
        return;
    };
    assert!(output.contains("2048x2048, 5 mip levels"), "{output}");

    let vanilla = VanillaPackResources::create_client_pack(
        &VanillaPackResources::vanilla_jar_root(),
        Path::new("no-asset-index"),
    );
    let layers = vanilla
        .resource_layers()
        .iter()
        .map(|layer| layer.clone() as Rc<dyn PackResources>)
        .collect();
    let manager = MultiPackResourceManager::new(PackType::CLIENT_RESOURCES, layers);
    let preparations = SpriteLoader::new(Identifier::with_default_namespace("blocks"), 16384)
        .load_and_stitch(
            &manager,
            &Identifier::with_default_namespace("blocks"),
            4,
            0,
        )
        .expect("the atlas fits");

    for level in 0..=preparations.mip_level {
        let width = (preparations.width >> level) as usize;
        let height = (preparations.height >> level) as usize;
        let mut expected = vec![0u8; width * height * 4];
        for sprite in &preparations.regions {
            let contents = sprite.contents();
            let image = &contents.by_mip_level()[level as usize];
            let (frame_x, frame_y) = contents.animated_texture().map_or((0, 0), |animated| {
                let frame = animated.frames[0].index;
                (
                    (animated.get_frame_x(frame) * contents.width()) >> level,
                    (animated.get_frame_y(frame) * contents.height()) >> level,
                )
            });
            let padding = sprite.padding() >> level;
            let sprite_width = contents.width() >> level;
            let sprite_height = contents.height() >> level;
            for j in 0..(contents.height() + sprite.padding() * 2) >> level {
                for i in 0..(contents.width() + sprite.padding() * 2) >> level {
                    let source_x = (i - padding).clamp(0, sprite_width - 1) + frame_x;
                    let source_y = (j - padding).clamp(0, sprite_height - 1) + frame_y;
                    let pixel = image.get_pixel_abgr(source_x, source_y).to_le_bytes();
                    let x = (sprite.get_x() >> level) + i;
                    let y = (sprite.get_y() >> level) + j;
                    let start = (x as usize + y as usize * width) * 4;
                    expected[start..start + 4].copy_from_slice(&pixel);
                }
            }
        }

        let (actual_width, actual_height, color_type, actual) =
            read_png(&dir.join(format!("minecraft_textures_atlas_blocks.png_{level}.png")));
        assert_eq!(
            (actual_width as usize, actual_height as usize, color_type),
            (width, height, png::ColorType::Rgba)
        );
        let differing = (0..width * height)
            .filter(|pixel| actual[pixel * 4..pixel * 4 + 4] != expected[pixel * 4..pixel * 4 + 4])
            .collect::<Vec<_>>();
        assert!(
            differing.is_empty(),
            "mip level {level}: {} of {} pixels differ, the first at ({}, {}): {:?} instead of {:?}",
            differing.len(),
            width * height,
            differing[0] % width,
            differing[0] / width,
            &actual[differing[0] * 4..differing[0] * 4 + 4],
            &expected[differing[0] * 4..differing[0] * 4 + 4]
        );
    }
}

/// The digest of vanilla's GUI-free walk spawn view
/// (`vanilla/26.3/oracle/walk-no-gui/screenshots/spawn.png`'s RGBA bytes), which the oracle
/// recorded.
const VANILLA_WALK_SPAWN_MD5: &str = "a38fcf9081510059fb5d52b0f34716e9";

/// The walk's spawn view, rendered offscreen from the level a capture of the walk's server fills
/// (`--render-scenario`): vanilla's, pixel for pixel. The capture is the oracle's on a machine
/// that has it, or the port's own (CI records it before the tests); without one the test says
/// so and passes.
#[test]
fn renders_the_walks_spawn_view_like_vanilla() {
    let capture = [
        std::env::var("RUSTCRAFT_CHUNK_CAPTURE")
            .ok()
            .map(PathBuf::from),
        Some(PathBuf::from("vanilla/26.3/oracle/walk/capture.jsonl")),
        Some(PathBuf::from("target/oracle/walk/capture.jsonl")),
    ]
    .into_iter()
    .flatten()
    .find(|path| path.exists());
    let Some(capture) = capture else {
        eprintln!("skipped: no capture of the walk (tools/port_client.py capture walk)");
        return;
    };
    let Some((width, height, color_type, pixels)) = screenshot(
        "walk-spawn.png",
        &[
            "--render-scenario",
            "oracle/scenarios/walk-no-gui.json",
            "--capture",
            capture.to_str().expect("a UTF-8 path"),
        ],
    ) else {
        return;
    };
    assert_eq!(
        (width, height, color_type),
        (854, 480, png::ColorType::Rgba)
    );
    let digest: [u8; 16] = Md5::digest(&pixels).into();
    let digest: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(digest, VANILLA_WALK_SPAWN_MD5);
}
