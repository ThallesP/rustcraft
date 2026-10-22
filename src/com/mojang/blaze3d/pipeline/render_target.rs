//! com/mojang/blaze3d/pipeline/RenderTarget.java
//!
//! A color texture and a depth texture of one size, which passes draw into. Only making them and
//! handing them out so far; resizing, blitting and destroying come with the frame graph.

use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use std::rc::Rc;

pub struct RenderTarget {
    pub width: i32,
    pub height: i32,
    label: String,
    color_format: Option<GpuFormat>,
    depth_format: Option<GpuFormat>,
    color_texture: Option<Rc<dyn GpuTexture>>,
    color_texture_view: Option<GpuTextureView>,
    depth_texture: Option<Rc<dyn GpuTexture>>,
    depth_texture_view: Option<GpuTextureView>,
}

impl RenderTarget {
    pub fn new(
        label: Option<&str>,
        color_format: Option<GpuFormat>,
        depth_format: Option<GpuFormat>,
    ) -> RenderTarget {
        RenderTarget {
            width: 0,
            height: 0,
            label: label.unwrap_or_default().to_owned(),
            color_format,
            depth_format,
            color_texture: None,
            color_texture_view: None,
            depth_texture: None,
            depth_texture_view: None,
        }
    }

    /// Every usage (15): copied from and to, sampled, and rendered into.
    pub fn create_buffers(&mut self, device: &dyn GpuDevice, width: i32, height: i32) {
        if let Some(depth_format) = self.depth_format {
            let texture = device.create_texture(
                Some(&format!("{} / Depth", self.label)),
                15,
                depth_format,
                width,
                height,
                1,
                1,
            );
            self.depth_texture_view = Some(device.create_texture_view(&texture));
            self.depth_texture = Some(texture);
        }
        if let Some(color_format) = self.color_format {
            let texture = device.create_texture(
                Some(&format!("{} / Color", self.label)),
                15,
                color_format,
                width,
                height,
                1,
                1,
            );
            self.color_texture_view = Some(device.create_texture_view(&texture));
            self.color_texture = Some(texture);
        }
        self.width = width;
        self.height = height;
    }

    pub fn get_color_texture(&self) -> Option<&Rc<dyn GpuTexture>> {
        self.color_texture.as_ref()
    }

    pub fn get_color_texture_view(&self) -> Option<&GpuTextureView> {
        self.color_texture_view.as_ref()
    }

    pub fn get_depth_texture(&self) -> Option<&Rc<dyn GpuTexture>> {
        self.depth_texture.as_ref()
    }

    pub fn get_depth_texture_view(&self) -> Option<&GpuTextureView> {
        self.depth_texture_view.as_ref()
    }
}
