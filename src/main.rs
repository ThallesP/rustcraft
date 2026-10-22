//! The client's entry point, for now a frame on the OpenGL backend the way vanilla starts it:
//! SDL's video subsystem, the GL library, the device on its hidden window, then the main render
//! target cleared like `GameRenderer.render` clears it at the start of each frame.
//!
//!   rustcraft --device-info                     prints the device as vanilla logs it
//!   rustcraft --screenshot out.png [--size WxH] [--clear AARRGGBB] [--quadrant AARRGGBB]
//!                                               renders a frame offscreen and writes it as a PNG
//!   rustcraft --window [--size WxH] [--clear AARRGGBB] [--frames N]
//!                                               shows frames in a window until it's closed
//!
//! The clear color defaults to `GuiRenderState.clearColorOverride`'s, transparent black; the
//! loading overlay's is its brand background, `FFEF323D`. `--quadrant` clears the lower left
//! quarter in GL's coordinates through a render pass's area, which shows the readback's
//! orientation.

use rustcraft::com::mojang::blaze3d::pipeline::main_target::MainTarget;
use rustcraft::com::mojang::blaze3d::platform::window::Window;
use rustcraft::com::mojang::blaze3d::systems::render_system::RenderSystem;
use rustcraft::com::mojang::renderpearl::api::commands::render_pass::RenderArea;
use rustcraft::com::mojang::renderpearl::api::commands::render_pass_descriptor::RenderPassDescriptor;
use rustcraft::com::mojang::renderpearl::api::device::gpu_backend::GpuBackend;
use rustcraft::com::mojang::renderpearl::api::device::gpu_debug_options::GpuDebugOptions;
use rustcraft::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use rustcraft::com::mojang::renderpearl::backend::opengl::gl_backend::GlBackend;
use rustcraft::net::minecraft::client::screenshot::Screenshot;
use rustcraft::net::minecraft::util::argb::ARGB;
use rustcraft::org::joml::vector4f::Vector4f;
use rustcraft::org::lwjgl::sdl::sdl_events::{SDLEvents, SdlEvent};
use std::path::PathBuf;
use std::process::ExitCode;

enum Mode {
    DeviceInfo,
    Screenshot(PathBuf),
    Window,
}

struct Arguments {
    mode: Mode,
    width: i32,
    height: i32,
    clear_color: Vector4f,
    quadrant_color: Option<Vector4f>,
    frames: Option<u32>,
}

fn parse_arguments() -> Result<Arguments, String> {
    let mut mode = None;
    let (mut width, mut height) = (854, 480);
    let mut clear_color = Vector4f::from_scalar(0.0);
    let mut quadrant_color = None;
    let mut frames = None;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        let mut value = || arguments.next().ok_or(format!("{argument} needs a value"));
        match argument.as_str() {
            "--device-info" => mode = Some(Mode::DeviceInfo),
            "--screenshot" => mode = Some(Mode::Screenshot(PathBuf::from(value()?))),
            "--window" => mode = Some(Mode::Window),
            "--size" => {
                let size = value()?;
                let (w, h) = size
                    .split_once('x')
                    .ok_or(format!("--size takes WxH, not {size}"))?;
                width = w.parse().map_err(|_| format!("bad width {w}"))?;
                height = h.parse().map_err(|_| format!("bad height {h}"))?;
            }
            "--clear" => {
                let hex = value()?;
                let argb = u32::from_str_radix(&hex, 16)
                    .map_err(|_| format!("--clear takes AARRGGBB, not {hex}"))?;
                clear_color = ARGB::vector4f_from_argb32(argb as i32);
            }
            "--quadrant" => {
                let hex = value()?;
                let argb = u32::from_str_radix(&hex, 16)
                    .map_err(|_| format!("--quadrant takes AARRGGBB, not {hex}"))?;
                quadrant_color = Some(ARGB::vector4f_from_argb32(argb as i32));
            }
            "--frames" => {
                frames = Some(
                    value()?
                        .parse()
                        .map_err(|_| "--frames takes a count".to_owned())?,
                )
            }
            _ => return Err(format!("unknown argument {argument}")),
        }
    }
    Ok(Arguments {
        mode: mode.ok_or("pass --device-info, --screenshot PATH or --window")?,
        width,
        height,
        clear_color,
        quadrant_color,
        frames,
    })
}

/// `GameRenderer.render`'s first command: the main target cleared to the clear color, depth to 0
/// (reversed depth), then a render pass over it, like the GUI's, which draws nothing yet.
fn render_frame(
    device: &dyn GpuDevice,
    main_target: &MainTarget,
    clear_color: &Vector4f,
    quadrant_color: Option<Vector4f>,
) {
    let encoder = device.create_command_encoder();
    encoder.clear_color_and_depth_textures(
        &**main_target
            .get_color_texture()
            .expect("the main target has color"),
        clear_color,
        &**main_target
            .get_depth_texture()
            .expect("the main target has depth"),
        0.0,
    );
    let pass = encoder.create_render_pass_with(
        "GUI",
        main_target
            .get_color_texture_view()
            .expect("the main target has color"),
        None,
        None,
        None,
    );
    drop(pass);

    if let Some(quadrant_color) = quadrant_color {
        let view = main_target
            .get_color_texture_view()
            .expect("the main target has color");
        let descriptor = RenderPassDescriptor::builder("Quadrant")
            .with_color_attachment(view.clone(), Some(quadrant_color))
            .with_render_area(RenderArea {
                x: 0,
                y: 0,
                width: main_target.width / 2,
                height: main_target.height / 2,
            })
            .build();
        drop(encoder.create_render_pass(descriptor));
    }
}

fn run(arguments: Arguments) -> Result<(), String> {
    RenderSystem::init_backend_system();
    let mut backend = GlBackend::new();
    backend.load_library().map_err(|error| error.to_string())?;
    let debug_options = GpuDebugOptions {
        log_level: 1,
        ..GpuDebugOptions::default()
    };
    let device = backend
        .create_device(&debug_options)
        .map_err(|error| error.to_string())?;
    let info = device.get_device_info();

    match arguments.mode {
        Mode::DeviceInfo => {
            println!(
                "Using graphics backend {}, using drivers: {}",
                info.backend_name, info.driver_info
            );
            println!(
                "Using graphics device: {} ({})",
                info.name, info.vendor_name
            );
            println!(
                "Using graphics device extensions: {}",
                info.underlying_extensions.join(", ")
            );
            println!(
                "isZZeroToOne: {}, shaderDrawParameters: {}, isExplicitDepthRequired: {}",
                info.is_z_zero_to_one,
                info.features.shader_draw_parameters,
                info.hints_and_workarounds.is_explicit_depth_required
            );
        }
        Mode::Screenshot(path) => {
            let main_target = MainTarget::new(&*device, arguments.width, arguments.height);
            render_frame(
                &*device,
                &main_target,
                &arguments.clear_color,
                arguments.quadrant_color,
            );
            let image = Screenshot::take_screenshot(&*device, &main_target, 1);
            image
                .write_to_file(&path)
                .map_err(|error| format!("can't write {}: {error}", path.display()))?;
            println!(
                "wrote {} ({}x{})",
                path.display(),
                image.get_width(),
                image.get_height()
            );
        }
        Mode::Window => {
            let window = Window::new(
                &backend,
                arguments.width,
                arguments.height,
                "Minecraft* 26.3 (rustcraft)",
            );
            let surface = device.create_surface(window.handle());
            let main_target = MainTarget::new(&*device, window.get_width(), window.get_height());
            let mut frame = 0;
            'frames: while arguments.frames.is_none_or(|frames| frame < frames) {
                let mut event = SdlEvent::default();
                while SDLEvents::SDL_PollEvent(&mut event) {
                    if event.event_type == SDLEvents::SDL_EVENT_QUIT
                        || event.event_type == SDLEvents::SDL_EVENT_WINDOW_CLOSE_REQUESTED
                    {
                        break 'frames;
                    }
                }
                render_frame(
                    &*device,
                    &main_target,
                    &arguments.clear_color,
                    arguments.quadrant_color,
                );
                surface.blit_from_texture(
                    main_target
                        .get_color_texture_view()
                        .expect("the main target has color"),
                );
                surface.present();
                frame += 1;
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match parse_arguments().and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rustcraft: {error}");
            ExitCode::FAILURE
        }
    }
}
