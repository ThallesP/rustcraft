//! net/minecraft/client/Screenshot.java
//!
//! Only `takeScreenshot`: the target's color texture copied into a buffer, then each pixel read
//! as a little-endian int (red in the low byte, so ABGR) into the image, bottom row first, since
//! GL's rows start at the bottom, with its alpha made opaque. A downscale factor averages each
//! block's red, green and blue. Saving to the screenshots folder (`grab`) comes with the key
//! binding.
//!
//! Java's callback runs when the copy's fence signals, which `RenderSystem` checks each frame (on
//! Vulkan, when the encoder's destruction queue comes round to the copy's submit again, two
//! submits on); here `take_screenshot` submits until it has run: once on GL, twice on Vulkan.

use crate::com::mojang::blaze3d::pipeline::render_target::RenderTarget;
use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer;
use crate::com::mojang::renderpearl::api::commands::command_encoder::CommandEncoder;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::net::minecraft::util::argb::ARGB;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Screenshot;

impl Screenshot {
    pub const SCREENSHOT_DIR: &str = "screenshots";

    /// Submits until a copy's callback has run (`done`), three times at most: a backend runs it
    /// once the GPU is done with the copy, at most two submits on.
    pub fn submit_until(encoder: &dyn CommandEncoder, done: impl Fn() -> bool) {
        for _ in 0..3 {
            encoder.submit();
            if done() {
                return;
            }
        }
    }

    /// Panics where Java throws for a target without a color texture, or a size the factor
    /// doesn't divide.
    pub fn take_screenshot(
        device: &dyn GpuDevice,
        target: &RenderTarget,
        downscale_factor: i32,
    ) -> NativeImage {
        let width = target.width;
        let height = target.height;
        let source_texture = target
            .get_color_texture()
            .expect("Tried to capture screenshot of an incomplete framebuffer");
        assert!(
            width % downscale_factor == 0 && height % downscale_factor == 0,
            "Image size is not divisible by downscale factor"
        );
        let block_size = source_texture.get_format().block_size();
        let buffer = device.create_buffer(
            Some("Screenshot buffer"),
            gpu_buffer::USAGE_MAP_READ | gpu_buffer::USAGE_COPY_DST,
            width as i64 * height as i64 * block_size as i64,
        );

        let image = Rc::new(RefCell::new(None));
        let callback_image = image.clone();
        let callback_buffer = buffer.clone();
        let encoder = device.create_command_encoder();
        encoder.copy_texture_to_buffer(
            &**source_texture,
            &buffer,
            0,
            Box::new(move || {
                let data = callback_buffer.map_read();
                let read = |index: i32| {
                    let start = (index * block_size) as usize;
                    i32::from_le_bytes(data[start..start + 4].try_into().expect("four bytes"))
                };
                let output_height = height / downscale_factor;
                let output_width = width / downscale_factor;
                let mut image = NativeImage::new(output_width, output_height);
                for y in 0..output_height {
                    for x in 0..output_width {
                        if downscale_factor == 1 {
                            let argb = read(x + y * width);
                            image.set_pixel_abgr(x, height - y - 1, argb | 0xFF00_0000u32 as i32);
                        } else {
                            let (mut red, mut green, mut blue) = (0, 0, 0);
                            for i in 0..downscale_factor {
                                for j in 0..downscale_factor {
                                    let argb = read(
                                        x * downscale_factor
                                            + i
                                            + (y * downscale_factor + j) * width,
                                    );
                                    red += ARGB::red(argb);
                                    green += ARGB::green(argb);
                                    blue += ARGB::blue(argb);
                                }
                            }
                            let sample_count = downscale_factor * downscale_factor;
                            image.set_pixel_abgr(
                                x,
                                output_height - y - 1,
                                ARGB::color(
                                    255,
                                    red / sample_count,
                                    green / sample_count,
                                    blue / sample_count,
                                ),
                            );
                        }
                    }
                }
                *callback_image.borrow_mut() = Some(image);
            }),
            0,
        );
        Screenshot::submit_until(encoder, || image.borrow().is_some());
        image.borrow_mut().take().expect("the copy's callback ran")
    }
}
