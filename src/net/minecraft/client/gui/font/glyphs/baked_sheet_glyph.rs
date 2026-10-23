//! net/minecraft/client/gui/font/glyphs/BakedSheetGlyph.java
//!
//! A glyph in a font texture: its UVs there, where it's drawn from its origin, and the GUI
//! pipeline for the texture's kind. Its instances are drawn as quads: the shadow first (and its
//! bold copy), then the glyph (and its bold copy), sheared for italics and thickened for bold.
//! It's also the effect glyph (`EffectGlyph`) the white glyph draws underlines, strikethroughs and
//! backgrounds with. The render types for the level's text (`GlyphRenderTypes`' `RenderType`s)
//! come with text in the level; the GUI only reads the pipeline.

use std::rc::Rc;
use std::sync::Arc;

use crate::com::mojang::blaze3d::font::glyph_info::GlyphInfo;
use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::java::lang::math::Math;
use crate::net::minecraft::network::chat::style::Style;
use crate::org::joml::matrix4f::Matrix4f;

pub struct BakedSheetGlyph {
    info: GlyphInfo,
    gui_pipeline: Arc<RenderPipeline>,
    /// None where the font texture has no device to be on (the goldens').
    texture_view: Option<GpuTextureView>,
    pub u0: f32,
    pub u1: f32,
    pub v0: f32,
    pub v1: f32,
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
}

/// `BakedSheetGlyph.GlyphInstance`: a glyph at a place, in a style.
#[derive(Clone)]
pub struct GlyphInstance {
    pub x: f32,
    pub y: f32,
    pub color: i32,
    pub shadow_color: i32,
    pub glyph: Rc<BakedSheetGlyph>,
    pub style: Style,
    pub bold_offset: f32,
    pub shadow_offset: f32,
}

/// `BakedSheetGlyph.EffectInstance`: a rectangle drawn with the glyph.
#[derive(Clone)]
pub struct EffectInstance {
    pub glyph: Rc<BakedSheetGlyph>,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub depth: f32,
    pub color: i32,
    pub shadow_color: i32,
    pub shadow_offset: f32,
}

impl BakedSheetGlyph {
    pub const Z_FIGHTER: f32 = 0.001;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        info: GlyphInfo,
        gui_pipeline: Arc<RenderPipeline>,
        texture_view: Option<GpuTextureView>,
        u0: f32,
        u1: f32,
        v0: f32,
        v1: f32,
        left: f32,
        right: f32,
        up: f32,
        down: f32,
    ) -> BakedSheetGlyph {
        BakedSheetGlyph {
            info,
            gui_pipeline,
            texture_view,
            u0,
            u1,
            v0,
            v1,
            left,
            right,
            up,
            down,
        }
    }

    pub fn info(&self) -> GlyphInfo {
        self.info
    }

    pub fn gui_pipeline(&self) -> &Arc<RenderPipeline> {
        &self.gui_pipeline
    }

    pub fn texture_view(&self) -> Option<&GpuTextureView> {
        self.texture_view.as_ref()
    }

    fn left_of(&self, instance: &GlyphInstance) -> f32 {
        instance.x
            + self.left
            + if instance.style.is_italic() {
                Math::min_f32(self.shear_top(), self.shear_bottom())
            } else {
                0.0
            }
            - extra_thickness(instance.style.is_bold())
    }

    fn top_of(&self, instance: &GlyphInstance) -> f32 {
        instance.y + self.up - extra_thickness(instance.style.is_bold())
    }

    fn right_of(&self, instance: &GlyphInstance) -> f32 {
        instance.x
            + self.right
            + if instance.has_shadow() {
                instance.shadow_offset
            } else {
                0.0
            }
            + if instance.style.is_italic() {
                Math::max_f32(self.shear_top(), self.shear_bottom())
            } else {
                0.0
            }
            + extra_thickness(instance.style.is_bold())
    }

    fn bottom_of(&self, instance: &GlyphInstance) -> f32 {
        instance.y
            + self.down
            + if instance.has_shadow() {
                instance.shadow_offset
            } else {
                0.0
            }
            + extra_thickness(instance.style.is_bold())
    }

    fn render_char(
        &self,
        glyph_instance: &GlyphInstance,
        pose: &Matrix4f,
        buffer: &mut dyn VertexConsumer,
        packed_light_coords: i32,
        flat: bool,
    ) {
        let style = &glyph_instance.style;
        let italic = style.is_italic();
        let x = glyph_instance.x;
        let y = glyph_instance.y;
        let color = glyph_instance.color;
        let bold = style.is_bold();
        let z_fighter = if flat { 0.0 } else { 0.001 };
        let depth = if glyph_instance.has_shadow() {
            let shadow_color = glyph_instance.shadow_color;
            self.render(
                italic,
                x + glyph_instance.shadow_offset,
                y + glyph_instance.shadow_offset,
                0.0,
                pose,
                buffer,
                shadow_color,
                bold,
                packed_light_coords,
            );
            if bold {
                self.render(
                    italic,
                    x + glyph_instance.bold_offset + glyph_instance.shadow_offset,
                    y + glyph_instance.shadow_offset,
                    z_fighter,
                    pose,
                    buffer,
                    shadow_color,
                    true,
                    packed_light_coords,
                );
            }
            if flat { 0.0 } else { 0.03 }
        } else {
            0.0
        };

        self.render(
            italic,
            x,
            y,
            depth,
            pose,
            buffer,
            color,
            bold,
            packed_light_coords,
        );
        if bold {
            self.render(
                italic,
                x + glyph_instance.bold_offset,
                y,
                depth + z_fighter,
                pose,
                buffer,
                color,
                true,
                packed_light_coords,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        italic: bool,
        x: f32,
        y: f32,
        z: f32,
        pose: &Matrix4f,
        builder: &mut dyn VertexConsumer,
        color: i32,
        bold: bool,
        packed_light_coords: i32,
    ) {
        let x0 = x + self.left;
        let x1 = x + self.right;
        let y0 = y + self.up;
        let y1 = y + self.down;
        let shear_y0 = if italic { self.shear_top() } else { 0.0 };
        let shear_y1 = if italic { self.shear_bottom() } else { 0.0 };
        let extra_thickness = extra_thickness(bold);
        builder
            .add_vertex_matrix(
                pose,
                x0 + shear_y0 - extra_thickness,
                y0 - extra_thickness,
                z,
            )
            .set_color(color)
            .set_uv(self.u0, self.v0)
            .set_light(packed_light_coords);
        builder
            .add_vertex_matrix(
                pose,
                x0 + shear_y1 - extra_thickness,
                y1 + extra_thickness,
                z,
            )
            .set_color(color)
            .set_uv(self.u0, self.v1)
            .set_light(packed_light_coords);
        builder
            .add_vertex_matrix(
                pose,
                x1 + shear_y1 + extra_thickness,
                y1 + extra_thickness,
                z,
            )
            .set_color(color)
            .set_uv(self.u1, self.v1)
            .set_light(packed_light_coords);
        builder
            .add_vertex_matrix(
                pose,
                x1 + shear_y0 + extra_thickness,
                y0 - extra_thickness,
                z,
            )
            .set_color(color)
            .set_uv(self.u1, self.v0)
            .set_light(packed_light_coords);
    }

    fn shear_bottom(&self) -> f32 {
        1.0 - 0.25 * self.down
    }

    fn shear_top(&self) -> f32 {
        1.0 - 0.25 * self.up
    }

    fn render_effect(
        &self,
        effect: &EffectInstance,
        pose: &Matrix4f,
        buffer: &mut dyn VertexConsumer,
        packed_light_coords: i32,
        flat: bool,
    ) {
        let mut depth = if flat { 0.0 } else { effect.depth };
        if effect.has_shadow() {
            self.build_effect(
                effect,
                effect.shadow_offset,
                depth,
                effect.shadow_color,
                buffer,
                packed_light_coords,
                pose,
            );
            depth += if flat { 0.0 } else { 0.03 };
        }
        self.build_effect(
            effect,
            0.0,
            depth,
            effect.color,
            buffer,
            packed_light_coords,
            pose,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn build_effect(
        &self,
        effect: &EffectInstance,
        offset: f32,
        z: f32,
        color: i32,
        buffer: &mut dyn VertexConsumer,
        packed_light_coords: i32,
        pose: &Matrix4f,
    ) {
        buffer
            .add_vertex_matrix(pose, effect.x0 + offset, effect.y1 + offset, z)
            .set_color(color)
            .set_uv(self.u0, self.v0)
            .set_light(packed_light_coords);
        buffer
            .add_vertex_matrix(pose, effect.x1 + offset, effect.y1 + offset, z)
            .set_color(color)
            .set_uv(self.u0, self.v1)
            .set_light(packed_light_coords);
        buffer
            .add_vertex_matrix(pose, effect.x1 + offset, effect.y0 + offset, z)
            .set_color(color)
            .set_uv(self.u1, self.v1)
            .set_light(packed_light_coords);
        buffer
            .add_vertex_matrix(pose, effect.x0 + offset, effect.y0 + offset, z)
            .set_color(color)
            .set_uv(self.u1, self.v0)
            .set_light(packed_light_coords);
    }

    /// `createEffect`: the glyph as an effect's rectangle.
    #[allow(clippy::too_many_arguments)]
    pub fn create_effect(
        self: &Rc<BakedSheetGlyph>,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        depth: f32,
        color: i32,
        shadow_color: i32,
        shadow_offset: f32,
    ) -> EffectInstance {
        EffectInstance {
            glyph: Rc::clone(self),
            x0,
            y0,
            x1,
            y1,
            depth,
            color,
            shadow_color,
            shadow_offset,
        }
    }
}

fn extra_thickness(bold: bool) -> f32 {
    if bold { 0.1 } else { 0.0 }
}

impl GlyphInstance {
    fn has_shadow(&self) -> bool {
        self.shadow_color != 0
    }

    pub fn left(&self) -> f32 {
        self.glyph.left_of(self)
    }

    pub fn top(&self) -> f32 {
        self.glyph.top_of(self)
    }

    pub fn right(&self) -> f32 {
        self.glyph.right_of(self)
    }

    pub fn bottom(&self) -> f32 {
        self.glyph.bottom_of(self)
    }

    /// `activeRight()`: the glyph's advance from its place.
    pub fn active_right(&self) -> f32 {
        self.x + self.glyph.info.get_advance_bold(self.style.is_bold())
    }

    pub fn render(
        &self,
        pose: &Matrix4f,
        buffer: &mut dyn VertexConsumer,
        packed_light_coords: i32,
        flat: bool,
    ) {
        self.glyph
            .render_char(self, pose, buffer, packed_light_coords, flat);
    }
}

impl EffectInstance {
    fn has_shadow(&self) -> bool {
        self.shadow_color != 0
    }

    pub fn left(&self) -> f32 {
        self.x0
    }

    pub fn top(&self) -> f32 {
        self.y0
    }

    pub fn right(&self) -> f32 {
        self.x1
            + if self.has_shadow() {
                self.shadow_offset
            } else {
                0.0
            }
    }

    pub fn bottom(&self) -> f32 {
        self.y1
            + if self.has_shadow() {
                self.shadow_offset
            } else {
                0.0
            }
    }

    /// `render`: Java's effects are never flat.
    pub fn render(
        &self,
        pose: &Matrix4f,
        buffer: &mut dyn VertexConsumer,
        packed_light_coords: i32,
        _flat: bool,
    ) {
        self.glyph
            .render_effect(self, pose, buffer, packed_light_coords, false);
    }
}
