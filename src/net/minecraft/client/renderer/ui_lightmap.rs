//! net/minecraft/client/renderer/UiLightmap.java
//!
//! The lightmap the GUI's draws bind while the GUI renders (`GameRenderer.useUiLightmap`): one
//! white pixel, a `DynamicTexture`'s (which isn't ported otherwise) texture and view.

use std::rc::Rc;

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::{self, GpuTexture};
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;

pub struct UiLightmap {
    _texture: Rc<dyn GpuTexture>,
    texture_view: GpuTextureView,
}

impl UiLightmap {
    pub fn new(device: &dyn GpuDevice) -> UiLightmap {
        let texture = device.create_texture(
            Some("UI Lightmap"),
            gpu_texture::USAGE_COPY_DST | gpu_texture::USAGE_TEXTURE_BINDING,
            GpuFormat::RGBA8_UNORM,
            1,
            1,
            1,
            1,
        );
        let mut pixels = NativeImage::new(1, 1);
        pixels.set_pixel(0, 0, -1);
        device
            .create_command_encoder()
            .write_to_texture_whole(&*texture, &pixels);
        let texture_view = device.create_texture_view(&texture);
        UiLightmap {
            _texture: texture,
            texture_view,
        }
    }

    pub fn get_texture_view(&self) -> &GpuTextureView {
        &self.texture_view
    }
}
