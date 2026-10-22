//! com/mojang/renderpearl/api/commands/RenderPassDescriptor.java

use crate::com::mojang::renderpearl::api::commands::render_pass::RenderArea;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::org::joml::vector4f::Vector4f;

pub struct RenderPassDescriptor {
    pub label: String,
    pub color_attachments: Vec<Option<Attachment<Option<Vector4f>>>>,
    pub depth_attachment: Option<Attachment<Option<f64>>>,
    pub render_area: RenderArea,
}

impl RenderPassDescriptor {
    pub fn builder(label: &str) -> Builder {
        Builder {
            label: label.to_owned(),
            color_attachments: Vec::new(),
            depth_attachment: None,
            render_area: None,
        }
    }
}

/// `RenderPassDescriptor.Attachment`: a view, and what to clear it to first if anything.
#[derive(Clone)]
pub struct Attachment<T> {
    pub texture_view: GpuTextureView,
    pub clear_value: T,
}

pub struct Builder {
    label: String,
    color_attachments: Vec<Option<Attachment<Option<Vector4f>>>>,
    depth_attachment: Option<Attachment<Option<f64>>>,
    render_area: Option<RenderArea>,
}

impl Builder {
    /// `withColorAttachment(GpuTextureView, Optional<Vector4fc>)`.
    pub fn with_color_attachment(
        mut self,
        texture_view: GpuTextureView,
        clear_value: Option<Vector4f>,
    ) -> Builder {
        self.color_attachments.push(Some(Attachment {
            texture_view,
            clear_value,
        }));
        self
    }

    pub fn with_unused_color_attachment(mut self) -> Builder {
        self.color_attachments.push(None);
        self
    }

    /// `withDepthAttachment(GpuTextureView, OptionalDouble)`.
    pub fn with_depth_attachment(
        mut self,
        texture_view: GpuTextureView,
        clear_value: Option<f64>,
    ) -> Builder {
        self.depth_attachment = Some(Attachment {
            texture_view,
            clear_value,
        });
        self
    }

    pub fn with_render_area(mut self, render_area: RenderArea) -> Builder {
        self.render_area = Some(render_area);
        self
    }

    pub fn build(self) -> RenderPassDescriptor {
        let render_area = self.render_area.unwrap_or_else(|| {
            Builder::default_render_area(&self.color_attachments, &self.depth_attachment)
        });
        RenderPassDescriptor {
            label: self.label,
            color_attachments: self.color_attachments,
            depth_attachment: self.depth_attachment,
            render_area,
        }
    }

    /// The last color attachment's size, else the depth attachment's.
    fn default_render_area(
        color_attachments: &[Option<Attachment<Option<Vector4f>>>],
        depth_attachment: &Option<Attachment<Option<f64>>>,
    ) -> RenderArea {
        let (mut width, mut height) = (0, 0);
        if !color_attachments.is_empty() {
            for attachment in color_attachments.iter().flatten() {
                width = attachment.texture_view.get_width(0);
                height = attachment.texture_view.get_height(0);
            }
        } else if let Some(depth) = depth_attachment {
            width = depth.texture_view.get_width(0);
            height = depth.texture_view.get_height(0);
        }
        RenderArea {
            x: 0,
            y: 0,
            width,
            height,
        }
    }
}
