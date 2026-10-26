//! The OpenGL backend on Mesa's llvmpipe under Xvfb, the way the oracle client runs vanilla
//! (`tools/oracle_client.py`'s `headless_env`): the device it makes, and a frame read back like
//! `Screenshot.takeScreenshot`. Each run is the `rustcraft` binary under `xvfb-run -a`, so SDL's
//! and GL's process state stays out of the other tests. Without `xvfb-run` the tests say so and
//! pass, since there's no display to open a context on.
//!
//! Vanilla's pixels are only the port's to match on the driver vanilla's screenshots were taken
//! on (`ORACLE_DRIVER`): another llvmpipe needn't rasterize and sample to the same bits. On
//! another driver the frames only have to render. With `RUSTCRAFT_ORACLE_DRIVER=1` (CI's job in
//! the oracle's Ubuntu and Mesa) nothing is skipped: no `xvfb-run`, another driver, or a capture
//! or live run missing fails the test.
//!
//! The same checks run on the macOS GL profile (`oracle/gl-profiles.json`, TESTING.md's "GL
//! profiles"): llvmpipe restricted to Apple's OpenGL 4.1, against vanilla's run under it. And on
//! the Vulkan backend (`--graphicsBackend vulkan`) on Mesa's lavapipe, against vanilla's own
//! Vulkan backend's run there (TESTING.md's "Backends").

use md5::{Digest, Md5};
use std::path::{Path, PathBuf};
use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_rustcraft");

/// The lines vanilla logged about its device on the oracle machine
/// (`vanilla/26.3/oracle/*/client.log`): Mesa 26.0.8-1ubuntu0.3's llvmpipe on LLVM 21.1.8, the
/// extensions in their `HashSet`'s order.
const ORACLE_DEVICE: [&str; 3] = [
    "backend OpenGL, using drivers: 4.5 (Core Profile) Mesa 26.0.8-1ubuntu0.3",
    "device: llvmpipe (LLVM 21.1.8, 256 bits) (Mesa)",
    "device extensions: GL_ARB_multi_draw_indirect, GL_ARB_buffer_storage, \
     GL_ARB_shader_draw_parameters, GL_ARB_base_instance, GL_KHR_debug, GL_ARB_draw_indirect, \
     GL_ARB_clip_control, GL_ARB_vertex_attrib_binding, GL_ARB_direct_state_access, \
     GL_EXT_texture_filter_anisotropic",
];
const ORACLE_DRIVER: &str = "4.5 (Core Profile) Mesa 26.0.8-1ubuntu0.3";
/// What vanilla logged on the oracle's driver under the macOS GL profile
/// (`vanilla/26.3/oracle-gl41/*/client.log`): OpenGL 4.1, and of the extensions it looks for only
/// the anisotropic filtering.
const ORACLE_DEVICE_GL41: [&str; 3] = [
    "backend OpenGL, using drivers: 4.1 (Core Profile) Mesa 26.0.8-1ubuntu0.3",
    "device: llvmpipe (LLVM 21.1.8, 256 bits) (Mesa)",
    "device extensions: GL_EXT_texture_filter_anisotropic",
];

/// What vanilla's Vulkan backend logged on the oracle's lavapipe
/// (`vanilla/26.3/oracle-vulkan/*/client.log`): the extensions the instance (I) and the device (D)
/// were made with, in their `HashSet`'s order.
const ORACLE_DEVICE_VULKAN: [&str; 3] = [
    "backend Vulkan, using drivers: 1.4.335 llvmpipe Mesa 26.0.8-1ubuntu0.3 (LLVM 21.1.8)",
    "device: llvmpipe (LLVM 21.1.8, 256 bits) (0x10005)",
    "device extensions: VK_EXT_calibrated_timestamps (D), VK_KHR_synchronization2 (D), \
     VK_EXT_multi_draw (D), VK_EXT_vertex_attribute_divisor (D), VK_KHR_swapchain (D), \
     VK_KHR_surface (I), VK_KHR_push_descriptor (D), VK_KHR_xlib_surface (I), VK_EXT_debug_utils (I), \
     VK_KHR_dynamic_rendering (D)",
];
/// The binary's arguments for its Vulkan backend.
const VULKAN: &[&str] = &["--graphicsBackend", "vulkan"];
/// Mesa's lavapipe, the Vulkan driver `tools/oracle_client.py` runs vanilla on.
const LAVAPIPE_ICD: &str = "/usr/share/vulkan/icd.d/lvp_icd.json";

/// Whether every pixel check must run here: `RUSTCRAFT_ORACLE_DRIVER=1`.
fn strict() -> bool {
    std::env::var_os("RUSTCRAFT_ORACLE_DRIVER").is_some_and(|value| value == "1")
}

/// A check that can't run here: a failure where it must, a note otherwise.
fn skip(reason: &str) {
    assert!(!strict(), "RUSTCRAFT_ORACLE_DRIVER=1, but {reason}");
    eprintln!("skipped: {reason}");
}

/// Whether the binary renders on the oracle's driver here, after `run` found `xvfb-run`.
fn on_oracle_driver() -> bool {
    let device = run(&["--device-info"]).expect("xvfb-run ran before");
    let on_it = device.contains(&format!("using drivers: {ORACLE_DRIVER}\n"));
    if !on_it {
        skip(&format!(
            "vanilla's screenshots were taken on {ORACLE_DRIVER}, this is {}",
            device.lines().next().unwrap_or_default()
        ));
    }
    on_it
}

/// The binary under a fresh Xvfb with llvmpipe, or `None` without `xvfb-run`.
fn run(arguments: &[&str]) -> Option<String> {
    run_on(arguments, None)
}

/// Mesa's environment for a GL profile, the one `tools/oracle_client.py` and
/// `tools/port_client.py` run both clients under (`--gl-profile`).
fn gl_profile_env(name: &str) -> Vec<(String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("oracle/gl-profiles.json");
    let profiles: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("the GL profiles"))
            .expect("JSON");
    profiles[name]
        .as_object()
        .unwrap_or_else(|| panic!("no GL profile {name}"))
        .iter()
        .map(|(key, value)| (key.clone(), value.as_str().expect("a string").to_owned()))
        .collect()
}

/// `run` on llvmpipe restricted to a GL profile's OpenGL, if there's one.
fn run_on(arguments: &[&str], gl_profile: Option<&str>) -> Option<String> {
    let found = Command::new("xvfb-run").arg("--help").output().is_ok();
    if !found {
        skip("no xvfb-run (sudo apt install xvfb)");
        return None;
    }
    if arguments.contains(&"vulkan") && !Path::new(LAVAPIPE_ICD).exists() {
        skip("no lavapipe (sudo apt install mesa-vulkan-drivers)");
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
        .env("VK_DRIVER_FILES", LAVAPIPE_ICD)
        .env("VK_ICD_FILENAMES", LAVAPIPE_ICD)
        .envs(gl_profile.map(gl_profile_env).unwrap_or_default())
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
    // driver, extensions in its HashSet's order; on the oracle's driver without one, it's the
    // lines the oracle logged. Then the flags must be the ones the shader goldens are recorded
    // with (RenderSuites' LLVMPIPE). Elsewhere (CI's runner) Mesa can differ.
    let port: Vec<&str> = lines[..3]
        .iter()
        .map(|line| line.strip_prefix("Using graphics ").expect("a device line"))
        .collect();
    let log = vanilla_dir("oracle").join("walk/client.log");
    if let Some(vanilla) = vanilla_device(&log) {
        assert_eq!(port, vanilla);
    } else if strict() {
        assert_eq!(port, ORACLE_DEVICE);
    } else {
        eprintln!(
            "no oracle client log at {}: checked the flags against the extensions only",
            log.display()
        );
        return;
    }
    assert_eq!(
        lines[3],
        "isZZeroToOne: true, shaderDrawParameters: true, isExplicitDepthRequired: false"
    );
}

/// The lines a vanilla run's log has about its device, if there's a log.
fn vanilla_device(log: &Path) -> Option<Vec<String>> {
    let log = std::fs::read_to_string(log).ok()?;
    Some(
        log.lines()
            .filter_map(|line| {
                line.split_once("(Minecraft) Using graphics ")
                    .map(|(_, rest)| rest.to_owned())
            })
            .collect(),
    )
}

/// A directory of vanilla's runs: `vanilla/<version>/oracle`.
fn vanilla_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla")
        .join(include_str!("../MINECRAFT_VERSION").trim())
        .join(name)
}

/// The device on the macOS GL profile, as vanilla makes it there: OpenGL 4.1, none of the
/// extensions a Mac lacks, so depth from -1 to 1 and no shader draw parameters; vanilla's log of
/// its run under the profile when this machine has one, the lines it logged on the oracle's
/// driver with `RUSTCRAFT_ORACLE_DRIVER=1`.
#[test]
fn makes_the_device_vanilla_makes_on_the_macos_gl_profile() {
    let Some(output) = run_on(&["--device-info"], Some("macos")) else {
        return;
    };
    let lines: Vec<&str> = output.lines().collect();
    assert!(
        lines[0].contains("using drivers: 4.1 (Core Profile)"),
        "not OpenGL 4.1: {output}"
    );
    assert_eq!(
        lines[3],
        "isZZeroToOne: false, shaderDrawParameters: false, isExplicitDepthRequired: false"
    );
    let port: Vec<&str> = lines[..3]
        .iter()
        .map(|line| line.strip_prefix("Using graphics ").expect("a device line"))
        .collect();
    let log = vanilla_dir("oracle-gl41").join("walk/client.log");
    if let Some(vanilla) = vanilla_device(&log) {
        assert_eq!(port, vanilla);
    } else if strict() {
        assert_eq!(port, ORACLE_DEVICE_GL41);
    } else {
        // Another Mesa's overrides may leave other extensions: the flags are what's checked.
        eprintln!(
            "no oracle client log at {}: checked the flags only",
            log.display()
        );
    }
}

/// The Vulkan device as vanilla's Vulkan backend makes it on lavapipe: its lines as vanilla's log
/// of its run there has them when this machine has one, the lines it logged on the oracle's
/// driver with `RUSTCRAFT_ORACLE_DRIVER=1`; depth from 0 to 1 and the shader draw parameters,
/// whatever the driver.
#[test]
fn makes_the_device_vanilla_makes_on_vulkan() {
    let Some(output) = run(&["--device-info", VULKAN[0], VULKAN[1]]) else {
        return;
    };
    let lines: Vec<&str> = output.lines().collect();
    assert!(
        lines[0].starts_with("Using graphics backend Vulkan"),
        "not Vulkan: {output}"
    );
    assert_eq!(
        lines[3],
        "isZZeroToOne: true, shaderDrawParameters: true, isExplicitDepthRequired: false"
    );
    let port: Vec<&str> = lines[..3]
        .iter()
        .map(|line| line.strip_prefix("Using graphics ").expect("a device line"))
        .collect();
    let log = vanilla_dir("oracle-vulkan").join("walk/client.log");
    if let Some(vanilla) = vanilla_device(&log) {
        assert_eq!(port, vanilla);
    } else if strict() {
        assert_eq!(port, ORACLE_DEVICE_VULKAN);
    } else {
        eprintln!(
            "no oracle client log at {}: checked the flags only",
            log.display()
        );
    }
}

/// Every pipeline `ShaderManager.reload` compiles at startup, the required ones and the
/// optional ones, compiles and links on llvmpipe from the GLSL vanilla hands the driver.
#[test]
fn compiles_every_pipeline_vanilla_compiles() {
    compiles_every_pipeline_vanilla_compiles_on(&[]);
}

/// The same on Vulkan, from the SPIR-V itself.
#[test]
fn compiles_every_pipeline_vanilla_compiles_on_vulkan() {
    compiles_every_pipeline_vanilla_compiles_on(VULKAN);
}

fn compiles_every_pipeline_vanilla_compiles_on(backend: &[&str]) {
    let Some(output) = run(&[&["--compile-pipelines"], backend].concat()) else {
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
    screenshot_on(name, arguments, None)
}

fn screenshot_on(
    name: &str,
    arguments: &[&str],
    gl_profile: Option<&str>,
) -> Option<(u32, u32, png::ColorType, Vec<u8>)> {
    let path: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let path_text = path.to_str().expect("a UTF-8 path");
    let mut all = vec!["--screenshot", path_text];
    all.extend_from_slice(arguments);
    run_on(&all, gl_profile)?;
    Some(read_png(&path))
}

/// The frame's clear, read back like vanilla's screenshots: RGBA at the target's size, every
/// pixel the clear color with its alpha made opaque.
#[test]
fn reads_back_a_cleared_frame_like_vanillas_screenshots() {
    reads_back_a_cleared_frame_on("", &[]);
}

#[test]
fn reads_back_a_cleared_frame_like_vanillas_screenshots_on_vulkan() {
    reads_back_a_cleared_frame_on("vulkan-", VULKAN);
}

fn reads_back_a_cleared_frame_on(prefix: &str, backend: &[&str]) {
    // GuiRenderState.clearColorOverride's default, transparent black.
    let Some((width, height, color_type, pixels)) =
        screenshot(&format!("{prefix}default-clear.png"), backend)
    else {
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
        &format!("{prefix}brand-clear.png"),
        &[&["--size", "427x241", "--clear", "FFEF323D"], backend].concat(),
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
    reads_back_frames_the_right_way_up_on("", &[]);
}

/// On Vulkan, whose framebuffer's first row is the render area's origin's too: the same image.
#[test]
fn reads_back_frames_the_right_way_up_on_vulkan() {
    reads_back_frames_the_right_way_up_on("vulkan-", VULKAN);
}

fn reads_back_frames_the_right_way_up_on(prefix: &str, backend: &[&str]) {
    let Some((width, height, _, pixels)) = screenshot(
        &format!("{prefix}quadrant.png"),
        &[&["--size", "320x240", "--quadrant", "FF0000FF"], backend].concat(),
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
    draws_gui_fills_exactly_on("", &[]);
}

#[test]
fn draws_gui_fills_exactly_on_vulkan() {
    draws_gui_fills_exactly_on("vulkan-", VULKAN);
}

fn draws_gui_fills_exactly_on(prefix: &str, backend: &[&str]) {
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
    arguments.extend_from_slice(backend);
    let Some((width, height, _, pixels)) =
        screenshot(&format!("{prefix}gui-fills.png"), &arguments)
    else {
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
            &[],
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

/// Vanilla's screenshots of the scenarios whose screenshots the port takes too, each by the
/// digest of its RGBA bytes (`vanilla/26.3/oracle/<scenario>/screenshots/<name>.png`), which the
/// oracle recorded on `ORACLE_DRIVER`.
const VANILLA_SCREENSHOTS: &[(&str, &str, &str)] = &[
    ("walk-no-gui", "spawn", "a38fcf9081510059fb5d52b0f34716e9"),
    (
        "walk-no-gui",
        "after-walk",
        "7d2656aefedcf34c937e0b57f87101f5",
    ),
    ("walk", "spawn", "40f5b5f93bd28c499cf9d2ec70243fed"),
    ("walk", "after-walk", "b4f2a3b57e31c51be4992e54dd84d878"),
    ("sky-sunset", "west", "c3f3c068f86e1c6960d345d041bf8c8e"),
    ("sky-sunset", "east", "d2d52caf94690f2a6afd31f959fc81c6"),
    ("sky-night", "zenith", "b216b4432026b50cd7733bb5bc18b25d"),
    ("sky-night", "stars", "976de7b4bb8226cc5099e593b3415ea1"),
    ("sky-sunrise", "east", "a8003efca04138403a4812fced227287"),
    ("sky-sunrise", "up", "945a20075d656a90f21083e91d038dcf"),
    ("pause-no-gui", "paused", "4350c9facc35752a7708ecba620d0ffb"),
    (
        "pause-no-gui",
        "hovered",
        "a2144669815c6349d67f9c0859546fd8",
    ),
    (
        "pause-no-gui",
        "tooltip",
        "eba198826998737548b8c87258092d41",
    ),
    (
        "pause-no-gui",
        "resumed",
        "2fc891762ababfe30cce0e7d8a4546a2",
    ),
    ("chat", "tellraw", "5885a05cfc100e8e3687b5ee89990eee"),
    ("chat", "feedback", "a74ffeed5795b07837f9518c105dad32"),
    ("chat", "fading", "fbdf11cea5dbc46f2da297397b1f617d"),
    ("chat", "faded", "217d29f7f3b67b6685884184540b1592"),
    (
        "options-no-gui",
        "options",
        "1d2e7be5274dcb3303aacbc66c2eff6e",
    ),
    ("options-no-gui", "fov", "76b0b7ca7fa3066abc290bcecad01091"),
    (
        "options-no-gui",
        "telemetry",
        "997b1fef94ee9bfb8edc2d501f2a7a24",
    ),
    ("options-no-gui", "back", "079e0fe8327d3f0ac3f1db4d795bb8b6"),
    (
        "options-no-gui",
        "resumed",
        "b928e0e01496f18eaf778f512dc28801",
    ),
];

fn rgba_digest(pixels: &[u8]) -> String {
    let digest: [u8; 16] = Md5::digest(pixels).into();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn vanilla_screenshot(
    screenshots: &'static [(&str, &str, &str)],
    scenario: &str,
    name: &str,
) -> &'static str {
    screenshots
        .iter()
        .find(|(listed, listed_name, _)| *listed == scenario && *listed_name == name)
        .map(|(_, _, digest)| *digest)
        .unwrap_or_else(|| panic!("no digest of vanilla's {scenario} {name}"))
}

/// The walk's spawn view, rendered offscreen from the level a capture of the walk's server fills
/// (`--render-scenario`): vanilla's, pixel for pixel, on the oracle's driver; on another the
/// frame only has to render. The capture is the oracle's on a machine that has it, or the port's
/// own (CI records it before the tests); without one the test says so and passes.
#[test]
fn renders_the_walks_spawn_view_like_vanilla() {
    render_the_walks_spawn_view("walk-spawn.png", &[]);
}

/// The same view, the window resized from 427 by 241 after the first frame (`Window`'s
/// framebuffer resize, `GameRenderer.resize`): the main target made again at the new size, the
/// framebuffers of the old one gone with it, the projection and the visible sections following.
/// The same pixels as vanilla's at that size from the start.
#[test]
fn renders_the_walks_spawn_view_like_vanilla_after_a_resize() {
    render_the_walks_spawn_view("walk-spawn-resized.png", &["--start-size", "427x241"]);
}

/// The same view on the macOS GL profile, where the terrain draws one section at a time
/// (`DrawSeparate`) with depth from -1 to 1: vanilla's under the profile.
#[test]
fn renders_the_walks_spawn_view_like_vanilla_on_the_macos_gl_profile() {
    render_the_walks_spawn_view_on(
        "walk-spawn-gl41.png",
        &[],
        Some("macos"),
        VANILLA_SCREENSHOTS_GL41,
    );
}

/// The same view on Vulkan: vanilla's Vulkan backend's on lavapipe, which differs from its GL
/// frame in a few thousand pixels by at most 2, the two drivers' rasterizers' own.
#[test]
fn renders_the_walks_spawn_view_like_vanilla_on_vulkan() {
    render_the_walks_spawn_view_on(
        "walk-spawn-vulkan.png",
        VULKAN,
        None,
        VANILLA_SCREENSHOTS_VULKAN,
    );
}

fn render_the_walks_spawn_view(name: &str, more_arguments: &[&str]) {
    render_the_walks_spawn_view_on(name, more_arguments, None, VANILLA_SCREENSHOTS);
}

fn render_the_walks_spawn_view_on(
    name: &str,
    more_arguments: &[&str],
    gl_profile: Option<&str>,
    screenshots: &'static [(&str, &str, &str)],
) {
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
        skip("no capture of the walk (tools/port_client.py capture walk)");
        return;
    };
    let mut arguments = vec![
        "--render-scenario",
        "oracle/scenarios/walk-no-gui.json",
        "--capture",
        capture.to_str().expect("a UTF-8 path"),
    ];
    arguments.extend_from_slice(more_arguments);
    let Some((width, height, color_type, pixels)) = screenshot_on(name, &arguments, gl_profile)
    else {
        return;
    };
    assert_eq!(
        (width, height, color_type),
        (854, 480, png::ColorType::Rgba)
    );
    if on_oracle_driver() {
        assert_eq!(
            rgba_digest(&pixels),
            vanilla_screenshot(screenshots, "walk-no-gui", "spawn")
        );
    }
}

/// The screenshots of the port's own runs on a server (`tools/port_client.py capture
/// <scenario>`, into `target/oracle/<scenario>/screenshots/`) against vanilla's, every one of
/// them, on the oracle's driver; a scenario the port hasn't run here is skipped. Where vanilla's
/// screenshots are, their digests are checked to be the ones above.
#[test]
fn takes_vanillas_screenshots_on_a_server() {
    takes_vanillas_screenshots("oracle", "", VANILLA_SCREENSHOTS);
}

/// Vanilla's Vulkan backend's screenshots on lavapipe (`vanilla/26.3/oracle-vulkan/`), by their
/// digests.
const VANILLA_SCREENSHOTS_VULKAN: &[(&str, &str, &str)] = &[
    ("walk-no-gui", "spawn", "a76d6cc27f4c9454c4115d8f7d18cdc7"),
    (
        "walk-no-gui",
        "after-walk",
        "1e44d5e7883bc2e31ceeb6b6bd640c38",
    ),
    ("walk", "spawn", "36d810d13206383ad289d6e70121d162"),
    ("walk", "after-walk", "b4d56f324f59f09a8654e66c429f54d0"),
];

/// The port's runs on Vulkan (`tools/port_client.py capture <scenario> --backend vulkan --out
/// target/oracle-vulkan`) against vanilla's Vulkan runs.
#[test]
fn takes_vanillas_screenshots_on_a_server_on_vulkan() {
    takes_vanillas_screenshots(
        "oracle-vulkan",
        " --backend vulkan --out target/oracle-vulkan",
        VANILLA_SCREENSHOTS_VULKAN,
    );
}

/// Vanilla's screenshots on the macOS GL profile (`vanilla/26.3/oracle-gl41/`), by their digests:
/// its default ones, bit for bit.
const VANILLA_SCREENSHOTS_GL41: &[(&str, &str, &str)] = &[
    ("walk-no-gui", "spawn", "a38fcf9081510059fb5d52b0f34716e9"),
    (
        "walk-no-gui",
        "after-walk",
        "7d2656aefedcf34c937e0b57f87101f5",
    ),
    ("walk", "spawn", "40f5b5f93bd28c499cf9d2ec70243fed"),
    ("walk", "after-walk", "b4f2a3b57e31c51be4992e54dd84d878"),
];

/// The same on the macOS GL profile: the port's runs under it (`tools/port_client.py capture
/// <scenario> --gl-profile macos --out target/oracle-gl41`) against vanilla's.
#[test]
fn takes_vanillas_screenshots_on_a_server_on_the_macos_gl_profile() {
    takes_vanillas_screenshots(
        "oracle-gl41",
        " --gl-profile macos --out target/oracle-gl41",
        VANILLA_SCREENSHOTS_GL41,
    );
}

/// The port's screenshots in `target/<runs>/` against vanilla's in `vanilla/26.3/<runs>/`,
/// `capture_options` being what `port_client.py capture` takes to record them.
fn takes_vanillas_screenshots(
    runs: &str,
    capture_options: &str,
    screenshots: &[(&str, &str, &str)],
) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let vanilla_dir = vanilla_dir(runs);
    for &(scenario, name, digest) in screenshots {
        let path = vanilla_dir.join(format!("{scenario}/screenshots/{name}.png"));
        if path.exists() {
            let (_, _, _, pixels) = read_png(&path);
            assert_eq!(
                rgba_digest(&pixels),
                digest,
                "vanilla's {scenario} {name} changed: update its digest"
            );
        }
    }

    if run(&["--device-info"]).is_none() || !on_oracle_driver() {
        return;
    }
    for &(scenario, name, digest) in screenshots {
        let path = root.join(format!("target/{runs}/{scenario}/screenshots/{name}.png"));
        if !path.exists() {
            skip(&format!(
                "no screenshot of the port's {scenario} (tools/port_client.py capture \
                 {scenario}{capture_options})"
            ));
            continue;
        }
        let (_, _, color_type, pixels) = read_png(&path);
        assert_eq!(color_type, png::ColorType::Rgba, "{scenario} {name}");
        assert_eq!(
            rgba_digest(&pixels),
            digest,
            "the port's {scenario} {name} differs from vanilla's: oracle-diff image \
             vanilla/26.3/{runs}/{scenario}/screenshots/{name}.png {}",
            path.display()
        );
    }
}
